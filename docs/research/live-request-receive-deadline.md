# Live loopback receive-budget repair

## Existing product contract

PRD v0.4 §17 requires size/depth and untrusted-document boundary enforcement. `docs/OPERABILITY.md` requires bounded failure behavior. This correction applies the already declared `NARUON_LIVE_IO_TIMEOUT = 1 second` receive allowance to the whole accepted request; it changes no estimator, tenant authority, wire schema or scientific target. The existing same-host trust limitation remains.

## Observed defect and RED

An independent read-only replay of the built `tepp-loopback` binary sent one header byte every 0.4 seconds. The listener had no response after 2.424 seconds, then returned HTTP400 only when input was closed. An idle client timed out at approximately one second and a prompt valid request succeeded; a trickled valid request was accepted after approximately 2.420 seconds. Those observations distinguish per-read inactivity timeout from a request receive budget.

Four permanent actual-socket regressions replay the same shared cause across Naruon/Analysis and header/body phases. All four failed with HTTP400 instead of the required budget refusal413 before production changes. They join the server thread before asserting, retain finite socket bounds and do not change the product allowance.

## Repair and limitations

A private borrowed-socket `Read` adapter records one monotonic deadline and installs only the remaining allowance before each partial read. It rechecks the deadline after the underlying read, refusing late complete bytes before dispatch. Header and `read_exact` body share that adapter; neither partial progress nor Interrupted retries receive a fresh allowance. Zero/exhausted duration is refused before calling `set_read_timeout` (Rust Project Developers, n.d.-a). The existing framing reader and error mapping remain authoritative.

The generic public parser over arbitrary `Read` remains a framing-only boundary; it cannot preempt an arbitrary supplied blocking reader. The accepted live socket uses the new adapter. Response writes retain the existing per-operation timeout: this is not a whole request/compute/write end-to-end deadline, parallel server, OS real-time scheduling guarantee, production authentication or durable processing claim. Current source tests, coverage and exact-head hosted approval must be acquired independently; prior MLX candidate results do not cover these new files.

## References (APA 7)

Rust Project Developers. (n.d.-a). *TcpStream: set_read_timeout*. Rust standard library documentation. Retrieved October 4, 2026, from https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.set_read_timeout

Rust Project Developers. (n.d.-b). *Read: read_exact*. Rust standard library documentation. Retrieved October 4, 2026, from https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact

Both primary pages were actually retrieved using parent web.run. Published standard-library version1.99 documentation supports interface semantics; actual execution uses repository Rust1.98.0 and retained native tests.
