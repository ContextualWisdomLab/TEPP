### Scientific validity

- Multi-window rolling-origin predictive K selection now fails closed when one document identity appears in more than one evaluation window, preventing held-out evidence from being rebound to a later snapshot and counted twice. A prior evaluation document may still enter a later training partition when the canonical split owner admits it; this guard does not mint or replace authoritative cross-snapshot availability provenance.
