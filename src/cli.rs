use clap::{App, AppSettings, Arg};

pub fn setup_command_line() -> App<'static, 'static> {
    App::new("run-or-raise")
        .version("0.1.0")
        .author("samuel.lauren@iki.fi")
        .about("Utility for launching applications or focusing their windows")
        .setting(AppSettings::TrailingVarArg)
        .arg(Arg::with_name("condition")
             .required(true)
             .value_name("CONDITION")
             .help("Condition to use for matching windows"))
        .arg(Arg::with_name("command")
             .required(true)
             .value_name("COMMAND")
             .help("Command to run if matching windows were not found"))
        .arg(Arg::with_name("args")
             .multiple(true)
             .help("Arguments to the COMMAND"))
}
