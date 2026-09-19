//! Operator CLI for loopback contextual-orchestrator lookup GET.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use orchestrator_live::{
    InterpretationRunLookupCliInvocation, OrchestratorLiveError,
    execute_interpretation_run_lookup_cli, read_interpretation_run_lookup_cli_stdin,
    render_interpretation_run_lookup_cli_stdout,
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
        Some("lookup") => run_lookup(&args),
        _ => Err(OrchestratorLiveError::InvalidWirePayload),
    }
}

fn run_lookup(args: &[String]) -> Result<(), OrchestratorLiveError> {
    let body = read_interpretation_run_lookup_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = InterpretationRunLookupCliInvocation::from_args(args, body)?;
    let response = execute_interpretation_run_lookup_cli(&invocation)?;
    let stdout = render_interpretation_run_lookup_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    Ok(())
}
