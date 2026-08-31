//! Operator CLI for loopback analysis-run retry-lineage GET.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use tepp_api::{
    AnalysisRunRetryLineageCliInvocation, ApiError, execute_analysis_run_retry_lineage_cli,
    read_analysis_run_retry_lineage_cli_stdin, render_analysis_run_retry_lineage_cli_stdout,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<(), ApiError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let body = read_analysis_run_retry_lineage_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = AnalysisRunRetryLineageCliInvocation::from_args(&args, body)?;
    let response = execute_analysis_run_retry_lineage_cli(&invocation)?;
    let stdout = render_analysis_run_retry_lineage_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}
