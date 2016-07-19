extern crate xcb;
extern crate regex;
extern crate encoding;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nom;

use std::env;
use std::collections::HashMap;
use std::sync::Mutex;
use std::process::Command;
use std::os::unix::process::CommandExt;
use xcb::{Connection, Window, Atom, GenericError, ClientMessageEvent, ClientMessageData};
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;
use regex::Regex;
use nom::IResult;

const XCB_EWMH_CLIENT_SOURCE_TYPE_OTHER: u32 = 2;

lazy_static! {
    static ref INTERNED_ATOMS: Mutex<HashMap<&'static str, Atom>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

struct WindowTreeIter<'a> {
    conn: &'a Connection,
    stack: Vec<Window>,
}

impl<'a> WindowTreeIter<'a> {
    fn new(conn: &'a Connection, win: Window) -> Result<WindowTreeIter<'a>, GenericError> {
        let reply = try!(xcb::query_tree(conn, win).get_reply());
        Ok(WindowTreeIter {
            conn: conn,
            stack: reply.children().to_owned(),
        })
    }
}

impl<'a> Iterator for WindowTreeIter<'a> {
    type Item = Result<Window, GenericError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|top| {
            xcb::query_tree(self.conn, top).get_reply().map(|reply| {
                self.stack.extend(reply.children());
                top
            })
        })
    }
}

fn get_atom(conn: &Connection, atom: &'static str) -> Atom {
    let current = {
        INTERNED_ATOMS.lock().unwrap().get(atom).cloned()
    };
    current.unwrap_or_else(|| {
        let interned = xcb::intern_atom(conn, true, atom).get_reply().unwrap().atom();
        INTERNED_ATOMS.lock().unwrap().insert(atom, interned);
        interned
    })
}

fn set_active_window(conn: &Connection, screen: &xcb::Screen, win: Window) {
    let net_active_window = get_atom(conn, "_NET_ACTIVE_WINDOW");
    let data = ClientMessageData::from_data32([XCB_EWMH_CLIENT_SOURCE_TYPE_OTHER,
                                               xcb::CURRENT_TIME,
                                               xcb::WINDOW_NONE,
                                               0,
                                               0]);
    let ev = ClientMessageEvent::new(32, win, net_active_window, data);
    xcb::send_event(conn,
                    false,
                    screen.root(),
                    xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
                    &ev);
}

fn get_string_property(conn: &Connection, window: Window, prop: Atom) -> Option<String> {
    let reply = match xcb::get_property(conn,
                                        false,
                                        window,
                                        prop,
                                        xcb::GET_PROPERTY_TYPE_ANY,
                                        0,
                                        u32::max_value())
                          .get_reply() {
        Ok(r) => r,
        _ => return None,
    };
    let atom_utf8_string = get_atom(conn, "UTF8_STRING");
    let reply_type = reply.type_();
    if reply_type == xcb::ATOM_STRING {
        ISO_8859_1.decode(reply.value(), DecoderTrap::Strict).ok()
    } else if reply_type == atom_utf8_string {
        String::from_utf8(reply.value().to_vec()).ok()
    } else {
        None
    }
}

fn is_regular_window(conn: &Connection, window: Window) -> bool {
    let atom_wm_state = get_atom(conn, "WM_STATE");
    xcb::get_property(conn,
                      false,
                      window,
                      atom_wm_state,
                      atom_wm_state,
                      0,
                      u32::max_value())
        .get_reply()
        .map(|state| state.value_len() > 0)
        .unwrap_or(false)
}

#[derive(Debug,PartialEq)]
enum Property {
    Class,
    Name,
    Role,
}

impl Property {
    fn from_window(&self, conn: &Connection, win: Window) -> Option<String> {
        match *self {
            Property::Class => {
                get_string_property(conn, win, xcb::ATOM_WM_CLASS)
                    .map(|p| p.split('\u{0}').nth(1).unwrap().to_owned())
            }
            Property::Name => {
                get_string_property(conn, win, get_atom(conn, "_NET_WM_NAME"))
                    .or(get_string_property(conn, win, xcb::ATOM_WM_NAME))
            }
            Property::Role => get_string_property(conn, win, get_atom(conn, "WM_WINDOW_ROLE")),
        }
    }
}

#[derive(Debug)]
struct Match {
    prop: Property,
    pattern: Regex,
}

impl Match {
    fn matches(&self, conn: &Connection, win: Window) -> bool {
        self.prop.from_window(conn, win).map(|p| self.pattern.is_match(&p)).unwrap_or(false)
    }
}

#[derive(Debug)]
enum Condition {
    Pure(Match),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}

