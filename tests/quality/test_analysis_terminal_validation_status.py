from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ANALYSIS_ENGINE = REPO_ROOT / "crates" / "analysis_engine" / "src"

PROFILE_CONTRACTS = {
    "copy_identity_artifact.rs": "COPY_IDENTITY_INFERENCE_STATUS",
    "inferred_status_artifact.rs": "INFERRED_STATUS_INFERENCE_STATUS",
    "location_membership_artifact.rs": "LOCATION_MEMBERSHIP_INFERENCE_STATUS",
    "episode_membership_artifact.rs": "EPISODE_MEMBERSHIP_INFERENCE_STATUS",
    "subevent_containment_artifact.rs": "SUBEVENT_CONTAINMENT_INFERENCE_STATUS",
    "membership_target_artifact.rs": "MEMBERSHIP_TARGET_INFERENCE_STATUS",
}


def test_domain_inference_claims_do_not_occupy_terminal_validation_status() -> None:
    for filename, inference_constant in PROFILE_CONTRACTS.items():
        source = (ANALYSIS_ENGINE / filename).read_text(encoding="utf-8")
        assert f"inference_status: {inference_constant}.into()" in source
        assert f"validation_status: {inference_constant}.into()" not in source
        assert f", {inference_constant},\n    )?;" not in source
        assert '"validated"' in source
