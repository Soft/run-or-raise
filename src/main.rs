extern crate xcb;
extern crate regex;
extern crate encoding;
extern crate termion;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;

mod parsing;
mod windows;
mod conditions;
mod utils;
mod cli;

use std::process::Command;
use std::os::unix::process::CommandExt;
use xcb::Connection;
use utils::{CouldFail, display_error};

fn exec_program(prog: &str, args: &[&str]) -> ! {
    Command::new(prog).args(args).exec();
    display_error(&format!("Could not execute program \"{}\"", prog))
}

fn main() {
    let matches = cli::setup_command_line().get_matches();

    let condition = matches.value_of("condition").unwrap();
    let prog = matches.value_of("command").unwrap();
    let prog_args = matches.values_of("args")
        .map(|v| v.collect())
        .unwrap_or(vec![]);

    let cond = condition.parse().unwrap_or_error("Invalid condition");

    let (conn, screen_num) = Connection::connect(None).unwrap_or_error("Cannot open display");
    let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();

    match windows::find_matching_window(&conn, &screen, &cond)
              .unwrap_or_error("Could not access windows") {
        Some(win) => windows::set_active_window(&conn, &screen, win),
        None => exec_program(prog, &prog_args as &[&str]),
    }
    conn.flush();
}
