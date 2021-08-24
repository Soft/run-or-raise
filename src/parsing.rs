use regex::Regex;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::space0;
use nom::combinator::{all_consuming, map, map_res, value};
use nom::error::{context, ContextError, FromExternalError, ParseError, VerboseError};
use nom::multi::many0;
use nom::sequence::{delimited, preceded, tuple};
use nom::{Finish, IResult};

use anyhow::Error;

use crate::conditions::*;

fn property<'a, E>(input: &'a str) -> IResult<&str, Property, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "property name",
        alt((
            value(Property::Class, tag("class")),
            value(Property::Name, tag("name")),
            value(Property::Role, tag("role")),
        )),
    )(input)
}

fn escape<'a, E>(input: &'a str) -> IResult<&str, &str, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    preceded(tag(r#"\"#), alt((tag(r#"""#), tag(r#"\"#))))(input)
}

fn no_escapes<'a, E>(input: &'a str) -> IResult<&str, &str, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    is_not(r#"\""#)(input)
}

fn string_content<'a, E>(input: &'a str) -> IResult<&str, String, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    map(many0(alt((no_escapes, escape))), |v| {
        v.into_iter().collect()
    })(input)
}

fn quoted_string<'a, E>(input: &'a str) -> IResult<&str, String, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "string literal",
        delimited(tag("\""), string_content, tag("\"")),
    )(input)
}

fn ws<'a, E>(input: &'a str) -> IResult<&str, (), E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    value((), space0)(input)
}

fn op_equal<'a, E>(input: &'a str) -> IResult<&str, Operator, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    map(tuple((tag("="), ws, quoted_string)), |(_, _, s)| {
        Operator::Equal(s)
    })(input)
}

fn regex<'a, E>(input: &'a str) -> IResult<&str, Regex, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, regex::Error> + ContextError<&'a str>,
{
    map_res(quoted_string, |s| Regex::new(&s))(input)
}

fn op_regex<'a, E>(input: &'a str) -> IResult<&str, Operator, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, regex::Error> + ContextError<&'a str>,
{
    map(tuple((tag("~"), ws, regex)), |(_, _, s)| Operator::Regex(s))(input)
}

fn match_<'a, E>(input: &'a str) -> IResult<&str, Match, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, regex::Error> + ContextError<&'a str>,
{
    context(
        "match expression",
        map(
            tuple((property, ws, alt((op_equal, op_regex)))),
            |(prop, _, op)| Match { prop, op },
        ),
    )(input)
}

fn condition<'a, E>(input: &'a str) -> IResult<&str, Condition, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, regex::Error> + ContextError<&'a str>,
{
    all_consuming(condition_inner)(input)
}

fn condition_inner<'a, E>(input: &'a str) -> IResult<&str, Condition, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, regex::Error> + ContextError<&'a str>,
{
    context("expression", delimited(ws, cond_or, ws))(input)
}

fn cond_or<'a, E>(input: &'a str) -> IResult<&str, Condition, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, regex::Error> + ContextError<&'a str>,
{
    map(
        tuple((
            cond_and,
            context(
                "or expression",
                many0(map(tuple((ws, tag("||"), ws, cond_and)), |(_, _, _, c)| c)),
            ),
        )),
        |(l, r)| {
            r.into_iter()
                .fold(l, |acc, x| Condition::Or(Box::new(acc), Box::new(x)))
        },
    )(input)
}

fn cond_and<'a, E>(input: &'a str) -> IResult<&str, Condition, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, regex::Error> + ContextError<&'a str>,
{
    map(
        tuple((
            cond_not,
            context(
                "and expression",
                many0(map(tuple((ws, tag("&&"), ws, cond_not)), |(_, _, _, c)| c)),
            ),
        )),
        |(l, r)| {
            r.into_iter()
                .fold(l, |acc, x| Condition::And(Box::new(acc), Box::new(x)))
        },
    )(input)
}

