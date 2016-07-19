extern crate xcb;
extern crate regex;
extern crate encoding;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate lazy_static;

mod parsing;
mod windows;
mod conditions;

use std::env;
use std::process::Command;
use std::os::unix::process::CommandExt;
use xcb::Connection;
use nom::IResult;

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

    let cond = match parsing::condition(pattern) {
        IResult::Done("", a) => a,
        _ => print_usage(&args[0]),
    };

    let (conn, screen_num) = Connection::connect(None).expect("Cannot open display");
    let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();

    match windows::find_matching_window(&conn, &screen, &cond) {
        Ok(Some(win)) => {
            windows::set_active_window(&conn, &screen, win);
        }
        Ok(None) => exec_program(prog, prog_args),
        Err(_) => panic!(),
    }
    conn.flush();
}
