use nom::{IResult, space};
use regex::Regex;

use conditions::*;

named!(pub property<&str, Property>,
       alt_complete!(value!(Property::Class, tag_s!("class"))
               | value!(Property::Name, tag_s!("name"))
               | value!(Property::Role, tag_s!("role"))));

named!(escape<&str, &str>,
    preceded!(tag_s!("\\"),
        alt_complete!(tag_s!("\"")
                | tag_s!("\\"))));

named!(no_escapes<&str, &str>, is_not_s!("\\\""));

named!(string_content<&str, String>,
    map!(many0!(alt_complete!(no_escapes | escape)),
        |v: Vec<&str>| {
            let mut res = String::new();
            res.extend(v);
            res
        }));

named!(pub quoted_string<&str, String>,
    chain!(tag_s!("\"")
            ~ s: string_content
            ~ tag_s!("\""),
        || s));

named!(ws<&str, ()>, value!((), many0!(space)));

named!(pub match_<&str, Match>,
    chain!(p: property
            ~ ws
            ~ tag_s!("=")
            ~ ws
            ~ r: map_res!(quoted_string, |s: String| { Regex::new(&s) }),
        || Match { prop: p, pattern: r }));

named!(pub condition<&str, Condition>,
    chain!(l: cond_and
            ~ r: many0!(chain!(ws ~ tag_s!("||") ~ ws ~ c:cond_and, || c)),
        || r.into_iter().fold(l, |acc, x| Condition::Or(Box::new(acc), Box::new(x)))));

named!(cond_and<&str, Condition>,
    chain!(l: cond_not
            ~ r: many0!(chain!(ws ~ tag_s!("&&") ~ ws ~ c:cond_not, || c)),
        || r.into_iter().fold(l, |acc, x| Condition::And(Box::new(acc), Box::new(x)))));

named!(cond_not<&str, Condition>,
    chain!(nots: many0!(chain!(tag_s!("!") ~ ws, || ()))
            ~ c: cond_pure,
        || nots.into_iter().fold(c, |acc, _| Condition::Not(Box::new(acc)))));

// named!(cond_parens<&str, Condition>,
//     alt_complete!(chain!(tag_s!("(") ~ ws ~ c: condition ~ ws ~ tag_s!(")"), || c)
//             | cond_pure));

named!(cond_pure<&str, Condition>, map!(match_, Condition::Pure));

#[test]
fn test_property() {
    assert_eq!(property("class"), IResult::Done(&""[..], Property::Class));
    assert_eq!(property("name"), IResult::Done(&""[..], Property::Name));
    assert_eq!(property("role"), IResult::Done(&""[..], Property::Role));
}

#[test]
fn test_escape() {
    assert_eq!(escape("\\\""), IResult::Done(&""[..], "\""));
    assert_eq!(escape("\\\\"), IResult::Done(&""[..], "\\"));
}

#[test]
fn test_no_escapes() {
    assert_eq!(no_escapes("Hello \\"), IResult::Done("\\", "Hello "));
    assert_eq!(no_escapes("Hello \""), IResult::Done("\"", "Hello "));
}

#[test]
fn test_quoted_string() {
    assert_eq!(quoted_string("\"Hello World\""),
               IResult::Done(&""[..], "Hello World".to_owned()));
    assert_eq!(quoted_string(r#""Hello \"World\"""#),
               IResult::Done(&""[..], "Hello \"World\"".to_owned()));
}

#[test]
fn test_match_() {
    if let IResult::Done(_, m) = match_("class = \"Firefox\"") {
        assert_eq!(m.prop, Property::Class);
        assert!(m.pattern.is_match("Firefox"));
    } else {
        panic!();
    }
}

// #[test]
// fn test_cond_or() {
// let cond = condition("class = \"Firefox\" && name = \"Emacs\" && role = \"browser\"");
// println!("{:?}", cond);
// let cond = condition("( role = \"browser\" )");
// println!("{:?}", cond);
// }
