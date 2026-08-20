"""Align TEPP project-history clocks and evidence provenance with LineageWeave."""

from __future__ import annotations

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    """Replace one exact source anchor or accept an already-applied edit."""
    target = Path(path)
    text = target.read_text(encoding="utf-8")
    if new in text:
        return
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one anchor, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


def main() -> None:
    """Patch the DTO, leakage rule, findings, and contract fixtures."""
    source = "crates/tepp_api/src/project_history.rs"
    contract_test = "crates/tepp_api/tests/lineageweave_project_history_contract.rs"

    replace_once(
        source,
        """    /// Event occurrence instant as RFC 3339.
    pub occurred_at: String,
    /// Instant at which this evidence was available to the analysis.
    pub available_at: String,
    /// Authorized LineageWeave source-post identity.
""",
        """    /// Event occurrence instant as RFC 3339.
    pub event_time: String,
    /// Instant at which this evidence was available to the analysis.
    pub available_at: String,
    /// Explicit provenance basis for `available_at`.
    pub availability_basis: String,
    /// Authorized LineageWeave source-post identity.
""",
    )
    replace_once(
        source,
        """        let left_time = parse_timestamp(&left.occurred_at);
        let right_time = parse_timestamp(&right.occurred_at);
""",
        """        let left_time = parse_timestamp(&left.event_time);
        let right_time = parse_timestamp(&right.event_time);
""",
    )
    replace_once(
        source,
        """        .map(|event| event.occurred_at.clone())
""",
        """        .map(|event| event.event_time.clone())
""",
    )
    # The same expression occurs once for the end after the start replacement.
    replace_once(
        source,
        """        .map(|event| event.occurred_at.clone())
""",
        """        .map(|event| event.event_time.clone())
""",
    )
    replace_once(
        source,
        """    validate_code(&event.event_type_code)?;
    validate_bounded_text(&event.event_title, 512)?;
""",
        """    validate_code(&event.event_type_code)?;
    validate_code(&event.availability_basis)?;
    validate_bounded_text(&event.event_title, 512)?;
""",
    )
    replace_once(
        source,
        """    let occurred_at = parse_timestamp(&event.occurred_at)?;
    let available_at = parse_timestamp(&event.available_at)?;
    if occurred_at > *cutoff || available_at > *cutoff {
        return Err(ApiError::InvalidWirePayload);
    }
""",
        """    let _event_time = parse_timestamp(&event.event_time)?;
    let available_at = parse_timestamp(&event.available_at)?;
    // Event time may lie after the analysis cutoff when a future commitment or
    // scheduled milestone was already known. Leakage is governed by evidence
    // availability, not by the time the described event occurs.
    if available_at > *cutoff {
        return Err(ApiError::InvalidWirePayload);
    }
""",
    )

    replace_once(
        source,
        """fn build_findings(
    ordered: &[ProjectHistoryEvent],
    focus_index: usize,
) -> Vec<ProjectHistoryFinding> {
    let before = &ordered[..focus_index];
    let after = &ordered[focus_index + 1..];
    let specification = first_type(before, "specification_changed");
    let handoff = first_type(before, "handoff_recorded");
    let mut findings = Vec::new();
    append_single_finding(
        &mut findings,
        first_type(before, "contract_awarded"),
        "contract_award_before_focus",
        "An explicit contract-award event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        specification,
        "specification_change_before_focus",
        "An explicit specification-change event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        first_type(before, "delivered"),
        "delivery_before_focus",
        "An explicit delivery event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        handoff,
        "handoff_before_focus",
        "An explicit operational-handoff event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        first_type(after, "rebid_started"),
        "rebid_after_focus",
        "An explicit rebid event follows the focus event.",
    );
    if let (Some(specification), Some(handoff)) = (specification, handoff) {
        findings.push(combined_finding(specification, handoff));
    }
    findings
}
""",
        """fn build_findings(
    ordered: &[ProjectHistoryEvent],
    focus_index: usize,
) -> Vec<ProjectHistoryFinding> {
    let before = &ordered[..focus_index];
    let focus = &ordered[focus_index];
    let after = &ordered[focus_index + 1..];
    let specification = first_type(before, "specification_changed");
    let handoff = first_type(before, "handoff_recorded");
    let mut findings = Vec::new();
    append_single_finding(
        &mut findings,
        first_type(before, "contract_awarded"),
        focus,
        "contract_award_before_focus",
        "An explicit contract-award event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        specification,
        focus,
        "specification_change_before_focus",
        "An explicit specification-change event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        first_type(before, "delivered"),
        focus,
        "delivery_before_focus",
        "An explicit delivery event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        handoff,
        focus,
        "handoff_before_focus",
        "An explicit operational-handoff event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        first_type(after, "rebid_started"),
        focus,
        "rebid_after_focus",
        "An explicit rebid event follows the focus event.",
    );
    if let (Some(specification), Some(handoff)) = (specification, handoff) {
        findings.push(combined_finding(specification, handoff, focus));
    }
    findings
}
""",
    )
    replace_once(
        source,
        """fn append_single_finding(
    findings: &mut Vec<ProjectHistoryFinding>,
    event: Option<&ProjectHistoryEvent>,
    finding_code: &str,
    summary: &str,
) {
    if let Some(event) = event {
        findings.push(ProjectHistoryFinding {
            finding_code: finding_code.to_owned(),
            summary: summary.to_owned(),
            related_event_ids: vec![event.event_id.clone()],
            evidence_post_ids: vec![event.source_post_id.clone()],
        });
    }
}

fn combined_finding(
    specification: &ProjectHistoryEvent,
    handoff: &ProjectHistoryEvent,
) -> ProjectHistoryFinding {
    let evidence_post_ids = [
        specification.source_post_id.clone(),
        handoff.source_post_id.clone(),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect();
    ProjectHistoryFinding {
        finding_code: "specification_change_and_handoff_before_focus".into(),
        summary: "Explicit specification-change and handoff events precede the focus event; this is a temporal association, not a causal conclusion.".into(),
        related_event_ids: vec![
            specification.event_id.clone(),
            handoff.event_id.clone(),
        ],
        evidence_post_ids,
    }
}
""",
        """fn append_single_finding(
    findings: &mut Vec<ProjectHistoryFinding>,
    event: Option<&ProjectHistoryEvent>,
    focus: &ProjectHistoryEvent,
    finding_code: &str,
    summary: &str,
) {
    if let Some(event) = event {
        let related_event_ids = [event.event_id.clone(), focus.event_id.clone()]
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let evidence_post_ids = [
            event.source_post_id.clone(),
            focus.source_post_id.clone(),
        ]
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
        findings.push(ProjectHistoryFinding {
            finding_code: finding_code.to_owned(),
            summary: format!(
                "{summary} This is a temporal association, not a causal conclusion."
            ),
            related_event_ids,
            evidence_post_ids,
        });
    }
}

fn combined_finding(
    specification: &ProjectHistoryEvent,
    handoff: &ProjectHistoryEvent,
    focus: &ProjectHistoryEvent,
) -> ProjectHistoryFinding {
    let related_event_ids = [
        specification.event_id.clone(),
        handoff.event_id.clone(),
        focus.event_id.clone(),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect();
    let evidence_post_ids = [
        specification.source_post_id.clone(),
        handoff.source_post_id.clone(),
        focus.source_post_id.clone(),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect();
    ProjectHistoryFinding {
        finding_code: "specification_change_and_handoff_before_focus".into(),
        summary: "Explicit specification-change and handoff events precede the focus event. This is a temporal association, not a causal conclusion.".into(),
        related_event_ids,
        evidence_post_ids,
    }
}
""",
    )

    replace_once(
        source,
        """                occurred_at: "2026-08-19T09:00:00Z".into(),
                available_at: "2026-08-19T10:00:00Z".into(),
                source_post_id: "post".into(),
""",
        """                event_time: "2026-08-19T09:00:00Z".into(),
                available_at: "2026-08-19T10:00:00Z".into(),
                availability_basis: "source_created_at_proxy".into(),
                source_post_id: "post".into(),
""",
    )

    replace_once(
        contract_test,
        """        occurred_at: occurred_at.into(),
        available_at: occurred_at.into(),
        source_post_id: source_post_id.into(),
""",
        """        event_time: occurred_at.into(),
        available_at: occurred_at.into(),
        availability_basis: "source_created_at_proxy".into(),
        source_post_id: source_post_id.into(),
""",
    )
    replace_once(
        contract_test,
        """    assert_eq!(projection.inference_status, "temporal_association_only");
    assert_eq!(projection.participant_count, 3);
""",
        """    assert_eq!(projection.inference_status, "temporal_association_only");
    assert_eq!(projection.participant_count, 3);
    assert!(projection
        .events
        .iter()
        .all(|event| event.availability_basis == "source_created_at_proxy"));
""",
    )
    replace_once(
        contract_test,
        """    assert!(projection
        .findings
        .iter()
        .all(|finding| !finding.evidence_post_ids.is_empty()));
}
""",
        """    assert!(projection.findings.iter().all(|finding| {
        !finding.evidence_post_ids.is_empty()
            && finding.related_event_ids.contains(&"event-voc".to_owned())
            && finding.summary.contains("temporal association")
            && finding.summary.contains("not a causal conclusion")
    }));
}
""",
    )
    replace_once(
        contract_test,
        """    let mut duplicate = sample_request();
    duplicate.events[1].event_id = duplicate.events[0].event_id.clone();
""",
        """    let mut scheduled = sample_request();
    scheduled.events[0].event_time = "2026-08-21T09:00:00Z".into();
    scheduled.events[0].available_at = "2026-08-19T12:00:00Z".into();
    assert!(project_history_projection(&scheduled).is_ok());

    let mut invalid_basis = sample_request();
    invalid_basis.events[0].availability_basis = "source.post.created_at".into();
    assert_eq!(
        project_history_projection(&invalid_basis),
        Err(ApiError::InvalidWirePayload)
    );

    let mut duplicate = sample_request();
    duplicate.events[1].event_id = duplicate.events[0].event_id.clone();
""",
    )


if __name__ == "__main__":
    main()
