//! Simulation scenario configuration.

use crate::SimulationError;

/// Deterministic parameters for the known-topic data-generating process.
///
/// Continuous effect magnitudes are stored in integer basis points of one
/// logistic-normal coordinate unit so configurations remain exactly comparable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TopicDgpConfig {
    true_topic_count: u32,
    vocabulary_size: u32,
    document_length: u32,
    topic_separation_bps: u32,
    temporal_drift_bps: u32,
    latent_standard_deviation_bps: u32,
    membership_effect_bps: u32,
    relation_effect_bps: u32,
}

impl TopicDgpConfig {
    /// Construct a validated known-topic DGP configuration.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidConfiguration`] when fewer than two
    /// topics are requested, the vocabulary is smaller than the topic count,
    /// document length is zero, separation or latent scale is zero, or an
    /// effect magnitude exceeds `10_000` basis points.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        true_topic_count: u32,
        vocabulary_size: u32,
        document_length: u32,
        topic_separation_bps: u32,
        temporal_drift_bps: u32,
        latent_standard_deviation_bps: u32,
        membership_effect_bps: u32,
        relation_effect_bps: u32,
    ) -> Result<Self, SimulationError> {
        let config = Self {
            true_topic_count,
            vocabulary_size,
            document_length,
            topic_separation_bps,
            temporal_drift_bps,
            latent_standard_deviation_bps,
            membership_effect_bps,
            relation_effect_bps,
        };
        config.validate()?;
        Ok(config)
    }

    /// Deterministic CI-scale topic DGP with overlap, temporal drift, and
    /// nonzero membership/relation effects.
    #[must_use]
    pub const fn ci_default() -> Self {
        Self {
            true_topic_count: 3,
            vocabulary_size: 12,
            document_length: 96,
            topic_separation_bps: 6_500,
            temporal_drift_bps: 800,
            latent_standard_deviation_bps: 1_200,
            membership_effect_bps: 400,
            relation_effect_bps: 300,
        }
    }

    /// Validate all known-topic DGP bounds.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidConfiguration`] for an invalid topic,
    /// vocabulary, length, or effect-magnitude contract.
    pub fn validate(self) -> Result<(), SimulationError> {
        if self.true_topic_count < 2
            || self.vocabulary_size < self.true_topic_count
            || self.document_length == 0
            || self.topic_separation_bps == 0
            || self.latent_standard_deviation_bps == 0
        {
            return Err(SimulationError::InvalidConfiguration);
        }
        for magnitude in [
            self.topic_separation_bps,
            self.temporal_drift_bps,
            self.latent_standard_deviation_bps,
            self.membership_effect_bps,
            self.relation_effect_bps,
        ] {
            if magnitude > 10_000 {
                return Err(SimulationError::InvalidConfiguration);
            }
        }
        Ok(())
    }

    /// True number of generated topics.
    #[must_use]
    pub const fn true_topic_count(self) -> u32 {
        self.true_topic_count
    }

    /// Number of generated vocabulary terms.
    #[must_use]
    pub const fn vocabulary_size(self) -> u32 {
        self.vocabulary_size
    }

    /// Tokens generated for each simulated document.
    #[must_use]
    pub const fn document_length(self) -> u32 {
        self.document_length
    }

    /// Topic-anchor separation in basis points of the content weight rule.
    #[must_use]
    pub const fn topic_separation_bps(self) -> u32 {
        self.topic_separation_bps
    }

    /// Event-time prevalence slope magnitude in basis points.
    #[must_use]
    pub const fn temporal_drift_bps(self) -> u32 {
        self.temporal_drift_bps
    }

    /// Marginal logistic-normal standard deviation in basis points.
    #[must_use]
    pub const fn latent_standard_deviation_bps(self) -> u32 {
        self.latent_standard_deviation_bps
    }

    /// Weighted multiple-membership contribution magnitude in basis points.
    #[must_use]
    pub const fn membership_effect_bps(self) -> u32 {
        self.membership_effect_bps
    }

    /// Incoming true-transition contribution magnitude in basis points.
    #[must_use]
    pub const fn relation_effect_bps(self) -> u32 {
        self.relation_effect_bps
    }
}

/// Bounded parameters for a reproducible truth simulation.
///
/// Rates use integer basis points (`0..=10_000`) so scenarios remain
/// equality-comparable and free of floating-point configuration drift.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SimulationConfig {
    seed: u64,
    event_count: u32,
    documents_per_event: u32,
    membership_targets: u32,
    max_report_delay_hours: u32,
    max_availability_delay_hours: u32,
    missingness_rate_bps: u32,
    relation_false_negative_bps: u32,
    relation_false_positive_bps: u32,
    revision_rate_bps: u32,
    translation_rate_bps: u32,
    template_copy_rate_bps: u32,
    topic_dgp: TopicDgpConfig,
}

