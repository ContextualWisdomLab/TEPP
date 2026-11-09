//! A live request's header and body share one monotonic receive deadline.

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use tepp_api::{AnalysisRunLiveService, NaruonLiveService};

/// Bytes dripped one at a time; never a complete header block or declared body.
const DRIP_BYTES: &[u8; 20] = b"POST /drip/drip/drip";

/// Pause between dripped bytes; far below the one-second receive budget.
const DRIP_INTERVAL: Duration = Duration::from_millis(150);

/// Upper bound on the server's accepted-request phase.
///
/// The budget is one second and a restarted budget cannot finish before the
/// three-second drip ends, so this leaves one second for scheduling and the
/// response write while still separating the two behaviours.
const RECEIVE_PHASE_LIMIT: Duration = Duration::from_millis(2000);

/// Connect before the server thread starts so its timer excludes accept wait.
fn connect_client(addr: SocketAddr) -> TcpStream {
    let client = TcpStream::connect(addr).expect("connect owned listener");
    client
        .set_write_timeout(Some(Duration::from_secs(2)))
        .expect("write bound");
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read bound");
    client
}

/// Keep each read active while the complete request exceeds the receive budget.
fn drip_request(mut client: TcpStream, body_phase: bool) {
    if body_phase {
        client
            .write_all(b"POST / HTTP/1.1\r\ncontent-length: 64\r\n\r\n")
            .expect("header");
    }
    for byte in DRIP_BYTES {
        if client.write_all(&[*byte]).is_err() {
            break;
        }
        thread::sleep(DRIP_INTERVAL);
    }
    let _ = client.shutdown(Shutdown::Write);
    let mut response = Vec::new();
    let _ = client.read_to_end(&mut response);
}

/// Assert the refusal status and the bounded accepted-request phase.
fn assert_budget_refusal(status_code: u16, elapsed: Duration) {
    assert_eq!(status_code, 413, "receive budget did not refuse the drip");
    assert!(
        elapsed < RECEIVE_PHASE_LIMIT,
        "receive budget restarted: {elapsed:?}"
    );
}

/// Exercise the public Naruon listener while settling its owned server thread.
fn naruon_deadline(body_phase: bool) {
    let mut service = NaruonLiveService::bind_loopback().expect("bind");
    let client = connect_client(service.local_addr().expect("address"));
    let worker = thread::spawn(move || {
        let started = Instant::now();
        let response = service.serve_one();
        (response, started.elapsed())
    });
    drip_request(client, body_phase);
    let (response, elapsed) = worker.join().expect("settled server thread");
    assert_budget_refusal(
        response.expect("redacted HTTP refusal").status_code,
        elapsed,
    );
}

/// Exercise the shared Analysis listener with the identical wire timing.
fn analysis_deadline(body_phase: bool) {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let client = connect_client(service.local_addr().expect("address"));
    let worker = thread::spawn(move || {
        let started = Instant::now();
        let response = service.serve_one();
        (response, started.elapsed())
    });
    drip_request(client, body_phase);
    let (response, elapsed) = worker.join().expect("settled server thread");
    assert_budget_refusal(
        response.expect("redacted HTTP refusal").status_code,
        elapsed,
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