// TODO: Avoid multiple lookups
impl Condition {
    fn matches(&self, conn: &Connection, win: Window) -> bool {
        match *self {
            Condition::Pure(ref m) => m.matches(conn, win),
            Condition::And(ref a, ref b) => a.matches(conn, win) && b.matches(conn, win),
            Condition::Or(ref a, ref b) => a.matches(conn, win) || b.matches(conn, win),
            Condition::Not(ref a) => !a.matches(conn, win),
        }
    }
}

named!(property<&str, Property>,
       alt_complete!(value!(Property::Class, tag_s!("class"))
               | value!(Property::Name, tag_s!("name"))
               | value!(Property::Role, tag_s!("role"))));

#[test]
fn test_property() {
    assert_eq!(property("class"), IResult::Done(&""[..], Property::Class));
    assert_eq!(property("name"), IResult::Done(&""[..], Property::Name));
    assert_eq!(property("role"), IResult::Done(&""[..], Property::Role));
}

named!(escape<&str, &str>,
    preceded!(tag_s!("\\"),
        alt_complete!(tag_s!("\"")
                | tag_s!("\\"))));

#[test]
fn test_escape() {
    assert_eq!(escape("\\\""), IResult::Done(&""[..], "\""));
    assert_eq!(escape("\\\\"), IResult::Done(&""[..], "\\"));
}

named!(no_escapes<&str, &str>, is_not_s!("\\\""));

#[test]
fn test_no_escapes() {
    assert_eq!(no_escapes("Hello \\"), IResult::Done("\\", "Hello "));
    assert_eq!(no_escapes("Hello \""), IResult::Done("\"", "Hello "));
}

named!(string_content<&str, String>,
    map!(many0!(alt_complete!(no_escapes | escape)),
        |v: Vec<&str>| {
            let mut res = String::new();
            res.extend(v);
            res
        }));

named!(quoted_string<&str, String>,
    chain!(tag_s!("\"")
            ~ s: string_content
            ~ tag_s!("\""),
        || s));

#[test]
fn test_quoted_string() {
    assert_eq!(quoted_string("\"Hello World\""),
               IResult::Done(&""[..], "Hello World".to_owned()));
    assert_eq!(quoted_string(r#""Hello \"World\"""#),
               IResult::Done(&""[..], "Hello \"World\"".to_owned()));
}

named!(ws<&str, ()>, value!((), many0!(nom::space)));

named!(match_<&str, Match>,
    chain!(p: property
            ~ ws
            ~ tag_s!("=")
            ~ ws
            ~ r: map_res!(quoted_string, |s: String| { Regex::new(&s) }),
        || Match { prop: p, pattern: r }));

#[test]
fn test_match_() {
    if let IResult::Done(_, m) = match_("class = \"Firefox\"") {
        assert_eq!(m.prop, Property::Class);
        assert!(m.pattern.is_match("Firefox"));
    } else {
        panic!();
    }
}

named!(condition<&str, Condition>,
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

// #[test]
// fn test_cond_or() {
// let cond = condition("class = \"Firefox\" && name = \"Emacs\" && role = \"browser\"");
// println!("{:?}", cond);
// let cond = condition("( role = \"browser\" )");
// println!("{:?}", cond);
// }

fn find_matching_window(conn: &Connection,
                        screen: &xcb::Screen,
                        cond: &Condition)
                        -> Result<Option<Window>, GenericError> {
    let wins = try!(WindowTreeIter::new(&conn, screen.root()));
    for w in wins {
        let w = try!(w);
        if is_regular_window(conn, w) && cond.matches(conn, w) {
            return Ok(Some(w));
        }
    }
    Ok(None)
}

fn exec_program(prog: &str, args: &[String]) -> ! {
    Command::new(prog).args(args).exec();
    panic!()
}

fn print_usage(prog: &str) -> ! {
    println!("{} PATTERN PROGRAM [ARGS...]", prog);
    std::process::exit(1);
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let (pattern, prog, prog_args) = if args.len() >= 3 {
        (&args[1], &args[2], &args[3..])
    } else {
        print_usage(&args[0]);
    };

    let cond = match condition(pattern) {
        IResult::Done("", a) => a,
        _ => print_usage(&args[0]),
    };

    let (conn, screen_num) = Connection::connect(None).expect("Cannot open display");
    let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();

    match find_matching_window(&conn, &screen, &cond) {
        Ok(Some(win)) => {
            set_active_window(&conn, &screen, win);
        }
        Ok(None) => exec_program(prog, prog_args),
        Err(_) => panic!(),
    }
    conn.flush();
}
