//! Runnable loopback ingress for trusted same-host TEPP consumers.

use std::net::SocketAddr;

use tepp_api::AnalysisRunLiveService;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:18081";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind_addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_BIND_ADDR.to_owned())
        .parse::<SocketAddr>()?;
    let mut service = AnalysisRunLiveService::bind(bind_addr)?;
    loop {
        service.serve_one()?;
    }
}
