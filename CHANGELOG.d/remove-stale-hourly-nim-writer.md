# Remove stale self-modifying hourly writer

- Remove `.github/workflows/hourly-nim-product-development.yml`, which could generate a patch, apply it, push a branch, and create a pull request from inside TEPP.
- Retire its caller-local provider credential/model-routing bootstrap instead of carrying a second contextual-orchestrator authority in the product repository.
- Keep model-backed GitHub automation, when needed, behind the canonical CWL `.github` reusable/exact-SHA boundary and contextual-orchestrator `orchestrator/free` policy; this change does not alter TEPP scientific arithmetic, coverage acceptance criteria, or release authority.
