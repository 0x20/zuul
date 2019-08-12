use std::fs::File;
use std::io::{prelude::*, BufRead, BufReader, Error as IoError};
use std::path::Path;

mod llparser;
mod parser;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum FilterComponent {
    Day(u8),                       // bit 0 is Mon, 1 is Tues, ... bit 6 is Sun
    Time { start: u16, end: u16 }, // both in minutes since midnight. Both are inclusive
    Number(String),                // The number to be recognized
    Label(String),
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Filter(Vec<FilterComponent>);

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Whitelist(Vec<Filter>);

pub enum Day {}

impl Day {
    const MON: u8 = 0x01;
    const TUE: u8 = 0x02;
    const WED: u8 = 0x04;
    const THU: u8 = 0x08;
    const FRI: u8 = 0x10;
    const SAT: u8 = 0x20;
    const SUN: u8 = 0x40;
}
