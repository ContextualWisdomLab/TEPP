"""Architecture fitness for Longitudinal Modeling ownership boundaries."""

from pathlib import Path


ARCHITECTURE = (Path(__file__).parents[2] / "ARCHITECTURE.md").read_text(encoding="utf-8")


def _section(start: str, end: str) -> str:
    """Return one architecture section bounded by exact headings/markers."""
    return ARCHITECTURE.split(start, 1)[1].split(end, 1)[0]


def test_conceptual_boundaries_do_not_assign_temporal_composition_to_psychometric_core() -> None:
    """Longitudinal/event-time composition belongs to its own bounded context."""
    bounded = _section("## Bounded services and Rust crates", "Every boundary must")
    psychometric_row = next(
        line for line in bounded.splitlines() if line.startswith("| `psychometric_core` |")
    )

    for leaked_authority in ("longitudinal invariance", "DSEM", "continuous-time paths"):
        assert leaked_authority not in psychometric_row, (
            "psychometric_core still claims Longitudinal Modeling authority: "
            f"{leaked_authority}"
        )

    assert "| `longitudinal_modeling` |" in bounded
    longitudinal_row = next(
        line for line in bounded.splitlines() if line.startswith("| `longitudinal_modeling` |")
    )
    for owned_semantic in ("event-time", "longitudinal", "state"):
        assert owned_semantic in longitudinal_row


def test_implementation_topology_has_one_psychometric_and_one_longitudinal_owner_row() -> None:
    """Implementation topology cannot duplicate or conflate responsibility rows."""
    topology = _section("## Implemented foundation topology", "Foundation crates expose")
    psychometric_rows = [
        line for line in topology.splitlines() if line.startswith("| `psychometric_core` |")
    ]
    longitudinal_rows = [
        line for line in topology.splitlines() if line.startswith("| `longitudinal_core` |")
    ]

    assert len(psychometric_rows) == 1, "psychometric_core has duplicate responsibility rows"
    assert len(longitudinal_rows) == 1, "longitudinal_core has duplicate responsibility rows"

    psychometric_row = psychometric_rows[0]
    for temporal_implementation in (
        "event-time log-rate",
        "discrete process noise",
        "stationary within-subject variance",
        "TDPREDEFFECT",
    ):
        assert temporal_implementation not in psychometric_row, (
            "temporal composition is still assigned to psychometric_core: "
            f"{temporal_implementation}"
        )

    longitudinal_row = longitudinal_rows[0]
    assert "temporal composition" in longitudinal_row
    assert "fast-mlsirm" in longitudinal_row
