//! Private deadline edge checks without adding assertion branches to src coverage.

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

use super::DeadlineReader;

#[test]
fn exhausted_budget_is_refused_before_the_socket_can_block() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let _client = TcpStream::connect(listener.local_addr().expect("address")).expect("connect");
    let (mut server, _) = listener.accept().expect("accept");
    let mut reader = DeadlineReader {
        stream: &mut server,
        deadline: Instant::now(),
    };
    let error = reader.read(&mut [0_u8; 1]).expect_err("exhausted budget");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(reader.stream.read_timeout().expect("timeout"), None);
}

#[test]
fn successful_partial_reads_retain_one_deadline_and_socket_ownership() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let mut client = TcpStream::connect(listener.local_addr().expect("address")).expect("connect");
    let (mut server, _) = listener.accept().expect("accept");
    client.write_all(b"ab").expect("input");
    let deadline = Instant::now() + Duration::from_secs(1);
    {
        let mut reader = DeadlineReader {
            stream: &mut server,
            deadline,
        };
        let mut bytes = [0_u8; 1];
        assert_eq!(reader.read(&mut bytes).expect("first read"), 1);
        assert_eq!(bytes, *b"a");
        let first = reader
            .stream
            .read_timeout()
            .expect("first timeout")
            .expect("bounded");
        assert_eq!(reader.read(&mut bytes).expect("second read"), 1);
        assert_eq!(bytes, *b"b");
        let second = reader
            .stream
            .read_timeout()
            .expect("second timeout")
            .expect("bounded");
        assert!(second <= first);
        assert_eq!(reader.deadline, deadline);
    }
    server.write_all(b"ok").expect("caller retains socket");
    let mut reply = [0_u8; 2];
    client.read_exact(&mut reply).expect("reply");
    assert_eq!(reply, *b"ok");
}
