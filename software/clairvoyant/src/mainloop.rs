use std::borrow::Cow;
use std::sync::mpsc::Receiver;

use embedded_hal::digital::v2::OutputPin;
use paho_mqtt::Client as MqttClient;
use slog::{warn, Logger};

use crate::blink::Blinky;
use crate::event::{Event, Regstate};
use crate::whitelist::{MatchContext, Whitelist};

pub struct MainLoop<DP: OutputPin> {
    pub event_chan: Receiver<Event>,
    pub logger: Logger,
    pub gpio_door: DP,
    pub mqtt: MqttClient,
    pub rpi_ok: Blinky<'static, DP>,
    pub gsm_ok: Blinky<'static, DP>,

    pub whitelist: Whitelist,
}

impl<DP: OutputPin> MainLoop<DP> {
    pub fn run(&mut self) {
        use crate::blink;
        use std::time::{Duration, Instant};
        let mut last_gsm_ok = Instant::now() - Duration::from_secs(1000);
        let mut gsm_notok = true;
        let mut blink_pat = Cow::Borrowed(blink::PAT_OFF);
        while let Ok(event) = self.event_chan.recv() {
            match event {
                Event::Ring(number) => self.handle_call(number),
                Event::Creg(regstate) => {
                    blink_pat = Cow::Borrowed(match regstate {
                        Regstate::Unregistered => blink::PAT_OFF,
                        Regstate::Registered => blink::PAT_SLOW,
                        Regstate::Searching => blink::PAT_FAST,
                        Regstate::Denied => blink::PAT_SOS,
                        Regstate::Roaming => blink::PAT_VSLOW,
                        Regstate::Unknown(rs) => {
                            warn!(self.logger, "Unknown regstate"; "regstate" => rs);
                            blink::PAT_OFF
                        }
                    });
                    last_gsm_ok = Instant::now();
                    self.gsm_ok.change_pattern(blink_pat.clone());
                    gsm_notok = false;
                }
                Event::GsmOk => {
                    last_gsm_ok = Instant::now();
                    if gsm_notok {
                        self.gsm_ok.change_pattern(blink_pat.clone())
                    }
                }
                Event::Heartbeat => {
                    if last_gsm_ok.elapsed() > Duration::from_secs(30) {
                        self.gsm_ok.change_pattern(Cow::Borrowed(blink::PAT_OFF));
                        gsm_notok = true;
                    }
                    self.gsm_ok.step();
                    self.rpi_ok.step();
                }
            }
        }
    }

    pub fn handle_call(&mut self, number: String) {
        use paho_mqtt::Message;
        self.mqtt
            .publish(Message::new("zuul/ring", number.as_bytes(), 0))
            .ok();

        if let Some(label) = self.whitelist.matches(&MatchContext::new(&number)) {
            self.mqtt
                .publish(Message::new("zuul/open", label.unwrap_or("anon"), 0))
                .ok();
        }
    }
}
