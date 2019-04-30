use std::path::{Path, PathBuf};
use structopt::StructOpt;

mod atparser;
mod config;

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

    let modem = atparser::Modem::new(&options.modem_port);
}
