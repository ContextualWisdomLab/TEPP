//! Operator CLI for loopback naruon export idempotency-key lookup GET.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use tepp_api::{
    execute_export_idempotency_lookup_cli, read_export_idempotency_lookup_cli_stdin,
    render_export_idempotency_lookup_cli_stdout, ApiError, ExportIdempotencyLookupCliInvocation,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tepp-export-lookup: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), ApiError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let body = read_export_idempotency_lookup_cli_stdin(io::stdin().is_terminal(), io::stdin())?;
    let invocation = ExportIdempotencyLookupCliInvocation::from_args(&args, body)?;
    let response = execute_export_idempotency_lookup_cli(&invocation)?;
    let stdout = render_export_idempotency_lookup_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}