impl SimulationConfig {
    /// Construct a validated configuration.
    ///
    /// The legacy constructor retains its call shape and installs the canonical
    /// CI-scale known-topic DGP. Call [`Self::with_topic_dgp`] to override it.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidConfiguration`] when counts are zero,
    /// membership targets exceed the six owned role labels, or any rate exceeds
    /// `10_000` basis points.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        seed: u64,
        event_count: u32,
        documents_per_event: u32,
        membership_targets: u32,
        max_report_delay_hours: u32,
        max_availability_delay_hours: u32,
        missingness_rate_bps: u32,
        relation_false_negative_bps: u32,
        relation_false_positive_bps: u32,
        revision_rate_bps: u32,
        translation_rate_bps: u32,
        template_copy_rate_bps: u32,
    ) -> Result<Self, SimulationError> {
        let config = Self {
            seed,
            event_count,
            documents_per_event,
            membership_targets,
            max_report_delay_hours,
            max_availability_delay_hours,
            missingness_rate_bps,
            relation_false_negative_bps,
            relation_false_positive_bps,
            revision_rate_bps,
            translation_rate_bps,
            template_copy_rate_bps,
            topic_dgp: TopicDgpConfig::ci_default(),
        };
        config.validate()?;
        Ok(config)
    }

    /// Construct a small deterministic default suitable for CI recovery studies.
    #[must_use]
    pub fn ci_default(seed: u64) -> Self {
        Self {
            seed,
            event_count: 4,
            documents_per_event: 1,
            membership_targets: 3,
            max_report_delay_hours: 48,
            max_availability_delay_hours: 24,
            missingness_rate_bps: 2_500,
            relation_false_negative_bps: 1_000,
            relation_false_positive_bps: 500,
            revision_rate_bps: 5_000,
            translation_rate_bps: 5_000,
            template_copy_rate_bps: 5_000,
            topic_dgp: TopicDgpConfig::ci_default(),
        }
    }

    /// Replace the known-topic DGP while preserving all temporal/noise settings.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidConfiguration`] when the replacement
    /// topic DGP or the resulting full simulation configuration is invalid.
    pub fn with_topic_dgp(mut self, topic_dgp: TopicDgpConfig) -> Result<Self, SimulationError> {
        topic_dgp.validate()?;
        self.topic_dgp = topic_dgp;
        self.validate()?;
        Ok(self)
    }

    /// Validate configuration bounds.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidConfiguration`] for empty counts,
    /// membership targets outside `1..=6`, or rates outside `0..=10_000`.
    pub fn validate(self) -> Result<(), SimulationError> {
        if self.event_count == 0 {
            return Err(SimulationError::InvalidConfiguration);
        }
        if self.documents_per_event == 0 {
            return Err(SimulationError::InvalidConfiguration);
        }
        if self.membership_targets == 0 || self.membership_targets > 6 {
            return Err(SimulationError::InvalidConfiguration);
        }
        for rate in [
            self.missingness_rate_bps,
            self.relation_false_negative_bps,
            self.relation_false_positive_bps,
            self.revision_rate_bps,
            self.translation_rate_bps,
            self.template_copy_rate_bps,
        ] {
            if rate > 10_000 {
                return Err(SimulationError::InvalidConfiguration);
            }
        }
        self.topic_dgp.validate()
    }

    /// Explicit RNG seed.
    #[must_use]
    pub const fn seed(self) -> u64 {
        self.seed
    }

    /// Number of latent events.
    #[must_use]
    pub const fn event_count(self) -> u32 {
        self.event_count
    }

    /// Base original documents generated per latent event.
    #[must_use]
    pub const fn documents_per_event(self) -> u32 {
        self.documents_per_event
    }

    /// Distinct membership targets attached to each document (`1..=6`).
    #[must_use]
    pub const fn membership_targets(self) -> u32 {
        self.membership_targets
    }

    /// Maximum event-to-document reporting delay in hours.
    #[must_use]
    pub const fn max_report_delay_hours(self) -> u32 {
        self.max_report_delay_hours
    }

    /// Maximum document-to-availability embargo delay in hours.
    #[must_use]
    pub const fn max_availability_delay_hours(self) -> u32 {
        self.max_availability_delay_hours
    }

    /// Missingness rate in basis points.
    #[must_use]
    pub const fn missingness_rate_bps(self) -> u32 {
        self.missingness_rate_bps
    }

    /// True-relation false-negative rate in basis points.
    #[must_use]
    pub const fn relation_false_negative_bps(self) -> u32 {
        self.relation_false_negative_bps
    }

    /// False-positive relation injection rate in basis points.
    #[must_use]
    pub const fn relation_false_positive_bps(self) -> u32 {
        self.relation_false_positive_bps
    }

    /// Revision variant rate in basis points.
    #[must_use]
    pub const fn revision_rate_bps(self) -> u32 {
        self.revision_rate_bps
    }

    /// Translation variant rate in basis points.
    #[must_use]
    pub const fn translation_rate_bps(self) -> u32 {
        self.translation_rate_bps
    }

    /// Template-copy variant rate in basis points.
    #[must_use]
    pub const fn template_copy_rate_bps(self) -> u32 {
        self.template_copy_rate_bps
    }

    /// Known-topic data-generating process parameters.
    #[must_use]
    pub const fn topic_dgp(self) -> TopicDgpConfig {
        self.topic_dgp
    }
}

