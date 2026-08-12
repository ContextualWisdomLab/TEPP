//! Simulation scenario configuration.

use crate::SimulationError;

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
}

impl SimulationConfig {
    /// Construct a validated configuration.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidConfiguration`] when counts are zero or
    /// any rate exceeds `10_000` basis points.
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
        }
    }

    /// Validate configuration bounds.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidConfiguration`] for empty counts or
    /// rates outside `0..=10_000`.
    pub fn validate(self) -> Result<(), SimulationError> {
        if self.event_count == 0 {
            return Err(SimulationError::InvalidConfiguration);
        }
        if self.documents_per_event == 0 {
            return Err(SimulationError::InvalidConfiguration);
        }
        if self.membership_targets == 0 {
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
        Ok(())
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

    /// Distinct membership targets attached to each document.
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
}

#[cfg(test)]
mod tests {
    use super::SimulationConfig;
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
        let ci = SimulationConfig::ci_default(42);
        ci.validate().expect("ci default");
        assert_eq!(ci.seed(), 42);
    }
}
