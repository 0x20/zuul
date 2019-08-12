use std::path::PathBuf;
use structopt::StructOpt;

mod blink;
mod config;
mod event;
mod mainloop;
mod modem;
mod whitelist;

#[derive(StructOpt, Debug, Default)]
struct Options {
    #[structopt(short = "w", long = "whitelist")]
    whitelist_filename: PathBuf,
    #[structopt(short = "n")]
    no_relay: bool,
    #[structopt(short = "s", long = "mqtt-server")]
    server: Option<String>,
    #[structopt(short = "c", long = "mqtt-client-id")]
    client_id: Option<String>,
    #[structopt(short = "m", long = "modem")]
    modem_port: PathBuf,
}

fn main() {
    let options: Options = StructOpt::from_args();
}
