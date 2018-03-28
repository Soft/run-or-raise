extern crate xcb;
extern crate regex;
extern crate encoding;
extern crate termion;
extern crate failure;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate lazy_static;

mod parsing;
mod windows;
mod conditions;

use std::env;
use std::process;
use std::process::Command;
use std::os::unix::process::CommandExt;
use xcb::Connection;
use failure::{Error, err_msg};

fn exec_program(prog: &str, args: &[String]) -> Error {
    let error = Command::new(prog).args(args).exec();
    err_msg(format!("Could not execute program \"{}\": {}", prog, error))
}

fn run() -> Result<(), Error> {
    let args: Vec<_> = env::args().collect();
    let app = &args[0];

    let (condition, prog, prog_args) = if args.len() >= 3 {
        (&args[1], &args[2], &args[3..])
    } else {
        return Err(err_msg(format!("{} CONDITION PROGRAM [ARGS...]", app)));
    };

    let cond = condition.parse()
        .map_err(|_| err_msg("Invalid condition"))?;

    let (conn, screen_num) = Connection::connect(None)?;
    let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();

    match windows::find_matching_window(&conn, &screen, &cond)? {
        Some(win) => windows::set_active_window(&conn, &screen, win)?,
        None => return Err(exec_program(prog, prog_args)),
    }
    conn.flush();

    Ok(())
}

fn main() {
    use termion::{color, style};

    if let Err(err) = run() {
        let message = format!("{}{}error:{} {}",
                              style::Bold,
                              color::Fg(color::Red),
                              style::Reset,
                              err);
        eprintln!("{}", message);
        process::exit(1);
    }

}
