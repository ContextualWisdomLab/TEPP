# Consumer ingress base stabilization

## Scope

The modular LineageWeave consumer-admission change is reviewed and merged as a direct successor to the already-merged loopback ingress in PR #107.

## Exact base correction

- Protected target branch: `main`
- `main` at the correction point: `c45be17a9dbce95ef81cee230e9d128abc7160ac`
- Product head before this evidence-only commit: `17f06e814e943ebd9bf592549e2d218a4efed112`
- Superseded target: the historical PR #107 feature branch

Retargeting does not remove or reimplement any admitted-consumer behavior. It prevents a successful merge from updating only the already-consumed feature branch instead of advancing TEPP `main`.

## Preserved product boundary

The change continues to preserve:

- one consumer-neutral `/v1/analysis-runs` ingress;
- a closed `naruon` and `lineageweave` consumer registry;
- credential-free request construction;
- consumer-qualified tenant/idempotency namespaces;
- deterministic replay and changed-payload rejection;
- the existing Naruon compatibility listener;
- the distinction between `202 Accepted` and a completed psychometric result.

## Merge evidence rule

Retargeting invalidates remembered branch-base assumptions. Merge requires fresh terminal checks and an independent approval on the exact current head. Cancelled, predecessor-head, child-PR, or author-only evidence is not transferable.
