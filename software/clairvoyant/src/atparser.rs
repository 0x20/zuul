use lazy_static::lazy_static;
use regex::Regex;
use serial::prelude::*;
use serial_core::SerialDevice;
use std::io::{BufRead, BufReader, Error as IoError, ErrorKind as IoErrorKind, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

lazy_static! {
    static ref ERROR_RE: Regex = Regex::new(
        "^(:?\
         OK|\
         +CM[SE] ERROR:.*\
         ERROR
         )$"
    ).unwrap();
}

type Port = serial_unix::TTYPort;
//type Port = serial::SystemPort;

pub enum ModemType {
    Uninitialized,
    Unknown,
    Sim800,
    HuaweiK3765,
}

pub struct Modem {
    port: BufReader<Port>,
    modem_type: ModemType,
    last_cmd_end: Instant,
    buffer: Vec<u8>,
}

pub struct CommandOutput<'a> {
    command: &'a [u8],
    ready: bool,
    modem: &'a mut Modem,
    done: bool,
}

impl Iterator for CommandOutput {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Result<Self::Item, IoError>> {
        let ERROR_RE: Regex;
        if self.done {
            return None;
        }

        let line = self.modem.get_line(None)?;
        if ready {
            if ERROR_RE.is_match(&line)
        }
    }
}

impl Modem {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, serial::Error> {
        let mut port: serial::SystemPort = serial::SystemPort::open(path.as_ref())?;

        let mut modem = Modem {
            port,
            modem_type: ModemType::Uninitialized,
            buffer: Vec::new(),
            last_cmd_end: Instant::now(),
        };
        modem.cfg_serial_port()?;
        modem.initialize_modem()?;
        Ok(modem)
    }

    fn cfg_serial_port(&mut self) -> Result<(), serial::Error> {
        self.port.get_mut().reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud115200)?;
            settings.set_flow_control(serial::FlowSoftware);
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            Ok(())
        })
    }

    fn initialize_modem(&mut self) -> Result<(), IoError> {
        self.port.get_mut().set_timeout(Duration::new(0, 100_000))?; // set timeout to 100ms

        // Read as much as possible from the port to get rid of old input
        loop {
            let buf = &mut [0u8; 128];
            match self.port.read(buf) {
                Ok(_) => continue,
                Err(err) if err.kind() == IoErrorKind::TimedOut => break,
                Err(err) => return Err(err),
            }
        }

        self.port.write_all(b"ATE1V1Q0\n")?;

        let mfg = self.send_command_short(b"AT+CGMI")?;
        let product = self.send_command_short(b"AT+CGMM")?;

        // Match

        Ok(())
        //        let sync_str = "AT+CGMI;+CGMM";
    }

    fn get_line(&mut self, mut _timeout: Option<Duration>) -> Result<Vec<u8>, IoError> {
        let mut line = Vec::new();
        self.port.read_until(b'\n', &mut line)?;
        if line.ends_with(&b"\n") {
            line.pop();
        }
        if line.ends_with(&b"\r") {
            line.pop();
        }
        Ok(line)
    }

    pub fn send_command(&mut self, command: &[u8]) -> Result<CommandOutput, IoError> {
        {
            let mut command = command.to_owned();
            command.push(b'\n');
            let port = self.port.get_mut().write_all(&command)?;
        }

        Ok(CommandOutput {
            command,
            ready: false,
            modem: self,
            done: false,
        })
    }

    pub fn send_command_short(&mut self, command: &[u8]) -> Result<Vec<u8>, IoError> {
        self.send_command(command).and_then(|output| {
            let mut output = Vec::new();
            output.read_to_end(&mut output)?;
            Ok(output)
        })
    }
}
