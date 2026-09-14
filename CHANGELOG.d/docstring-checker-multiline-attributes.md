### Fixed

- `scripts/check_docstrings.py` no longer reports a documented public item as
  undocumented when a multi-line attribute (for example `#[expect(...)]`)
  sits between its `///` block and the item; attribute continuation lines
  are now transparent up to the closing bracket.
