# Event-mention persistence (doctoring)

## Scope

`event_mention` already exists on the foundation schema. This slice adds the
fail-closed insert contract that keeps mention identity distinct from the
promoted instance it supports. A mention cannot be persisted as if it were
the instance (`event_mention_id != event_instance_id`), and confidence is
restricted to a finite score in `(0, 1]`.

This does not add a new migration number, so it can land independently of
stacked `0005`/`0006` PRs.

## Authority

Hovy, E., Marcus, M., Palmer, M., Ramshaw, L., & Weischedel, R. (2006).
OntoNotes: The 90% solution. In *Proceedings of the Human Language Technology
Conference of the NAACL, Companion Volume: Short Papers* (pp. 57–60).
Association for Computational Linguistics.

Pustejovsky, J., Castano, J., Ingria, R., Saurí, R., Gaizauskas, R., Setzer,
A., Katz, G., & Radev, D. (2003). TimeML: Robust specification of event and
temporal expressions in text. In *New Directions in Question Answering* (pp.
28–34). AAAI Press.

Mentions are observations; instances are promoted entities. Collapsing those
identities would treat an observation as the event itself (Hovy et al., 2006;
Pustejovsky et al., 2003).

## Verification

- contract tests reject mention-as-instance and non-finite or out-of-range
  confidence;
- recording-session coverage for insert SQL;
- live PostgreSQL CI inserts a valid mention and refuses identity collapse
  when `TEPP_LIVE_POSTGRES=1`.