fn cond_not<'a, E>(input: &'a str) -> IResult<&str, Condition, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, regex::Error> + ContextError<&'a str>,
{
    map(
        tuple((
            context("not expression", many0(value((), tuple((tag("!"), ws))))),
            cond_parens,
        )),
        |(nots, cond)| {
            nots.into_iter()
                .fold(cond, |acc, _| Condition::Not(Box::new(acc)))
        },
    )(input)
}

fn cond_parens<'a, E>(input: &'a str) -> IResult<&str, Condition, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, regex::Error> + ContextError<&'a str>,
{
    alt((
        context(
            "grouping expression",
            map(
                tuple((tag("("), ws, condition_inner, ws, tag(")"))),
                |(_, _, cond, _, _)| cond,
            ),
        ),
        cond_pure,
    ))(input)
}

fn cond_pure<'a, E>(input: &'a str) -> IResult<&str, Condition, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, regex::Error> + ContextError<&'a str>,
{
    map(match_, Condition::Pure)(input)
}

impl FromStr for Condition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        condition::<VerboseError<_>>(s)
            .finish()
            .map(|(_, v)| v)
            .map_err(|e| Error::msg(nom::error::convert_error(s, e)).context("Invalid condition"))
    }
}

#[test]
fn test_property() {
    let result = property::<()>("class");
    assert_eq!(result.unwrap(), ("", Property::Class));
    let result = property::<()>("name");
    assert_eq!(result.unwrap(), ("", Property::Name));
    let result = property::<()>("role");
    assert_eq!(result.unwrap(), ("", Property::Role));
}