#[cfg(test)]
mod tests {
    use super::{SimulationConfig, TopicDgpConfig};
    use crate::SimulationError;

    #[test]
    fn empty_counts_and_over_range_rates_fail_closed() {
        assert_eq!(
            SimulationConfig::new(1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0),
            Err(SimulationError::InvalidConfiguration)
        );
        assert_eq!(
            SimulationConfig::new(1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0),
            Err(SimulationError::InvalidConfiguration)
        );
        assert_eq!(
            SimulationConfig::new(1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0),
            Err(SimulationError::InvalidConfiguration)
        );
        assert_eq!(
            SimulationConfig::new(1, 1, 1, 1, 1, 1, 10_001, 0, 0, 0, 0, 0),
            Err(SimulationError::InvalidConfiguration)
        );
        let good =
            SimulationConfig::new(9, 2, 1, 2, 12, 6, 100, 200, 300, 400, 500, 600).expect("valid");
        assert_eq!(good.seed(), 9);
        assert_eq!(good.event_count(), 2);
        assert_eq!(good.documents_per_event(), 1);
        assert_eq!(good.membership_targets(), 2);
        assert_eq!(good.max_report_delay_hours(), 12);
        assert_eq!(good.max_availability_delay_hours(), 6);
        assert_eq!(good.missingness_rate_bps(), 100);
        assert_eq!(good.relation_false_negative_bps(), 200);
        assert_eq!(good.relation_false_positive_bps(), 300);
        assert_eq!(good.revision_rate_bps(), 400);
        assert_eq!(good.translation_rate_bps(), 500);
        assert_eq!(good.template_copy_rate_bps(), 600);
        assert_eq!(good.topic_dgp(), TopicDgpConfig::ci_default());
        let ci = SimulationConfig::ci_default(42);
        ci.validate().expect("ci default");
        assert_eq!(ci.seed(), 42);
    }

    #[test]
    fn topic_dgp_bounds_and_replacement_are_fail_closed() {
        for invalid in [
            TopicDgpConfig::new(1, 12, 96, 6_500, 800, 1_200, 400, 300),
            TopicDgpConfig::new(3, 2, 96, 6_500, 800, 1_200, 400, 300),
            TopicDgpConfig::new(3, 12, 0, 6_500, 800, 1_200, 400, 300),
            TopicDgpConfig::new(3, 12, 96, 0, 800, 1_200, 400, 300),
            TopicDgpConfig::new(3, 12, 96, 6_500, 800, 0, 400, 300),
            TopicDgpConfig::new(3, 12, 96, 10_001, 800, 1_200, 400, 300),
            TopicDgpConfig::new(3, 12, 96, 6_500, 10_001, 1_200, 400, 300),
            TopicDgpConfig::new(3, 12, 96, 6_500, 800, 10_001, 400, 300),
            TopicDgpConfig::new(3, 12, 96, 6_500, 800, 1_200, 10_001, 300),
            TopicDgpConfig::new(3, 12, 96, 6_500, 800, 1_200, 400, 10_001),
        ] {
            assert_eq!(invalid, Err(SimulationError::InvalidConfiguration));
        }

        let topic = TopicDgpConfig::new(4, 16, 120, 7_000, 900, 1_300, 500, 250)
            .expect("topic config");
        assert_eq!(topic.true_topic_count(), 4);
        assert_eq!(topic.vocabulary_size(), 16);
        assert_eq!(topic.document_length(), 120);
        assert_eq!(topic.topic_separation_bps(), 7_000);
        assert_eq!(topic.temporal_drift_bps(), 900);
        assert_eq!(topic.latent_standard_deviation_bps(), 1_300);
        assert_eq!(topic.membership_effect_bps(), 500);
        assert_eq!(topic.relation_effect_bps(), 250);
        let configured = SimulationConfig::ci_default(7)
            .with_topic_dgp(topic)
            .expect("replacement");
        assert_eq!(configured.topic_dgp(), topic);
    }
}
