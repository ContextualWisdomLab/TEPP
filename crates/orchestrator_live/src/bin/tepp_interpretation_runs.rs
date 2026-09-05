//! Operator CLI for loopback contextual-orchestrator interpretation-run POST and collection GET.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use orchestrator_live::{
    execute_interpretation_run_cli, execute_interpretation_run_collection_cli,
    read_interpretation_run_cli_stdin, read_interpretation_run_collection_cli_stdin,
    render_interpretation_run_cli_stdout, render_interpretation_run_collection_cli_stdout,
    InterpretationRunCliInvocation, InterpretationRunCollectionCliInvocation,
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
        Some("create") => run_create(&args),
        Some("list") => run_list(&args),
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

fn run_list(args: &[String]) -> Result<(), OrchestratorLiveError> {
    let body =
        read_interpretation_run_collection_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = InterpretationRunCollectionCliInvocation::from_args(args, body)?;
    let response = execute_interpretation_run_collection_cli(&invocation)?;
    let stdout = render_interpretation_run_collection_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    Ok(())
}
