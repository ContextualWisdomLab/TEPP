//! Operator CLI for loopback contextual-orchestrator stored-request GET.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use orchestrator_live::{
    execute_interpretation_run_stored_request_cli, read_interpretation_run_stored_request_cli_stdin,
    render_interpretation_run_stored_request_cli_stdout,
    InterpretationRunStoredRequestCliInvocation, OrchestratorLiveError,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<(), OrchestratorLiveError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("get") => run_get(&args),
        _ => Err(OrchestratorLiveError::InvalidWirePayload),
    }
}

fn run_get(args: &[String]) -> Result<(), OrchestratorLiveError> {
    let body =
        read_interpretation_run_stored_request_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = InterpretationRunStoredRequestCliInvocation::from_args(args, body)?;
    let response = execute_interpretation_run_stored_request_cli(&invocation)?;
    let stdout = render_interpretation_run_stored_request_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    Ok(())
}
