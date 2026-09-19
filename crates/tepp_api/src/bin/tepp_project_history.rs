//! Operator CLI for loopback `LineageWeave` project-history POST.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use tepp_api::{
    ApiError, ProjectHistoryCliInvocation, execute_project_history_cli,
    read_project_history_cli_stdin, render_project_history_cli_stdout,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), ApiError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("query") => run_query(&args),
        _ => Err(ApiError::InvalidWirePayload),
    }
}

fn run_query(args: &[String]) -> Result<(), ApiError> {
    let body = read_project_history_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = ProjectHistoryCliInvocation::from_args(args, body)?;
    let response = execute_project_history_cli(&invocation)?;
    let stdout = render_project_history_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    Ok(())
}
