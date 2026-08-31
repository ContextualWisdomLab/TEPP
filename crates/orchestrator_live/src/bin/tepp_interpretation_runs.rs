//! Operator CLI for loopback contextual-orchestrator interpretation-run POST.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use orchestrator_live::{
    InterpretationRunCliInvocation, OrchestratorLiveError, execute_interpretation_run_cli,
    read_interpretation_run_cli_stdin, render_interpretation_run_cli_stdout,
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
        Some("create") => run_create(&args),
        _ => Err(OrchestratorLiveError::InvalidWirePayload),
    }
}

fn run_create(args: &[String]) -> Result<(), OrchestratorLiveError> {
    let body = read_interpretation_run_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = InterpretationRunCliInvocation::from_args(args, body)?;
    let response = execute_interpretation_run_cli(&invocation)?;
    let stdout = render_interpretation_run_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(OrchestratorLiveError::InvalidWirePayload)
    }
}
