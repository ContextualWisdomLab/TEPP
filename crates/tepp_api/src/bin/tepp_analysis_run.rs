//! Operator CLI for loopback scientific-acceptance analysis-run lifecycle.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use tepp_api::{
    AnalysisRunCliInvocation, AnalysisRunCliVerb, ApiError, execute_analysis_run_cli,
    read_analysis_run_cli_stdin, render_analysis_run_cli_stdout,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<(), ApiError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verb = AnalysisRunCliVerb::parse(args.first().ok_or(ApiError::InvalidWirePayload)?)?;
    let body = read_analysis_run_cli_stdin(verb, io::stdin().is_terminal(), io::stdin())?;
    let invocation = AnalysisRunCliInvocation::from_args(&args, body)?;
    let response = execute_analysis_run_cli(&invocation)?;
    let stdout = render_analysis_run_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}
