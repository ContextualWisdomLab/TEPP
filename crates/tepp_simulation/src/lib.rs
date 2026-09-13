#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Realistic temporal and event truth simulation for recovery studies.
//!
//! `tepp_simulation` generates small, deterministic known-truth corpora that
//! separate latent event occurrence from document creation and availability,
//! attach multilevel memberships, inject controlled missingness and relation
//! noise, and emit a digest-bound truth manifest for scientific recovery tests.

mod configuration;
mod document_process;
mod error;
mod latent_event;
mod missingness;
mod relation_process;
mod rng;
mod truth_manifest;

/// Bounded parameters for a reproducible truth simulation.
pub use configuration::SimulationConfig;
/// Method-effect labels for generated documents.
pub use document_process::DocumentMethodEffect;
/// Synthetic non-wrapping calendar bound in hours.
pub use document_process::SYNTHETIC_YEAR_HOURS;
/// Simulated document observation.
pub use document_process::SimulatedDocument;
/// Simulated multilevel membership assignment.
pub use document_process::SimulatedMembership;
/// Construct delayed event/document/available clocks.
pub use document_process::delayed_clocks;
/// Membership role vocabulary helper.
pub use document_process::membership_role_at;
/// Refuse a document that is not yet available at the cutoff.
pub use document_process::refuse_unavailable_document;
/// Fail-closed simulation errors.
pub use error::SimulationError;
/// Latent event truth row.
pub use latent_event::LatentEvent;
/// Known latent event state.
pub use latent_event::LatentEventState;
/// Mask observed event times under missingness.
pub use missingness::mask_event_time;
/// Validate missingness rates.
pub use missingness::validate_missingness_rate_bps;
/// Observed noisy relation.
pub use relation_process::ObservedRelation;
/// Simulation relation vocabulary.
pub use relation_process::SimulatedRelationKind;
/// True generative relation.
pub use relation_process::TrueRelation;
/// Deterministic generator.
pub use rng::SeededRng;
/// Digest-bound known-truth corpus.
pub use truth_manifest::TruthManifest;
/// Digest helper for configuration fingerprints.
pub use truth_manifest::{digest_bytes, digest_documents};

use uuid::Uuid;

/// Generate a deterministic truth corpus from a validated configuration.
///
/// # Errors
///
/// Returns [`SimulationError::TemporalInvariantViolation`] when a schedule would
/// wrap the synthetic calendar bound.
#[allow(clippy::too_many_lines)]
pub fn generate(config: SimulationConfig) -> Result<TruthManifest, SimulationError> {
    let mut rng = SeededRng::new(config.seed());
    let config_digest = digest_bytes(&config_fingerprint(config));

    let mut events = Vec::with_capacity(config.event_count() as usize);
    let mut documents = Vec::new();
    let mut true_relations = Vec::new();

    let hour_spacing = config
        .max_report_delay_hours()
        .saturating_add(config.max_availability_delay_hours())
        .saturating_add(24)
        .max(1);

    for ordinal in 0..config.event_count() {
        let event_hour_index = ordinal.saturating_mul(hour_spacing);
        let report_delay = sample_delay_hours(&mut rng, config.max_report_delay_hours());
        let availability_delay =
            sample_delay_hours(&mut rng, config.max_availability_delay_hours());
        let (event_time, document_time, available_time) =
            delayed_clocks(event_hour_index, report_delay, availability_delay)?;
        let event_id = next_id(&mut rng);
        let state = if rng.bernoulli_bps(1_000) {
            LatentEventState::Planned
        } else {
            LatentEventState::Occurred
        };
        events.push(LatentEvent::new(event_id, event_time, ordinal, state));

        for _ in 0..config.documents_per_event() {
            let original = build_document(
                &mut rng,
                config,
                event_id,
                event_time,
                document_time,
                available_time,
                DocumentMethodEffect::Original,
                None,
            );
            let parent_id = original.document_id();
            documents.push(original);

            true_relations.push(TrueRelation::new(
                next_id(&mut rng),
                SimulatedRelationKind::RetrospectivelyReports,
                parent_id,
                event_id,
            ));

            push_variant_if_drawn(
                &mut rng,
                config,
                &mut documents,
                &mut true_relations,
                event_id,
                event_time,
                event_hour_index,
                report_delay,
                availability_delay,
                parent_id,
                DocumentMethodEffect::Revision,
                SimulatedRelationKind::Revises,
                config.revision_rate_bps(),
            )?;
            push_variant_if_drawn(
                &mut rng,
                config,
                &mut documents,
                &mut true_relations,
                event_id,
                event_time,
                event_hour_index,
                report_delay,
                availability_delay,
                parent_id,
                DocumentMethodEffect::Translation,
                SimulatedRelationKind::References,
                config.translation_rate_bps(),
            )?;
            push_variant_if_drawn(
                &mut rng,
                config,
                &mut documents,
                &mut true_relations,
                event_id,
                event_time,
                event_hour_index,
                report_delay,
                availability_delay,
                parent_id,
                DocumentMethodEffect::TemplateCopy,
                SimulatedRelationKind::TemplateCopyOf,
                config.template_copy_rate_bps(),
            )?;
        }
    }

    for window in events.windows(2) {
        true_relations.push(TrueRelation::new(
            next_id(&mut rng),
            SimulatedRelationKind::TransitionsTo,
            window[0].event_id(),
            window[1].event_id(),
        ));
    }

    let observed_relations = apply_relation_noise(&mut rng, config, &true_relations, &documents);
    Ok(TruthManifest::new(
        config.seed(),
        config_digest,
        events,
        documents,
        true_relations,
        observed_relations,
    ))
}

