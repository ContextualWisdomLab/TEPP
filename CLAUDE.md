# CLAUDE.md

Read and follow `AGENTS.md` before changing this repository. The repository-wide contracts in that file are normative.

## Working method

- Use test-driven development for every behavior change.
- Keep changes bounded to one independently reviewable product or scientific slice.
- Prefer explicit types and small modules with stable interfaces.
- Preserve source spans, temporal provenance, uncertainty, and model-version metadata end to end.
- Do not replace statistical estimation with an LLM judgment.
- Do not convert association, temporal precedence, or document links into causal language without identification evidence.
- Do not remove repeated report language with global stopword lists or use TF-IDF/BM25 as inferential weights. Model template, section, copied-text, style, modality, and corpus-background sources explicitly.
- Do not treat raw topic proportions as ordinary Euclidean indicators. Use logistic-normal coordinates or valid log-ratio coordinates and propagate posterior uncertainty into ESEM/DSEM.
- Never use future-available evidence in historical model fits.

## Verification before completion

Before stating that a task is complete, run the exact focused tests, complete test suite, line/branch coverage gate, docstring gate, formatter, linter, dependency/security checks, build/package checks, and any required CPU/GPU parity or true-parameter study. Report actual evidence and unresolved external gates.
