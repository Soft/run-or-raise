extern crate xcb;
extern crate regex;
extern crate encoding;
extern crate termion;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate lazy_static;

mod parsing;
mod windows;
mod conditions;
mod utils;

use std::env;
use std::process::Command;
use std::os::unix::process::CommandExt;
use xcb::Connection;
use utils::{Failure, CouldFail, display_error};

fn exec_program(prog: &str, args: &[String]) -> ! {
    Command::new(prog).args(args).exec();
    display_error(&format!("Could not execute program \"{}\"", prog))
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let app = &args[0];

    let (condition, prog, prog_args) = if args.len() >= 3 {
        (&args[1], &args[2], &args[3..])
    } else {
        display_error(Failure::new(&format!("{} CONDITION PROGRAM [ARGS...]", app))
                          .prefix("usage"));
    };

    let cond = condition.parse().unwrap_or_error("Invalid condition");

    let (conn, screen_num) = Connection::connect(None).unwrap_or_error("Cannot open display");
    let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();

    match windows::find_matching_window(&conn, &screen, &cond)
              .unwrap_or_error("Could not access windows") {
        Some(win) => windows::set_active_window(&conn, &screen, win),
        None => exec_program(prog, prog_args),
    }
    conn.flush();
}
