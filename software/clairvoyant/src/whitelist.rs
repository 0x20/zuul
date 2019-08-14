use chrono::{Datelike, Timelike};
use std::path::{Path, PathBuf};

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
pub struct Whitelist {
    cache: Vec<Filter>,
    source: PathBuf,
}

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

pub struct MatchContext<'a> {
    number: &'a str,
    day: u8,
    time: u16,
}

impl<'a> MatchContext<'a> {
    pub fn new(number: &'a str) -> Self {
        let now = chrono::Local::now();
        let day = 1u8 << now.weekday().num_days_from_monday() as u8;
        let time = (now.hour() * 60 + now.minute()) as u16;
        MatchContext { number, day, time }
    }
}

impl FilterComponent {
    fn matches(&self, ctx: &MatchContext) -> bool {
        match self {
            FilterComponent::Day(d) => (ctx.day & *d) != 0,

            FilterComponent::Time { start, end } => ctx.time >= *start && ctx.time <= *end,
            FilterComponent::Number(num) => ctx.number == num,
            FilterComponent::Label(_) => true,
        }
    }

    fn label(&self) -> Option<&str> {
        if let FilterComponent::Label(lbl) = self {
            Some(lbl)
        } else {
            None
        }
    }
}

impl Filter {
    /// Returns
    /// None if it doesn't match,
    /// Some(None) if it matches an unlabeled line
    /// Some(Some(str)) if it matches a labelled line
    fn matches(&self, ctx: &MatchContext) -> Option<Option<&str>> {
        if self.0.iter().all(|component| component.matches(ctx)) {
            Some(self.0.iter().flat_map(FilterComponent::label).next())
        } else {
            None
        }
    }
}

fn parse_file(path: &Path) -> std::io::Result<Vec<Filter>> {
    use std::io::{Error, ErrorKind};
    let source = std::fs::read_to_string(path)?;
    let (rest, parsed) =
        parser::config(&source).map_err(|err: nom::Err<nom::error::VerboseError<_>>| {
            Error::new(ErrorKind::InvalidData, failure::err_msg("Parse error"))
        })?;
    if rest != "" {
        return Err(Error::new(
            ErrorKind::InvalidData,
            failure::err_msg("Invalid data").compat(),
        ));
    }

    return Ok(parsed);
}

impl Whitelist {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let source = path.as_ref().to_owned();
        let cache = parse_file(&source)?;
        Ok(Whitelist { cache, source })
    }

    pub(crate) fn matches(&self, ctx: &MatchContext) -> Option<Option<&str>> {
        let mut matched = false;
        for filter in self.cache.iter() {
            if let Some(label) = filter.matches(ctx) {
                matched = true;
                if label.is_some() {
                    return Some(label);
                }
            }
        }
        if matched {
            return Some(None);
        } else {
            return None;
        }
    }
}
