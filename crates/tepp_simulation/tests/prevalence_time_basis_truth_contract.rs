//! Known prevalence trajectories must expose their physical EventTime basis.

use temporal_core::EventTime;
use tepp_simulation::{LatentEvent, SimulationConfig, SimulationError, TruthManifest, generate};

fn with_events(generated: &TruthManifest, events: Vec<LatentEvent>) -> TruthManifest {
    TruthManifest::new(
        generated.seed(),
        generated.config_digest().to_owned(),
        events,
        generated.documents().to_vec(),
        generated.true_relations().to_vec(),
        generated.observed_relations().to_vec(),
        generated.topic_truth().clone(),
    )
}

#[test]
fn generated_topic_truth_exposes_the_exact_physical_time_basis() {
    let config = SimulationConfig::new(2040, 5, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0)
        .expect("simulation config");
    let manifest = generate(config).expect("generated truth");
    let basis = manifest
        .prevalence_time_basis()
        .expect("prevalence time basis");

    assert_eq!(basis.origin_event_time(), manifest.events()[0].event_time());
    assert!(basis.scale_seconds() > 0.0);
    assert!(basis.center_seconds_from_origin() > 0.0);
    assert!((basis.center_seconds_from_origin() - basis.scale_seconds()).abs() < 1.0e-12);

    let denominator = (manifest.events().len() - 1) as f64;
    for (index, event) in manifest.events().iter().enumerate() {
        let expected = 2.0 * index as f64 / denominator - 1.0;
        let actual = basis
            .coordinate_at(event.event_time())
            .expect("event coordinate");
        assert!((actual - expected).abs() < 1.0e-12);
    }
}

#[test]
fn malformed_or_non_affine_event_geometry_fails_closed() {
    let generated = generate(
        SimulationConfig::new(2041, 3, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0)
            .expect("simulation config"),
    )
    .expect("generated truth");

    let one_event = with_events(&generated, vec![generated.events()[0].clone()]);
    assert_eq!(
        one_event.prevalence_time_basis(),
        Err(SimulationError::ManifestInvariantViolation)
    );

    let mut non_contiguous = generated.events().to_vec();
    non_contiguous[1] = LatentEvent::new(
        non_contiguous[1].event_id(),
        non_contiguous[1].event_time(),
        2,
        non_contiguous[1].state(),
    );
    assert_eq!(
        with_events(&generated, non_contiguous).prevalence_time_basis(),
        Err(SimulationError::ManifestInvariantViolation)
    );

    let irregular = vec![
        LatentEvent::new(
            generated.events()[0].event_id(),
            EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("t0"),
            0,
            generated.events()[0].state(),
        ),
        LatentEvent::new(
            generated.events()[1].event_id(),
            EventTime::parse_rfc3339("2026-01-02T00:00:00Z").expect("t1"),
            1,
            generated.events()[1].state(),
        ),
        LatentEvent::new(
            generated.events()[2].event_id(),
            EventTime::parse_rfc3339("2026-01-04T00:00:00Z").expect("t2"),
            2,
            generated.events()[2].state(),
        ),
    ];
    assert_eq!(
        with_events(&generated, irregular).prevalence_time_basis(),
        Err(SimulationError::ManifestInvariantViolation)
    );
}
