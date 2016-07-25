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

fn print_usage(prog: &str) -> ! {
    display_error(Failure::new(&format!("{} CONDITION PROGRAM [ARGS...]", prog)).prefix("usage"));
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let app = &args[0];

    let (condition, prog, prog_args) = if args.len() >= 3 {
        (&args[1], &args[2], &args[3..])
    } else {
        print_usage(app);
    };

    let cond = match condition.parse() {
        Ok(cond) => cond,
        _ => print_usage(app),
    };

    let (conn, screen_num) = Connection::connect(None).unwrap_or_error("Cannot open display");
    let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();

    match windows::find_matching_window(&conn, &screen, &cond) {
        Ok(Some(win)) => {
            windows::set_active_window(&conn, &screen, win);
        }
        Ok(None) => exec_program(prog, prog_args),
        Err(_) => display_error("Could not access windows"),
    }
    conn.flush();
}
