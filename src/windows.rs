use crate::conditions::Condition;
use anyhow::{anyhow, Error, Result};
use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};
use lazy_static::*;
use std::collections::HashMap;
use std::sync::Mutex;
use xcb::Xid;
use xcb::{
    self,
    x::{self, Atom, Screen, Window},
    Connection,
};

const XCB_EWMH_CLIENT_SOURCE_TYPE_OTHER: u32 = 2;

lazy_static! {
    static ref INTERNED_ATOMS: Mutex<HashMap<&'static [u8], Atom>> = Mutex::new(HashMap::new());
}

pub struct WindowTreeIter<'a> {
    pub conn: &'a Connection,
    pub stack: Vec<Window>,
}

impl<'a> WindowTreeIter<'a> {
    fn new(conn: &'a Connection, win: Window) -> Result<WindowTreeIter<'a>, xcb::Error> {
        let cookie = conn.send_request(&x::QueryTree { window: win });
        let reply = conn.wait_for_reply(cookie)?;
        Ok(WindowTreeIter {
            conn,
            stack: reply.children().to_owned(),
        })
    }
}

impl<'a> Iterator for WindowTreeIter<'a> {
    type Item = Result<Window, xcb::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|top| {
            let cookie = self.conn.send_request(&x::QueryTree { window: top });
            self.conn.wait_for_reply(cookie).map(|reply| {
                self.stack.extend(reply.children());
                top
            })
        })
    }
}

pub fn get_atom(conn: &Connection, atom: &'static [u8]) -> Result<Atom> {
    fn err<E>(_: E) -> Error {
        anyhow!("Failed to access atom map")
    }
    let current = { INTERNED_ATOMS.lock().map_err(err)?.get(atom).cloned() };
    match current {
        Some(current) => Ok(current),
        None => {
            let cookie = conn.send_request(&x::InternAtom {
                only_if_exists: false,
                name: atom,
            });
            let interned = conn.wait_for_reply(cookie)?.atom();
            INTERNED_ATOMS.lock().map_err(err)?.insert(atom, interned);
            Ok(interned)
        }
    }
}

pub fn set_active_window(conn: &Connection, screen: &Screen, win: Window) -> Result<(), Error> {
    let net_active_window = get_atom(conn, b"_NET_ACTIVE_WINDOW")?;
    let data = x::ClientMessageData::Data32([
        XCB_EWMH_CLIENT_SOURCE_TYPE_OTHER,
        x::CURRENT_TIME,
        x::WINDOW_NONE.resource_id(),
        0,
        0,
    ]);
    let event = x::ClientMessageEvent::new(win, net_active_window, data);
    conn.send_and_check_request(&x::SendEvent {
        propagate: false,
        destination: x::SendEventDest::Window(screen.root()),
        event_mask: x::EventMask::SUBSTRUCTURE_NOTIFY | x::EventMask::SUBSTRUCTURE_REDIRECT,
        event: &event,
    })
    .map_err(Into::into)
}

pub fn get_string_property(
    conn: &Connection,
    window: Window,
    property: Atom,
) -> Result<Option<String>, Error> {
    let cookie = conn.send_request(&x::GetProperty {
        delete: false,
        window,
        property,
        r#type: x::GETPROPERTYTYPE_ANY,
        long_offset: 0,
        long_length: u32::max_value(),
    });
    let reply = match conn.wait_for_reply(cookie) {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };
    let atom_utf8_string = get_atom(conn, b"UTF8_STRING")?;
    let reply_type = reply.r#type();
    if reply_type == x::ATOM_STRING {
        Ok(ISO_8859_1.decode(reply.value(), DecoderTrap::Strict).ok())
    } else if reply_type == atom_utf8_string {
        Ok(String::from_utf8(reply.value().to_vec()).ok())
    } else {
        Ok(None)
    }
}

pub fn is_regular_window(conn: &Connection, window: Window) -> Result<bool, Error> {
    let atom_wm_state = get_atom(conn, b"WM_STATE")?;
    let cookie = conn.send_request(&x::GetProperty {
        delete: false,
        window,
        property: atom_wm_state,
        r#type: atom_wm_state,
        long_offset: 0,
        long_length: u32::max_value(),
    });
    Ok(conn
        .wait_for_reply(cookie)
        .map(|state| !state.value::<u32>().is_empty())
        .unwrap_or(false))
}

pub fn find_matching_window(
    conn: &Connection,
    screen: &Screen,
    cond: &Condition,
) -> Result<Option<Window>, Error> {
    let windows = WindowTreeIter::new(conn, screen.root())?;
    for window in windows {
        let window = window?;
        if is_regular_window(conn, window)? && cond.matches(conn, window)? {
            return Ok(Some(window));
        }
    }
    Ok(None)
}
