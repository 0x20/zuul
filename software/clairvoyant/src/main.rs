use crate::blink::Blinky;
use crate::whitelist::Whitelist;
use failure::Error;
use failure::_core::time::Duration;
use rppal::gpio::Gpio;
use slog::{o, Drain, Logger};
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use structopt::StructOpt;

mod blink;
mod event;
mod mainloop;
mod modem;
mod timer;
mod whitelist;

#[derive(StructOpt, Debug, Default)]
struct Options {
    #[structopt(short = "w", long = "whitelist")]
    whitelist_filename: PathBuf,
    #[structopt(short = "n")]
    no_relay: bool,
    #[structopt(short = "s", long = "mqtt-server")]
    server: Option<String>,
    #[structopt(short = "m", long = "modem", default_value = "/dev/ttyAMA0")]
    modem_port: PathBuf,
    #[structopt(short = "j", long = "use-journald")]
    use_journald: bool,
}

fn init_logger(journald: bool) -> Logger {
    let drain = if journald {
        let drain = slog_journald::JournaldDrain.fuse();
        slog_async::Async::default(drain)
    } else {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        slog_async::Async::default(drain)
    };
    slog::Logger::root(drain.ignore_res(), o!())
}

fn main() -> Result<(), Error> {
    let options: Options = StructOpt::from_args();
    let gpio = Gpio::new()?;

    let (chan_snd, chan_rcv) = channel();

    let logger = init_logger(options.use_journald);

    let modem = modem::Modem::new(
        &options.modem_port,
        chan_snd.clone(),
        gpio.get(17)?.into_output(),
        logger.new(o! {
            "component" => "modem",
        }),
    )?;

    let mqtt = if let Some(server_uri) = options.server {
        let mqtt = paho_mqtt::Client::new(server_uri)?;
        mqtt.connect(
            paho_mqtt::ConnectOptionsBuilder::new()
                .clean_session(true)
                .will_message(paho_mqtt::Message::new_retained("zuul/online", "false", 0))
                .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(32))
                .finalize(),
        )?;
        mqtt
    } else {
        paho_mqtt::Client::new(String::new())?
    };

    let modem_thread = modem.spawn()?;
    timer::timer(chan_snd);

    mainloop::MainLoop {
        event_chan: chan_rcv,
        logger,
        gpio_door: gpio.get(27)?.into_output(),
        mqtt: mqtt,
        rpi_ok: Blinky::new(gpio.get(22)?.into_output(), Cow::Borrowed(blink::PAT_OFF)),
        gsm_ok: Blinky::new(gpio.get(23)?.into_output(), Cow::Borrowed(blink::PAT_OFF)),
        whitelist: Whitelist::new(options.whitelist_filename)?,
    }
    .run();

    modem_thread
        .join()
        .map_err(|_| failure::err_msg("Modem thread panicked"))?;

    Ok(())
}
