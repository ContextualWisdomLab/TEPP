### Changed

- Remove `# pragma: no cover` from every owned Python tooling entrypoint and
  drop the global `exclude_lines` policy from `.coveragerc`; each covered
  script's real `__main__` boundary is now executed in-process and a
  regression fails if pragma-based suppression is reintroduced (#493).
