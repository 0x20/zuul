use std::borrow::Cow;

use embedded_hal::digital::v2::OutputPin;

pub struct Blinky<'a, Pin: OutputPin> {
    pattern: Cow<'a, [u8]>,
    pin: Pin,
    pos: usize,
    delay: u8,
    state: bool,
}

#[allow(unused)]
mod patterns {
    pub const PAT_ON: &[u8] = b"\xF0";
    pub const PAT_OFF: &[u8] = b"\x0F";
    pub const PAT_SLOW: &[u8] = b"\x55";
    pub const PAT_VSLOW: &[u8] = b"\xAA";
    pub const PAT_FAST: &[u8] = b"\x22";
    pub const PAT_HEARTBEAT: &[u8] = b"\x22\x26";
    pub const PAT_SOS: &[u8] = b"\x22\x22\x22\x62\x62\x62\x22\x22\x2C";
}

pub use patterns::*;

impl<'a, Pin: OutputPin> Blinky<'a, Pin> {
    pub fn new(pin: Pin, pattern: Cow<'a, [u8]>) -> Self {
        let pos = pattern.len() - 1;
        Blinky {
            pattern,
            pin,
            pos,
            delay: 0,
            state: false,
        }
    }

    pub fn step(&mut self) {
        let last_state = self.state;
        while self.delay == 0 {
            // step to the next state
            if self.state {
                // pin is on, switch to off state
                self.delay = self.pattern[self.pos] & 0xF;
                self.state = false;
            } else {
                self.pos = (self.pos + 1) % self.pattern.len();
                self.delay = self.pattern[self.pos] >> 4;
                self.state = true;
            }
        }
        if self.state != last_state {
            if self.state {
                self.pin.set_high().ok();
            } else {
                self.pin.set_low().ok();
            }
        }
        self.delay -= 1;
    }

    /// Patterns are a sequence of bytes, each one with a time on in the high nibble and a time off in the low nibble.
    /// Times are represented in number of calls to `step()`
    pub fn change_pattern(&mut self, pattern: Cow<'a, [u8]>) {
        if pattern == self.pattern {
            return;
        }
        self.pattern = pattern;
        self.delay = 0;
        self.state = false;
        self.pos = self.pattern.len() - 1;
    }
}
