//! Operator CLI for loopback `LineageWeave` temporal-context GET-by-id.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use tepp_api::{
    execute_temporal_context_retrieval_cli, read_temporal_context_retrieval_cli_stdin,
    render_temporal_context_retrieval_cli_stdout, ApiError, TemporalContextRetrievalCliInvocation,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<(), ApiError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let body = read_temporal_context_retrieval_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = TemporalContextRetrievalCliInvocation::from_args(&args, body)?;
    let response = execute_temporal_context_retrieval_cli(&invocation)?;
    let stdout = render_temporal_context_retrieval_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if response.status_code == 200 {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}
