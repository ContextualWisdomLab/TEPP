# Whole-request loopback receive deadline

- Apply one monotonic one-second receive budget to the complete HTTP header and declared body in both the Naruon and shared Analysis live listeners. Partial progress no longer restarts the allowance.
- Preserve existing framing/byte limits, fail-closed HTTP 413 mapping, accepted socket ownership, and separate per-operation response-write timeout. The generic in-memory `Read` parser remains unchanged and cannot preempt an arbitrary caller-provided reader.
- Add real loopback slow-header and slow-body regressions for both listeners. This is a bounded same-host availability repair, not production tenancy, durability, concurrent serving, a model deadline, or a scientific release claim.
