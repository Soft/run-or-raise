use std::io::{self, Write};
use std::result::Result;
use std::option::Option;
use std::process;
use termion::{color, style};

pub struct Failure<'a, 'm> {
    code: i32,
    prefix: Option<&'m str>,
    message: &'a str,
}

impl<'a, 'm> Failure<'a, 'm> {
    pub fn new(msg: &'a str) -> Failure<'a, 'static> {
        Failure {
            code: 1,
            prefix: Some("error"),
            message: msg,
        }
    }

    pub fn prefix(mut self, p: &'m str) -> Failure<'a, 'm> {
        self.prefix = Some(p);
        self
    }

    pub fn code(mut self, code: i32) -> Failure<'a, 'm> {
        self.code = code;
        self
    }
}

impl<'a, T: ?Sized> From<&'a T> for Failure<'a, 'static> where T: AsRef<str> {
    fn from(msg: &'a T) -> Self {
        Failure::new(msg.as_ref())
    }
}

pub fn display_error<'a, 'm, T>(msg: T) -> !
    where T: Into<Failure<'a, 'm>>
{
    let msg = msg.into();
    let prefix = match msg.prefix {
        Some(ref p) => {
            format!("{}{}{}:{} ",
                    style::Bold,
                    color::Fg(color::Red),
                    p,
                    style::Reset)
        }
        _ => "".to_owned(),
    };
    writeln!(io::stderr(), "{}{}", prefix, msg.message).unwrap();
    process::exit(msg.code);
}

pub trait CouldFail<V> {
    fn unwrap_or_error<'a, 'm, T>(self, T) -> V where T: Into<Failure<'a, 'm>>;
}

impl<V, E> CouldFail<V> for Result<V, E> {
    fn unwrap_or_error<'a, 'm, T>(self, msg: T) -> V
        where T: Into<Failure<'a, 'm>>
    {
        match self {
            Ok(v) => v,
            _ => display_error(msg),
        }
    }
}

impl<V> CouldFail<V> for Option<V> {
    fn unwrap_or_error<'a, 'm, T>(self, msg: T) -> V
        where T: Into<Failure<'a, 'm>>
    {
        match self {
            Some(v) => v,
            _ => display_error(msg),
        }
    }
}
