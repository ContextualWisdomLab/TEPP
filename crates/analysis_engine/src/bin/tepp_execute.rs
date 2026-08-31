//! Operator CLI for POST /execute through typed naruon/`LineageWeave` exchanges.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use analysis_engine::{
    ScientificAcceptanceExecuteCliInvocation, execute_scientific_acceptance_execute_cli,
    read_scientific_acceptance_execute_cli_stdin, render_scientific_acceptance_execute_cli_stdout,
};
use tepp_api::ApiError;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<(), ApiError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let body =
        read_scientific_acceptance_execute_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = ScientificAcceptanceExecuteCliInvocation::from_args(&args, body)?;
    let response = execute_scientific_acceptance_execute_cli(&invocation)?;
    let stdout = render_scientific_acceptance_execute_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}
