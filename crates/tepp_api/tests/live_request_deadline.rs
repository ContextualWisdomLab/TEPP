//! A live request's header and body share one monotonic receive deadline.

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use tepp_api::{AnalysisRunLiveService, NaruonLiveService};

/// Keep each read active while the complete request exceeds the receive budget.
fn drip_request(addr: SocketAddr, body_phase: bool) {
    let mut client = TcpStream::connect(addr).expect("connect owned listener");
    client
        .set_write_timeout(Some(Duration::from_secs(2)))
        .expect("write bound");
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read bound");
    if body_phase {
        client
            .write_all(b"POST / HTTP/1.1\r\ncontent-length: 20\r\n\r\n")
            .expect("header");
    }
    for byte in b"POST /POST" {
        if client.write_all(&[*byte]).is_err() {
            break;
        }
        thread::sleep(Duration::from_millis(150));
    }
    let _ = client.shutdown(Shutdown::Write);
    let mut response = Vec::new();
    let _ = client.read_to_end(&mut response);
}

/// Exercise the public Naruon listener while settling its owned server thread.
fn naruon_deadline(body_phase: bool) {
    let mut service = NaruonLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("address");
    let worker = thread::spawn(move || {
        let started = Instant::now();
        let response = service.serve_one();
        (response, started.elapsed())
    });
    drip_request(addr, body_phase);
    let (response, elapsed) = worker.join().expect("settled server thread");
    assert_eq!(response.expect("redacted HTTP refusal").status_code, 413);
    assert!(
        elapsed < Duration::from_millis(1350),
        "receive budget restarted: {elapsed:?}"
    );
}

/// Exercise the shared Analysis listener with the identical wire timing.
fn analysis_deadline(body_phase: bool) {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("address");
    let worker = thread::spawn(move || {
        let started = Instant::now();
        let response = service.serve_one();
        (response, started.elapsed())
    });
    drip_request(addr, body_phase);
    let (response, elapsed) = worker.join().expect("settled server thread");
    assert_eq!(response.expect("redacted HTTP refusal").status_code, 413);
    assert!(
        elapsed < Duration::from_millis(1350),
        "receive budget restarted: {elapsed:?}"
    );
}

#[test]
fn naruon_slow_header_does_not_restart_receive_budget() {
    naruon_deadline(false);
}

#[test]
fn naruon_slow_body_does_not_restart_receive_budget() {
    naruon_deadline(true);
}

#[test]
fn analysis_slow_header_does_not_restart_receive_budget() {
    analysis_deadline(false);
}

#[test]
fn analysis_slow_body_does_not_restart_receive_budget() {
    analysis_deadline(true);
}
