//! Operator CLI for loopback analysis-run status GET.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use tepp_api::{
    AnalysisRunStatusCliInvocation, ApiError, execute_analysis_run_status_cli,
    read_analysis_run_status_cli_stdin, render_analysis_run_status_cli_stdout,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<(), ApiError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("status") => run_status(&args),
        _ => Err(ApiError::InvalidWirePayload),
    }
}

fn run_status(args: &[String]) -> Result<(), ApiError> {
    let body = read_analysis_run_status_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = AnalysisRunStatusCliInvocation::from_args(args, body)?;
    let response = execute_analysis_run_status_cli(&invocation)?;
    let stdout = render_analysis_run_status_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}
