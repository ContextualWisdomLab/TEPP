# Project-history parent restack evidence

## Exact parents

This stacked branch preserves both reviewed lines through an ordinary two-parent merge commit:

- project-history child before restack: `855c6c7153c2f66a1c14e842ad700f571592dd35`;
- current modular-consumer parent: `cbb3dc0aa657c8d95f18be512ae33d0a1263f2ca`.
- resulting merge commit: `c9103d1e4becb597af98470dfe54ec7d1603762c`, with parents `855c6c7153c2f66a1c14e842ad700f571592dd35` and `cbb3dc0aa657c8d95f18be512ae33d0a1263f2ca`;
- current PR head at verification: `18c0fcd43a35fa934505065f1465c5b825f30e84`;
- protected remote `main` head at verification: `c45be17a9dbce95ef81cee230e9d128abc7160ac` (the local `main` ref was `3810bb73e3606431e1e19497b9746a8335e5d379`).

`git merge-base --is-ancestor` confirms the merge commit is an ancestor of this
branch head, but not of either observed local or protected-remote `main` ref.

The merge retains the parent’s current analysis-run parsing, consumer-aware live ingress, wire validation, and regression tests. It retains the child’s `/v1/project-histories` DTOs, credential-free LineageWeave exchange, deterministic projection logic, scientific-claim boundary, and contract tests.

## Conflict disposition

The two branches both touched the crate root and root changelog. The crate root keeps the child’s superset exports, including all parent analysis-run exports plus the project-history module. The root changelog keeps the parent entry, while the child release note is retained as `CHANGELOG.d/lineageweave-project-history.md` rather than deleting the parent record.

No branch history is rewritten, no force update is used, and no product capability is removed. Fresh exact-head Rust, documentation, security, SAST, coverage, and independent-review evidence remains mandatory after this merge.
