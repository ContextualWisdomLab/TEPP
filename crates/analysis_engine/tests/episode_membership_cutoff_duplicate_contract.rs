//! Regression contract for cutoff-safe episode-membership duplicate admission.

use analysis_engine::{
    EPISODE_MEMBERSHIP_MODEL_CONTRACT_VERSION, EPISODE_MEMBERSHIP_OUTPUT_PROFILE,
    EpisodeMembershipAssignment, execute_episode_membership_run,
};
use episode_membership::EventWindow;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn window(start: i64, end: i64) -> EventWindow {
    EventWindow::new(start, end).expect("valid event window")
}

fn assignment(
    assignment_id: &str,
    membership: EventWindow,
    episode: EventWindow,
    available_at: &str,
) -> EpisodeMembershipAssignment {
    EpisodeMembershipAssignment::new(
        assignment_id,
        membership,
        episode,
        AvailableTime::parse_rfc3339(available_at).expect("valid availability time"),
    )
    .expect("valid assignment")
}

#[test]
fn future_duplicate_identity_cannot_change_historical_cutoff_result() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "episode-membership-cutoff-duplicate".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-episode-membership-cutoff-duplicate".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: EPISODE_MEMBERSHIP_MODEL_CONTRACT_VERSION.into(),
        output_profile: EPISODE_MEMBERSHIP_OUTPUT_PROFILE.into(),
    };
    let accepted = AnalysisRunAccepted::new(
        "run-episode-membership-cutoff-duplicate",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted receipt");
    let cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff).expect("cutoff");
    let episode = window(10, 20);

    let visible = vec![
        assignment(
            "contained-a",
            window(11, 19),
            episode,
            "2026-07-01T00:00:00Z",
        ),
        assignment("escaped-b", window(9, 15), episode, "2026-07-02T00:00:00Z"),
    ];
    let baseline = execute_episode_membership_run(
        &request,
        &accepted,
        &request.snapshot_id,
        cutoff,
        &visible,
        "2026-08-02T00:00:00Z",
    )
    .expect("historical baseline");

    let mut with_future_duplicate = vec![assignment(
        "contained-a",
        window(12, 13),
        episode,
        "2026-08-02T00:00:00Z",
    )];
    with_future_duplicate.extend(visible);
    let replay = execute_episode_membership_run(
        &request,
        &accepted,
        &request.snapshot_id,
        cutoff,
        &with_future_duplicate,
        "2026-08-02T00:00:00Z",
    )
    .expect("future-unavailable duplicate must not enter cutoff admission");

    assert_eq!(replay.artifact, baseline.artifact);
    assert_eq!(replay.terminal_result, baseline.terminal_result);
}
