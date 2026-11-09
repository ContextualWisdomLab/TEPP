//! Offline command that runs the CPU `f64` topic-lineage estimator on one
//! bounded JSON input file and prints one digest-bound JSON result line.
//!
//! Usage: `tepp-topic-lineage <input.json>`. Exit status `0` means success,
//! `2` usage, `3` invalid input, `4` estimator non-convergence, and `5` I/O.

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::from(analysis_engine::run_topic_lineage_cli(
        std::env::args_os().skip(1),
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
    ))
}
