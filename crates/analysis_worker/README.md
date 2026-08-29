# analysis_worker

One-shot durable worker for an already accepted TEPP analysis run. The worker
uses the existing Rust analysis engine and PostgreSQL persistence authority; it
does not schedule runs or replace upstream tenant authorization.
