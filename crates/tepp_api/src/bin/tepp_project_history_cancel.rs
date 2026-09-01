//! Operator CLI for loopback `LineageWeave` project-history cancel POST.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use tepp_api::{
    execute_project_history_cancel_cli, read_project_history_cancel_cli_stdin,
    render_project_history_cancel_cli_stdout, ApiError, ProjectHistoryCancelCliInvocation,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<(), ApiError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let body = read_project_history_cancel_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = ProjectHistoryCancelCliInvocation::from_args(&args, body)?;
    let response = execute_project_history_cancel_cli(&invocation)?;
    let stdout = render_project_history_cancel_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}
