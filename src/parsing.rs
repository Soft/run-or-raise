use nom::{IError, space};
use std::str::FromStr;
use regex::Regex;

use conditions::*;

named!(property<&str, Property>,
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

named!(quoted_string<&str, String>,
       do_parse!(tag_s!("\"") >>
                 s: string_content >>
                 tag_s!("\"") >>
                 (s)));

named!(ws<&str, ()>, value!((), many0!(space)));

named!(op_equal<&str, Operator>,
       do_parse!(tag_s!("=") >>
                 ws >>
                 s: quoted_string >>
                 (Operator::Equal(s))));

named!(op_regex<&str, Operator>,
       do_parse!(tag_s!("~") >>
                 ws >>
                 r: map_res!(quoted_string, |s: String| Regex::new(&s)) >>
                 (Operator::Regex(r))));

named!(match_<&str, Match>,
       do_parse!(p: property >>
                 ws >>
                 op: alt_complete!(op_equal | op_regex) >>
                 (Match { prop: p, op: op })));

named!(condition<&str, Condition>,
       do_parse!(l: cond_and >>
                 r: many0!(do_parse!(ws >>
                                     tag_s!("||") >>
                                     ws >>
                                     c:cond_and >>
                                     (c))) >>
                 (r.into_iter().fold(l, |acc, x| Condition::Or(Box::new(acc), Box::new(x))))));

named!(cond_and<&str, Condition>,
       do_parse!(l: cond_not >>
                 r: many0!(do_parse!(ws >>
                                     tag_s!("&&") >>
                                     ws >>
                                     c:cond_not >>
                                     (c))) >>
                 (r.into_iter().fold(l, |acc, x| Condition::And(Box::new(acc), Box::new(x))))));

named!(cond_not<&str, Condition>,
       do_parse!(nots: many0!(do_parse!(tag_s!("!") >>
                                        ws >>
                                        ())) >>
                 c: cond_parens >>
                 (nots.into_iter().fold(c, |acc, _| Condition::Not(Box::new(acc))))));

named!(cond_parens<&str, Condition>,
       alt_complete!(do_parse!(tag_s!("(") >>
                               ws >>
                               c: condition >>
                               ws >>
                               tag_s!(")") >>
                               (c))
                     | cond_pure));

named!(cond_pure<&str, Condition>, map!(match_, Condition::Pure));

impl FromStr for Condition {
    type Err = IError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        condition(s).to_full_result()
    }
}

#[test]
fn test_property() {
    use nom::IResult::Done;
    assert_eq!(property("class"), Done(&""[..], Property::Class));
    assert_eq!(property("name"), Done(&""[..], Property::Name));
    assert_eq!(property("role"), Done(&""[..], Property::Role));
}

#[test]
fn test_escape() {
    use nom::IResult::Done;
    assert_eq!(escape("\\\""), Done(&""[..], "\""));
    assert_eq!(escape("\\\\"), Done(&""[..], "\\"));
}

#[test]
fn test_no_escapes() {
    use nom::IResult::Done;
    assert_eq!(no_escapes("Hello \\"), Done("\\", "Hello "));
    assert_eq!(no_escapes("Hello \""), Done("\"", "Hello "));
}

#[test]
fn test_quoted_string() {
    use nom::IResult::Done;
    assert_eq!(quoted_string("\"Hello World\""),
               Done(&""[..], "Hello World".to_owned()));
    assert_eq!(quoted_string(r#""Hello \"World\"""#),
               Done(&""[..], "Hello \"World\"".to_owned()));
}

#[test]
fn test_match_() {
    use nom::IResult::Done;
    if let Done(_, m) = match_("class ~ \"Firefox\"") {
        assert_eq!(m.prop, Property::Class);
        if let Operator::Regex(ref p) = m.op {
            assert!(p.is_match("Firefox"));
        } else {
            panic!();
        }
    } else {
        panic!();
    }
}

#[test]
fn test_cond_or() {
    let cond = condition("class = \"Firefox\" && name = \"Emacs\" && role = \"browser\"");
    println!("{:#?}", cond);
    assert!(cond.is_done());
    let cond = condition("class = \"Firefox\" && (name = \"Emacs\" && role = \"browser\")");
    println!("{:#?}", cond);
    assert!(cond.is_done());
    let cond = condition("( role = \"browser\" )");
    println!("{:#?}", cond);
    assert!(cond.is_done());
}
