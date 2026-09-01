//! Operator CLI for loopback contextual-orchestrator interpretation-run cancel.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use orchestrator_live::{
    execute_interpretation_run_cancel_cli, read_interpretation_run_cancel_cli_stdin,
    render_interpretation_run_cancel_cli_stdout, InterpretationRunCancelCliInvocation,
    OrchestratorLiveError,
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
        Some("cancel") => run_cancel(&args),
        _ => Err(OrchestratorLiveError::InvalidWirePayload),
    }
}

fn run_cancel(args: &[String]) -> Result<(), OrchestratorLiveError> {
    let body = read_interpretation_run_cancel_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = InterpretationRunCancelCliInvocation::from_args(args, body)?;
    let response = execute_interpretation_run_cancel_cli(&invocation)?;
    let stdout = render_interpretation_run_cancel_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    Ok(())
}
