mod cli;
mod commands;
mod runner;

use std::process::ExitCode;

fn main() -> ExitCode {
    let matches = cli::build_command().get_matches();
    ExitCode::from(runner::run(&matches))
}
