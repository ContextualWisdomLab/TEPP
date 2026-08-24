//! Runnable loopback ingress for trusted same-host TEPP consumers.

use std::net::SocketAddr;

use tepp_api::AnalysisRunLiveService;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:18081";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let bind_addr = arguments
        .next()
        .unwrap_or(DEFAULT_BIND_ADDR.to_owned())
        .parse::<SocketAddr>()?;
    let request_limit = arguments
        .next()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .unwrap_or(usize::MAX);
    let mut service = AnalysisRunLiveService::bind(bind_addr)?;
    println!("{}", service.local_addr()?);
    (0..request_limit).for_each(|_| drop(service.serve_one()));
    Ok(())
}
