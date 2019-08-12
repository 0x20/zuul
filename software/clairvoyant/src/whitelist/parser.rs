use nom::{
    branch::*, bytes::complete::*, character::complete::space1, combinator::*, error::ParseError,
    multi::*, sequence::*, Compare, IResult, InputTake, Needed,
};

use super::{Filter, FilterComponent};
use crate::whitelist::Whitelist;
use nom::character::complete::{char, one_of, space0};
use nom::error::ErrorKind;

fn rule<'a, Err: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Filter, Err> {
    map(
        preceded(space0, separated_nonempty_list(space1, filter_component)),
        Filter,
    )(i)
}

fn filter_component<'a, Err: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, FilterComponent, Err> {
    alt((day_filter, time_filter, number_filter, label_filter))(i)
}

fn day_filter<'a, Err: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, FilterComponent, Err> {
    let (i, _) = tag("day")(i)?;
    let (i, _) = space1(i)?;
    let (i, ranges): (&str, Vec<u8>) = separated_nonempty_list(tag(","), day_range)(i)?;
    Ok((
        i,
        FilterComponent::Day(ranges.iter().fold(0, |acc, new| acc | *new)),
    ))
}

fn day_range<'a, Err: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, u8, Err> {
    let (i, start) = day(i)?;
    let (i, end) = opt(preceded(tag("-"), day))(i)?;

    let ret = if let Some(end) = end {
        if end >= start {
            ((end << 1) - start) & 0x7F
        } else {
            (0x80 - start) | ((end << 1) - 1)
        }
    } else {
        start
    };

    Ok((i, ret))
}

/// Takes
/// * day numbers (mon = 1)
/// * english three-letter abbrevs
/// * Dutch two-letter abbrevs, plus "woe" and "vrij"
fn day<'a, Err: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, u8, Err> {
    alt((
        value(0x01, alt((tag("1"), tag_no_case("mon"), tag_no_case("ma")))),
        value(0x02, alt((tag("2"), tag_no_case("tue"), tag_no_case("di")))),
        value(
            0x04,
            alt((
                tag("3"),
                tag_no_case("wed"),
                tag_no_case("woe"),
                tag_no_case("wo"),
            )),
        ),
        value(0x08, alt((tag("4"), tag_no_case("thu"), tag_no_case("do")))),
        value(
            0x10,
            alt((
                tag("5"),
                tag_no_case("fri"),
                tag_no_case("vrij"),
                tag_no_case("vrĳ"), // Just in case somebody copies a ligature
                tag_no_case("vr"),
            )),
        ),
        value(0x20, alt((tag("6"), tag_no_case("sat"), tag_no_case("za")))),
        value(0x40, alt((tag("7"), tag_no_case("sun"), tag_no_case("zo")))),
    ))(i)
}

fn time_filter<'a, Err: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, FilterComponent, Err> {
    let (i, _) = tag("time")(i)?;
    let (i, _) = space1(i)?;
    let (i, (start, end)) = separated_pair(parse_time, char('-'), parse_time)(i)?;
    Ok((i, FilterComponent::Time { start, end }))
}

fn digit<'a, Err: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, u16, Err> {
    map(one_of("0123456789"), |ch| ch.to_digit(10).unwrap() as u16)(i)
}

fn ndigit<'a, Err: ParseError<&'a str>>(
    m: usize,
    n: usize,
) -> impl Fn(&'a str) -> IResult<&'a str, u16, Err> {
    map(many_m_n(m, n, digit), |list| {
        list.iter().fold(0, |acc, new| acc * 10 + *new)
    })
}

fn parse_time<'a, Err: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, u16, Err> {
    map(
        separated_pair(ndigit(1, 2), char(':'), ndigit(2, 2)),
        |(h, m)| h * 60 + m,
    )(i)
}

fn number_filter<'a, Err: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, FilterComponent, Err> {
    let (i, _) = tag("num")(i)?;
    let (i, _) = space1(i)?;
    let (i, num) = is_a("0123456789#*")(i)?;
    Ok((i, FilterComponent::Number(num.to_owned())))
}
fn label_filter<'a, Err: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, FilterComponent, Err> {
    let (i, _) = tag("label")(i)?;
    let (i, _) = space1(i)?;
    let (i, label) = is_a("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789")(i)?;

    Ok((i, FilterComponent::Label(label.to_string())))
}

pub fn comment<'a, Err: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), Err> {
    value(
        (),
        tuple((space0, opt(tuple((char('#'), is_not("\n")))), char('\n'))),
    )(i)
}

pub fn eof<'a, Err: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), Err> {
    if i.len() == 0 {
        Ok((i, ()))
    } else {
        Err(nom::Err::Error(Err::from_error_kind(i, ErrorKind::Eof)))
    }
}

pub fn config<'a, Err: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Whitelist, Err> {
    map(
        all_consuming(preceded(
            many0(comment),
            map(
                many_till(terminated(rule, value((), many1(comment))), eof),
                |(a, _)| a,
            ),
        )),
        Whitelist,
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::error::{ErrorKind, VerboseError};
    use slog::Error;

    //type SimpleError<'a> = (&'a str, ErrorKind);
    type SimpleError<'a> = VerboseError<&'a str>;

    #[test]
    fn test_day_parser() {
        assert_eq!(day::<(&str, ErrorKind)>("mon"), Ok(("", 0x01)));
        assert_eq!(day::<(&str, ErrorKind)>("woe"), Ok(("", 0x04)));
    }

    #[test]
    fn test_normal_day_range() {
        assert_eq!(day_range::<(&str, ErrorKind)>("ma-wo"), Ok(("", 0x07)));
        assert_eq!(day_range::<(&str, ErrorKind)>("za-ma"), Ok(("", 0x61)));
    }

    #[test]
    fn test_config() {
        assert_eq!(
            many0::<_, _, SimpleError, _>(comment)("\n"),
            Ok(("", vec![()]))
        );

        assert_eq!(
            config::<SimpleError>(
                "
            # preceding comment
            day thu time 18:00-24:00
            number 12128675309 label Jenny # End-of-line comment
            
            # line comment\n"
            ),
            Ok((
                "",
                Whitelist(vec![
                    Filter(vec![
                        FilterComponent::Day(0x08),
                        FilterComponent::Time {
                            start: 18 * 60,
                            end: 24 * 60
                        },
                    ]),
                    Filter(vec![
                        FilterComponent::Number("12128675309".to_string()),
                        FilterComponent::Label("Jenny".to_string()),
                    ])
                ])
            ))
        )
    }
}
