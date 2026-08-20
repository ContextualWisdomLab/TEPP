"""Add an explicit evidence-availability basis to TEPP project histories."""

from __future__ import annotations

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    """Replace one exact source anchor or accept an already-applied edit."""
    target = Path(path)
    text = target.read_text(encoding="utf-8")
    if new in text:
        return
    if text.count(old) != 1:
        raise SystemExit(f"{path}: expected one anchor, found {text.count(old)}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


def main() -> None:
    """Patch the DTO, validation, and both contract fixtures."""
    replace_once(
        "crates/tepp_api/src/project_history.rs",
        """    /// Instant at which this evidence was available to the analysis.
    pub available_at: String,
    /// Authorized LineageWeave source-post identity.
""",
        """    /// Instant at which this evidence was available to the analysis.
    pub available_at: String,
    /// Provenance basis for `available_at`, such as a source-created proxy.
    pub availability_basis_code: String,
    /// Authorized LineageWeave source-post identity.
""",
    )
    replace_once(
        "crates/tepp_api/src/project_history.rs",
        """    validate_code(&event.event_type_code)?;
    validate_bounded_text(&event.event_title, 512)?;
""",
        """    validate_code(&event.event_type_code)?;
    validate_code(&event.availability_basis_code)?;
    validate_bounded_text(&event.event_title, 512)?;
""",
    )
    replace_once(
        "crates/tepp_api/src/project_history.rs",
        """                available_at: "2026-08-19T10:00:00Z".into(),
                source_post_id: "post".into(),
""",
        """                available_at: "2026-08-19T10:00:00Z".into(),
                availability_basis_code: "source_created_at_proxy".into(),
                source_post_id: "post".into(),
""",
    )
    replace_once(
        "crates/tepp_api/tests/lineageweave_project_history_contract.rs",
        """        available_at: occurred_at.into(),
        source_post_id: source_post_id.into(),
""",
        """        available_at: occurred_at.into(),
        availability_basis_code: "source_created_at_proxy".into(),
        source_post_id: source_post_id.into(),
""",
    )
    replace_once(
        "crates/tepp_api/tests/lineageweave_project_history_contract.rs",
        """    assert_eq!(projection.inference_status, "temporal_association_only");
    assert_eq!(projection.participant_count, 3);
""",
        """    assert_eq!(projection.inference_status, "temporal_association_only");
    assert_eq!(projection.participant_count, 3);
    assert!(projection.events.iter().all(|event| {
        event.availability_basis_code == "source_created_at_proxy"
    }));
""",
    )


if __name__ == "__main__":
    main()
