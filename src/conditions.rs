use anyhow::{anyhow, Result};
use regex::Regex;
use xcb::{
    x::{self, Window},
    Connection,
};

use crate::windows::{get_atom, get_string_property};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Property {
    Class,
    Name,
    Role,
}

#[derive(Debug)]
pub struct Match {
    pub prop: Property,
    pub op: Operator,
}

#[derive(Debug)]
pub enum Operator {
    Regex(Regex),
    Equal(String),
}

#[derive(Debug)]
pub enum Condition {
    Pure(Match),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}

impl Property {
    pub fn from_window(&self, conn: &Connection, win: Window) -> Result<Option<String>> {
        match *self {
            Property::Class => {
                get_string_property(conn, win, x::ATOM_WM_CLASS)?.map_or(Ok(None), |p| {
                    p.split('\u{0}')
                        .nth(1)
                        .ok_or_else(|| anyhow!("Invalid class defintion"))
                        .map(|s| Some(s.to_owned()))
                })
            }
            Property::Name => get_string_property(conn, win, get_atom(conn, b"_NET_WM_NAME")?)?
                .map_or_else(
                    || get_string_property(conn, win, x::ATOM_WM_NAME),
                    |v| Ok(Some(v)),
                ),
            Property::Role => get_string_property(conn, win, get_atom(conn, b"WM_WINDOW_ROLE")?),
        }
    }
}

impl Match {
    pub fn matches(&self, conn: &Connection, win: Window) -> Result<bool> {
        Ok(self
            .prop
            .from_window(conn, win)?
            .map(|p| match self.op {
                Operator::Regex(ref pattern) => pattern.is_match(&p),
                Operator::Equal(ref value) => value == &p,
            })
            .unwrap_or(false))
    }
}

impl Condition {
    pub fn matches(&self, conn: &Connection, win: x::Window) -> Result<bool> {
        Ok(match *self {
            Condition::Pure(ref m) => m.matches(conn, win)?,
            Condition::And(ref a, ref b) => a.matches(conn, win)? && b.matches(conn, win)?,
            Condition::Or(ref a, ref b) => a.matches(conn, win)? || b.matches(conn, win)?,
            Condition::Not(ref a) => !a.matches(conn, win)?,
        })
    }
}
