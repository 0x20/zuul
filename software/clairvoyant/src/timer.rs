use std::sync::mpsc::Sender;
use std::thread::{spawn, JoinHandle};

use crate::event::Event;

pub fn timer(chan: Sender<Event>) -> JoinHandle<()> {
    let dur = std::time::Duration::from_millis(10);
    spawn(move || loop {
        std::thread::sleep(dur);
        chan.send(Event::Heartbeat).ok();
    })
}
