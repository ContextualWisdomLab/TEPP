//! Operator CLI for loopback analysis-run collection GET, cancel POST, and create POST.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use tepp_api::{
    AnalysisRunCancelCliInvocation, AnalysisRunCollectionCliInvocation,
    AnalysisRunCollectionCliVerb, AnalysisRunCreateCliInvocation, ApiError,
    execute_analysis_run_cancel_cli, execute_analysis_run_collection_cli,
    execute_analysis_run_create_cli, read_analysis_run_cancel_cli_stdin,
    read_analysis_run_collection_cli_stdin, read_analysis_run_create_cli_stdin,
    render_analysis_run_cancel_cli_stdout, render_analysis_run_collection_cli_stdout,
    render_analysis_run_create_cli_stdout,
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
        Some("list") => run_list(&args),
        Some("cancel") => run_cancel(&args),
        Some("create") => run_create(&args),
        _ => Err(ApiError::InvalidWirePayload),
    }
}

fn run_list(args: &[String]) -> Result<(), ApiError> {
    let verb =
        AnalysisRunCollectionCliVerb::parse(args.first().ok_or(ApiError::InvalidWirePayload)?)?;
    let body = read_analysis_run_collection_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = AnalysisRunCollectionCliInvocation::from_args(args, body)?;
    if invocation.verb != verb {
        return Err(ApiError::InvalidWirePayload);
    }
    let response = execute_analysis_run_collection_cli(&invocation)?;
    let stdout = render_analysis_run_collection_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

fn run_cancel(args: &[String]) -> Result<(), ApiError> {
    let body = read_analysis_run_cancel_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = AnalysisRunCancelCliInvocation::from_args(args, body)?;
    let response = execute_analysis_run_cancel_cli(&invocation)?;
    let stdout = render_analysis_run_cancel_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

fn run_create(args: &[String]) -> Result<(), ApiError> {
    let body = read_analysis_run_create_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = AnalysisRunCreateCliInvocation::from_args(args, body)?;
    let response = execute_analysis_run_create_cli(&invocation)?;
    let stdout = render_analysis_run_create_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}
