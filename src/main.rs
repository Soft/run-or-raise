mod conditions;
mod parsing;
mod windows;

use anyhow::{bail, Error, Result};
use std::env;
use std::os::unix::process::CommandExt;
use std::process;
use std::process::Command;
use xcb::Connection;

fn exec_program(prog: &str, args: &[String]) -> Error {
    let error = Command::new(prog).args(args).exec();
    Error::new(error).context("Executing program failed")
}

fn run() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let app = &args[0];

    let (condition, prog, prog_args) = if args.len() >= 3 {
        (&args[1], &args[2], &args[3..])
    } else {
        bail!("{} CONDITION PROGRAM [ARGS...]", app);
    };

    let cond = condition.parse()?;

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
    if let Err(err) = run() {
        eprintln!("{}: {}", env!("CARGO_BIN_NAME"), err);
        err.chain()
            .skip(1)
            .for_each(|cause| eprintln!("caused by:\n{}", cause));
        process::exit(1);
    }
}
