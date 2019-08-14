use std::io::{prelude::*, BufRead, BufReader, Error as IoError};
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use embedded_hal::digital::v2::OutputPin;
use lazy_static::lazy_static;
use regex::bytes::Regex;
use serial::prelude::*;

use crate::event::{Event, Regstate};
use slog::{debug, info, warn, Logger};
use std::time::Duration;

lazy_static! {
    static ref CREG_RE: Regex = Regex::new(r"\+CREG: *(?:\d*,)(\d+)\r\n").unwrap();
    static ref CLIP_RE: Regex = Regex::new(r#"\+CLIP: *"([^"]+)""#).unwrap();
    static ref CPIN_RE: Regex = Regex::new(r"\+CPIN: *([^\r\n]+)\r\n").unwrap();
}

type Port = serial_unix::TTYPort;

pub struct Modem<PP: OutputPin> {
    port: BufReader<Port>,
    chan: mpsc::Sender<Event>,
    pwr_gpio: PP,
    logger: Logger,
}

impl<PP: OutputPin + 'static> Modem<PP> {
    pub fn new<P: AsRef<Path>>(
        path: P,
        chan: mpsc::Sender<Event>,
        pwr_gpio: PP,
        logger: Logger,
    ) -> Result<Self, serial::Error> {
        let mut port = BufReader::new(serial::SystemPort::open(path.as_ref())?);

        port.get_mut().reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud115200)?;
            settings.set_flow_control(serial::FlowSoftware);
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            Ok(())
        })?;

        Ok(Modem {
            port,
            chan,
            pwr_gpio,
            logger,
        })
    }

    fn send_cmd(&mut self, cmd: &[u8]) -> Result<(), IoError> {
        self.port.get_mut().write_all(cmd)
    }

    pub fn spawn(self) -> Result<thread::JoinHandle<()>, IoError>
    where
        PP: Send,
    {
        thread::Builder::new()
            .name("modem".to_string())
            .spawn(move || {
                self.run();
            })
    }

    fn run(mut self) {
        // Start by making sure that the GSM is powered down, so we can power it up in a known state
        self.pwr_gpio.set_low().ok();
        self.send_cmd(b"AT+CPOWD=1\n").unwrap();
        std::thread::sleep(Duration::from_secs(1));
        self.pwr_gpio.set_high().ok();
        let mut line = Vec::new();
        loop {
            // Wait for RDY
            line.clear();
            self.port
                .read_until(0x0a, &mut line)
                .expect("Failed to read data");
            debug!(self.logger, "Received input"; "line" => String::from_utf8_lossy(&line));

            if line == b"RDY\r\n" {
                self.pwr_gpio.set_low().ok();
            } else if let Some(cpin) = Regex::captures(&CPIN_RE, &line) {
                // PIN request
                match &cpin[1] {
                    b"SIM PIN" => {
                        info!(self.logger, "Unlocking SIM");
                        self.send_cmd(b"ATQ0V1E1+CREG=1;+CLIP=1;+CPIN=1111\n")
                            .unwrap();
                    }
                    b"READY" => {
                        info!(self.logger, "SIM unlocked");
                    }
                    other => {
                        warn!(self.logger, "Unknown PIN state"; "cpin" => &*String::from_utf8_lossy(other));
                    }
                }
            } else if let Some(creg) = Regex::captures(&CREG_RE, &line) {
                let raw_data = String::from_utf8_lossy(&creg[1]);
                let state = match i32::from_str_radix(&raw_data, 10) {
                    Ok(0) => Regstate::Unregistered,
                    Ok(1) => Regstate::Registered,
                    Ok(2) => Regstate::Searching,
                    Ok(3) => Regstate::Denied,
                    Ok(5) => Regstate::Roaming,
                    Ok(n) => Regstate::Unknown(n),
                    Err(_) => {
                        warn!(self.logger, "Unparsable regstate"; "creg" => &*raw_data);
                        Regstate::Unknown(4)
                    }
                };

                self.chan
                    .send(Event::Creg(state))
                    .expect("Event processing thread is dead");
            } else if let Some(ring) = Regex::captures(&CLIP_RE, &line) {
                self.chan
                    .send(Event::Ring(String::from_utf8_lossy(&ring[1]).into_owned()))
                    .expect("Event processing thread is dead");
            } else {
                debug!(self.logger, "Unrecognized data from modem"; "line" => &*String::from_utf8_lossy(&line))
            }
        }
    }
}
