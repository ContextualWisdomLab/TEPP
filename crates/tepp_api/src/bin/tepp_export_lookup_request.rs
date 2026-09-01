//! Operator CLI for loopback naruon export lookup stored-request GET.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use tepp_api::{
    ApiError, ExportIdempotencyLookupStoredRequestCliInvocation,
    execute_export_idempotency_lookup_stored_request_cli,
    read_export_idempotency_lookup_stored_request_cli_stdin,
    render_export_idempotency_lookup_stored_request_cli_stdout,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tepp-export-lookup-request: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), ApiError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let body = read_export_idempotency_lookup_stored_request_cli_stdin(
        io::stdin().is_terminal(),
        io::stdin(),
    )?;
    let invocation = ExportIdempotencyLookupStoredRequestCliInvocation::from_args(&args, body)?;
    let response = execute_export_idempotency_lookup_stored_request_cli(&invocation)?;
    let stdout =
        render_export_idempotency_lookup_stored_request_cli_stdout(&invocation, &response)?;
    println!("{stdout}");
    if (200..300).contains(&response.status_code) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}
