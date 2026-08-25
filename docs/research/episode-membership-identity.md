# Episode membership cannot escape the episode interval (doctoring)

## Scope

`episode_membership` keeps a document's episode assignment inside the
episode's event-time interval. Recovery is the computed share of
containment flags that match known truth.

This slice does not persist memberships, allocate migration `0008`, or
replace `membership_core` or `subevent_containment`.

## Authority

### Normative TEPP contract

- `docs/adr/0003-relational-event-multiple-membership.md` — episodes
  form time-varying multiple-membership assignments with governed
  validity intervals.
- `docs/adr/0002-six-clock-temporal-semantics.md` — membership validity
  is event/valid time.

### Supporting literature

Allen (1983) defines interval `during` and equality. A membership that
starts before or ends after its episode is not `during` that episode.

Allen, J. F. (1983). Maintaining knowledge about temporal intervals.
*Communications of the ACM, 26*(11), 832–843.
https://doi.org/10.1145/182.358434
