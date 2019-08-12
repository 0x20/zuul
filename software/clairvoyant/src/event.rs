#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum Regstate {
    Unregistered,
    Registered,
    Searching,
    Denied,
    Roaming,
    Unknown(i32),
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum Event {
    Heartbeat,
    Ring(String),
    Creg(Regstate),
    GsmOk,
}