fn config_fingerprint(config: SimulationConfig) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in [
        config.seed(),
        u64::from(config.event_count()),
        u64::from(config.documents_per_event()),
        u64::from(config.membership_targets()),
        u64::from(config.max_report_delay_hours()),
        u64::from(config.max_availability_delay_hours()),
        u64::from(config.missingness_rate_bps()),
        u64::from(config.relation_false_negative_bps()),
        u64::from(config.relation_false_positive_bps()),
        u64::from(config.revision_rate_bps()),
        u64::from(config.translation_rate_bps()),
        u64::from(config.template_copy_rate_bps()),
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

fn next_id(rng: &mut SeededRng) -> Uuid {
    let mut bytes = [0_u8; 16];
    bytes[..8].copy_from_slice(&rng.next_u64().to_le_bytes());
    bytes[8..].copy_from_slice(&rng.next_u64().to_le_bytes());
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

#[allow(clippy::too_many_arguments)]
fn build_document(
    rng: &mut SeededRng,
    config: SimulationConfig,
    event_id: Uuid,
    event_time: temporal_core::EventTime,
    document_time: temporal_core::DocumentTime,
    available_time: temporal_core::AvailableTime,
    method_effect: DocumentMethodEffect,
    parent_document_id: Option<Uuid>,
) -> SimulatedDocument {
    let mut memberships = Vec::with_capacity(config.membership_targets() as usize);
    for index in 0..config.membership_targets() {
        memberships.push(SimulatedMembership::new(
            next_id(rng),
            membership_role_at(index as usize),
            10_000 / config.membership_targets().max(1),
        ));
    }
    let is_missing = rng.bernoulli_bps(config.missingness_rate_bps());
    let observed_event_time = mask_event_time(event_time, is_missing);
    SimulatedDocument::trusted(
        next_id(rng),
        event_id,
        document_time,
        available_time,
        method_effect,
        parent_document_id,
        observed_event_time,
        memberships,
    )
}

#[allow(clippy::too_many_arguments)]
fn push_variant_if_drawn(
    rng: &mut SeededRng,
    config: SimulationConfig,
    documents: &mut Vec<SimulatedDocument>,
    true_relations: &mut Vec<TrueRelation>,
    event_id: Uuid,
    event_time: temporal_core::EventTime,
    event_hour_index: u32,
    report_delay: u32,
    availability_delay: u32,
    parent_id: Uuid,
    method: DocumentMethodEffect,
    relation_kind: SimulatedRelationKind,
    rate_bps: u32,
) -> Result<(), SimulationError> {
    if !rng.bernoulli_bps(rate_bps) {
        return Ok(());
    }
    let (_, document_time, available_time) = delayed_clocks(
        event_hour_index.saturating_add(1),
        report_delay.saturating_add(1),
        availability_delay,
    )?;
    let variant = build_document(
        rng,
        config,
        event_id,
        event_time,
        document_time,
        available_time,
        method,
        Some(parent_id),
    );
    let variant_id = variant.document_id();
    documents.push(variant);
    true_relations.push(TrueRelation::new(
        next_id(rng),
        relation_kind,
        variant_id,
        parent_id,
    ));
    Ok(())
}

fn apply_relation_noise(
    rng: &mut SeededRng,
    config: SimulationConfig,
    true_relations: &[TrueRelation],
    documents: &[SimulatedDocument],
) -> Vec<ObservedRelation> {
    let mut observed = Vec::new();
    for relation in true_relations {
        if rng.bernoulli_bps(config.relation_false_negative_bps()) {
            continue;
        }
        observed.push(ObservedRelation::new(
            relation.relation_id(),
            relation.kind(),
            relation.source_id(),
            relation.target_id(),
            true,
        ));
    }
    if !can_inject_false_positive(documents.len(), rng, config.relation_false_positive_bps()) {
        return observed;
    }
    #[allow(clippy::cast_possible_truncation)]
    let left_index = rng.gen_range(documents.len() as u64) as usize;
    let right_index = (left_index + 1) % documents.len();
    observed.push(ObservedRelation::new(
        next_id(rng),
        SimulatedRelationKind::References,
        documents[left_index].document_id(),
        documents[right_index].document_id(),
        false,
    ));
    observed
}

#[allow(clippy::cast_possible_truncation)]
fn sample_delay_hours(rng: &mut SeededRng, max_hours: u32) -> u32 {
    if max_hours == 0 {
        0
    } else {
        // Upper bound is at most `u32::MAX + 1`, so the cast is exact.
        rng.gen_range(u64::from(max_hours) + 1) as u32
    }
}

fn can_inject_false_positive(
    document_count: usize,
    rng: &mut SeededRng,
    false_positive_bps: u32,
) -> bool {
    document_count >= 2 && rng.bernoulli_bps(false_positive_bps)
}

#[cfg(test)]
mod tests {
    use super::{
        SYNTHETIC_YEAR_HOURS, SimulationConfig, SimulationError, can_inject_false_positive,
        generate, sample_delay_hours,
    };

    #[test]
    fn same_seed_yields_identical_manifest_digest() {
        let config = SimulationConfig::ci_default(11);
        let first = generate(config).expect("first");
        let second = generate(config).expect("second");
        assert_eq!(first.content_digest(), second.content_digest());
        assert_eq!(first.config_digest(), second.config_digest());
        assert_eq!(first.event_count(), config.event_count() as usize);
        assert!(first.document_count() >= config.event_count() as usize);
        first.verify_invariants().expect("invariants");
    }

    #[test]
    fn different_seeds_diverge_and_zero_delay_configs_work() {
        let a = generate(SimulationConfig::ci_default(1)).expect("a");
        let b = generate(SimulationConfig::ci_default(2)).expect("b");
        assert_ne!(a.content_digest(), b.content_digest());

        let zero_delay = SimulationConfig::new(3, 2, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0).expect("cfg");
        let manifest = generate(zero_delay).expect("zero delay");
        assert_eq!(manifest.event_count(), 2);
        for document in manifest.documents() {
            assert!(document.available_time().instant() >= document.document_time().instant());
            assert_eq!(document.memberships().len(), 2);
            assert!(document.observed_event_time().is_some());
        }
        assert!(
            manifest
                .true_relations()
                .iter()
                .any(|relation| relation.kind().is_transition())
        );
    }

    #[test]
    fn invalid_configuration_is_rejected() {
        assert_eq!(
            SimulationConfig::new(1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0),
            Err(SimulationError::InvalidConfiguration)
        );
    }

    #[test]
    fn noise_helpers_and_single_document_path() {
        let config = SimulationConfig::new(5, 1, 1, 1, 0, 0, 0, 0, 10_000, 0, 0, 0).expect("cfg");
        let manifest = generate(config).expect("single");
        assert_eq!(manifest.event_count(), 1);
        assert_eq!(manifest.document_count(), 1);
        assert!(
            manifest
                .observed_relations()
                .iter()
                .all(super::ObservedRelation::is_true_positive)
        );
        let mut rng = super::SeededRng::new(1);
        assert_eq!(sample_delay_hours(&mut rng, 0), 0);
        assert!(sample_delay_hours(&mut rng, 3) <= 3);
        assert!(!can_inject_false_positive(1, &mut rng, 10_000));
        assert!(!can_inject_false_positive(2, &mut rng, 0));
        assert!(can_inject_false_positive(2, &mut rng, 10_000));
    }

    #[test]
    fn oversized_schedules_fail_closed() {
        let config = SimulationConfig::new(1, 8, 1, 1, 2_000, 2_000, 0, 0, 0, 0, 0, 0)
            .expect("config counts are valid");
        assert_eq!(
            generate(config),
            Err(SimulationError::TemporalInvariantViolation)
        );
    }

    #[test]
    fn variant_near_calendar_bound_fails_closed() {
        // Original fits when report_delay == YEAR-2; derivative (+1 hour, +1 report)
        // crosses the synthetic year and must fail closed. Exercise each variant
        // slot so every `push_variant_if_drawn` error path is covered.
        let max_report = SYNTHETIC_YEAR_HOURS - 2;
        for (revision, translation, template) in [(10_000, 0, 0), (0, 10_000, 0), (0, 0, 10_000)] {
            let mut saw_failure = false;
            for seed in 0..50_000_u64 {
                let config = SimulationConfig::new(
                    seed,
                    1,
                    1,
                    1,
                    max_report,
                    0,
                    0,
                    0,
                    0,
                    revision,
                    translation,
                    template,
                )
                .expect("cfg");
                if generate(config) == Err(SimulationError::TemporalInvariantViolation) {
                    saw_failure = true;
                    break;
                }
            }
            assert!(
                saw_failure,
                "expected bound-crossing failure for rates {revision}/{translation}/{template}"
            );
        }
    }

    #[test]
    fn full_variant_rates_emit_derivatives_and_false_positives() {
        let config = SimulationConfig::new(17, 3, 1, 3, 6, 3, 0, 0, 10_000, 10_000, 10_000, 10_000)
            .expect("cfg");
        let manifest = generate(config).expect("variants");
        assert!(manifest.document_count() > manifest.event_count());
        assert!(
            manifest
                .documents()
                .iter()
                .any(|document| document.method_effect().is_derivative())
        );
        assert!(
            manifest
                .observed_relations()
                .iter()
                .any(|relation| !relation.is_true_positive())
        );
        manifest.verify_invariants().expect("invariants");
    }
}
