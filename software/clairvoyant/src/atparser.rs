use serial::prelude::*;
use std::io::{Error as IoError, Read, Write};
use std::path::Path;
use std::time::Duration;

type Port = serial_unix::TTYPort;
//type Port = serial::SystemPort;

pub enum ModemType {
    Unknown,
    Sim800,
    HuaweiK3765,
}

pub struct Modem {
    port: Port,
    modem_type: ModemType,
    buffer: Vec<u8>,
    urc_buffer: Vec<u8>,
}

impl Modem {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, serial::Error> {
        let mut port: serial::SystemPort = serial::SystemPort::open(path.as_ref())?;

        let mut modem = Modem {
            port,
            modem_type: ModemType::Unknown,
            buffer: Vec::new(),
            urc_buffer: Vec::new(),
        };
        modem.cfg_serial_port()?;
        Ok(modem)
    }

    fn cfg_serial_port(&mut self) -> Result<(), serial::Error> {
        self.port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud115200)?;
            settings.set_flow_control(serial::FlowSoftware);
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            Ok(())
        })
    }

    fn initialize_modem(&mut self) -> Result<(), IoError> {
        self.port.write_all(b"ATE1V1Q0\r\n")?;

        self.port.set_timeout(Duration::new(0, 10_000)); // set timeout to 10ms

        // Read as much as possible from the port to get rid of old input
        loop {
            let buf = &mut [0u8; 128];
            match self.port.read(buf) {
                Ok(_) => continue,
                Err(err) => print!("{:?} {:?}\n", &err, &err.kind()),
            }
        }
        //        let sync_str = "AT+CGMI;+CGMM";
    }
}
