//! Operator CLI for loopback `LineageWeave` project-history collection GET.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use tepp_api::{
    ApiError, ProjectHistoryCollectionCliInvocation, execute_project_history_collection_cli,
    read_project_history_collection_cli_stdin, render_project_history_collection_cli_stdout,
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
    let body = read_project_history_collection_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = ProjectHistoryCollectionCliInvocation::from_args(&args, body)?;
    let response = execute_project_history_collection_cli(&invocation)?;
    let stdout = render_project_history_collection_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    Ok(())
}