#[test]
fn test_escape() {
    let result = escape::<()>(r#"\""#);
    assert_eq!(result.unwrap(), ("", r#"""#));
    let result = escape::<()>(r#"\\"#);
    assert_eq!(result.unwrap(), ("", r#"\"#));
}

#[test]
fn test_no_escapes() {
    let result = no_escapes::<()>(r#"Hello \"#);
    assert_eq!(result.unwrap(), (r#"\"#, "Hello "));
    let result = no_escapes::<()>(r#"Hello ""#);
    assert_eq!(result.unwrap(), (r#"""#, "Hello "));
}

#[test]
fn test_quoted_string() {
    let result = quoted_string::<()>("\"Hello World\"");
    assert_eq!(result.unwrap(), ("", "Hello World".into()));
    let result = quoted_string::<()>(r#""Hello \"World\"""#);
    assert_eq!(result.unwrap(), ("", "Hello \"World\"".into()));
}

#[test]
fn test_match() {
    let (rest, value) = match_::<()>("class ~ \"Firefox\"").unwrap();
    assert_eq!(rest, "");
    assert_eq!(value.prop, Property::Class);
    if let Operator::Regex(ref p) = value.op {
        assert!(p.is_match("Firefox"));
    } else {
        panic!();
    }
}

#[test]
fn test_condition() {
    let (rest, value) = condition::<VerboseError<_>>(r#"class = "Firefox""#).unwrap();
    assert_eq!(rest, "");
    assert!(matches!(value, Condition::Pure(
            Match {
                prop: Property::Class,
                op: Operator::Equal(ref s)
            }) if s == "Firefox"));

    let (rest, value) =
        condition::<VerboseError<_>>(r#"class = "Firefox" && name = "Emacs""#).unwrap();
    assert_eq!(rest, "");
    let (ls, rs) = match value {
        Condition::And(ls, rs) => (ls, rs),
        _ => panic!(),
    };
    assert!(matches!(*ls, Condition::Pure(Match {
        prop: Property::Class,
        op: Operator::Equal(ref s)
    }) if s == "Firefox"));
    assert!(matches!(*rs, Condition::Pure(Match {
        prop: Property::Name,
        op: Operator::Equal(s)
    }) if s == "Emacs"));

    let (rest, value) =
        condition::<VerboseError<_>>(r#"class = "Firefox" && name = "Emacs" && role = "browser""#)
            .unwrap();
    let (ls, rs) = match value {
        Condition::And(ls, rs) => (ls, rs),
        _ => panic!(),
    };
    assert_eq!(rest, "");
    let (ls_ls, ls_rs) = match *ls {
        Condition::And(ls, rs) => (ls, rs),
        _ => panic!(),
    };
    assert!(matches!(*ls_ls, Condition::Pure(Match {
        prop: Property::Class,
        op: Operator::Equal(ref s)
    }) if s == "Firefox"));
    assert!(matches!(*ls_rs, Condition::Pure(Match {
        prop: Property::Name,
        op: Operator::Equal(s)
    }) if s == "Emacs"));
    assert!(matches!(*rs, Condition::Pure(Match {
        prop: Property::Role,
        op: Operator::Equal(s)
    }) if s == "browser"));

    let (rest, value) =
        condition::<VerboseError<_>>(r#"class = "Firefox" || name = "Emacs" && role = "browser""#)
            .unwrap();
    assert_eq!(rest, "");
    let (ls, rs) = match value {
        Condition::Or(ls, rs) => (ls, rs),
        _ => panic!(),
    };
    assert!(matches!(*ls, Condition::Pure(Match {
        prop: Property::Class,
        op: Operator::Equal(ref s)
    }) if s == "Firefox"));
    let (rs_ls, rs_rs) = match *rs {
        Condition::And(ls, rs) => (ls, rs),
        _ => panic!(),
    };
    assert!(matches!(*rs_ls, Condition::Pure(Match {
        prop: Property::Name,
        op: Operator::Equal(s)
    }) if s == "Emacs"));
    assert!(matches!(*rs_rs, Condition::Pure(Match {
        prop: Property::Role,
        op: Operator::Equal(s)
    }) if s == "browser"));

    let (rest, value) =
        condition::<VerboseError<_>>(r#"class = "Firefox" && name = "Emacs" || role = "browser""#)
            .unwrap();
    assert_eq!(rest, "");
    let (ls, rs) = match value {
        Condition::Or(ls, rs) => (ls, rs),
        _ => panic!(),
    };
    let (ls_ls, ls_rs) = match *ls {
        Condition::And(ls, rs) => (ls, rs),
        _ => panic!(),
    };
    assert!(matches!(*ls_ls, Condition::Pure(Match {
        prop: Property::Class,
        op: Operator::Equal(s)
    }) if s == "Firefox"));
    assert!(matches!(*ls_rs, Condition::Pure(Match {
        prop: Property::Name,
        op: Operator::Equal(s)
    }) if s == "Emacs"));
    assert!(matches!(*rs, Condition::Pure(Match {
        prop: Property::Role,
        op: Operator::Equal(s)
    }) if s == "browser"));

    let (rest, value) = condition::<VerboseError<_>>(r#"(class = "Firefox")"#).unwrap();
    assert_eq!(rest, "");
    assert!(matches!(value, Condition::Pure(Match {
        prop: Property::Class,
        op: Operator::Equal(s)
    }) if s == "Firefox"));

    let (rest, value) = condition::<VerboseError<_>>(
        r#"(class = "Firefox" || name = "Emacs") && role = "browser""#,
    )
    .unwrap();
    assert_eq!(rest, "");
    let (ls, rs) = match value {
        Condition::And(ls, rs) => (ls, rs),
        _ => panic!(),
    };
    let (ls_ls, ls_rs) = match *ls {
        Condition::Or(ls, rs) => (ls, rs),
        _ => panic!(),
    };
    assert!(matches!(*ls_ls, Condition::Pure(Match {
        prop: Property::Class,
        op: Operator::Equal(s)
    }) if s == "Firefox"));
    assert!(matches!(*ls_rs, Condition::Pure(Match {
        prop: Property::Name,
        op: Operator::Equal(s)
    }) if s == "Emacs"));
    assert!(matches!(*rs, Condition::Pure(Match {
        prop: Property::Role,
        op: Operator::Equal(s)
    }) if s == "browser"));
}
