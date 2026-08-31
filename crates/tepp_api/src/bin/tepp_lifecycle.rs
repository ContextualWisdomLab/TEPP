//! Operator CLI for loopback analysis-run running and terminal POST.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use tepp_api::{
    AnalysisRunLifecycleCliInvocation, ApiError, execute_analysis_run_lifecycle_cli,
    read_analysis_run_lifecycle_cli_stdin, render_analysis_run_lifecycle_cli_stdout,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<(), ApiError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let body = read_analysis_run_lifecycle_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = AnalysisRunLifecycleCliInvocation::from_args(&args, body)?;
    let response = execute_analysis_run_lifecycle_cli(&invocation)?;
    let stdout = render_analysis_run_lifecycle_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}
