//! Scientific recovery promotion over independent simulation replications.
//!
//! The Validation Evidence boundary treats one outer element as one independent
//! simulation repetition. Temporal states, clustered units, cross-classified
//! memberships, and other correlated coordinates stay inside that repetition and
//! are reduced to one replication-level RMSE before Monte Carlo uncertainty is
//! evaluated. This prevents within-replication rows from masquerading as `n_sim`.

use crate::claim;
use crate::{PromotedClaim, ValidationError, root_mean_square_error};

/// Promote a scientific claim from grouped recovery replications.
///
/// `planned_replications` is the validation-profile-owned simulation denominator.
/// It must be at least two and must equal both outer slice lengths before any
/// numerical aggregation occurs. A failed, missing, or silently dropped recovery
/// repetition therefore blocks scientific promotion instead of letting survivor
/// rows redefine `n_sim`. A future profile that tolerates estimator failures must
/// represent attempted/recovered/failed outcomes explicitly and predeclare that
/// failure policy; this boundary does not infer or silently discard failures.
///
/// `truth_replications[i]` and `recovered_replications[i]` describe the same
/// estimand state for one independent simulation repetition. Each pair is reduced
/// to one replication-level RMSE first. The internal claim gate then computes the
/// overall RMSE and its delta-method Monte Carlo standard error across those
/// replication-level squared errors. Replications therefore receive equal Monte
/// Carlo weight regardless of how many correlated state coordinates they contain.
///
/// The caller-owned validation profile remains responsible for deciding which
/// coordinates belong inside one replication, whether the outer replications are
/// scientifically independent, and how many replications the design requires.
/// This boundary deliberately does not infer an effective sample size from row,
/// cluster, membership, or time counts.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidConfiguration`] when fewer than two planned
/// independent replications are declared. Returns [`ValidationError::InvalidInput`]
/// when the presented outer denominator differs from the planned count, the truth
/// and recovery replication counts differ, or an inner truth/recovery pair is
/// invalid. The underlying exact-head claim gate also rejects invalid
/// accuracy/uncertainty configuration, head mismatch, or a conservative RMSE bound
/// that does not remain strictly inside the practical target.
pub fn promote_scientific_recovery(
    candidate_head: &str,
    protected_head: &str,
    truth_replications: &[&[f64]],
    recovered_replications: &[&[f64]],
    planned_replications: usize,
    max_rmse: f64,
    se_multiplier: f64,
) -> Result<PromotedClaim, ValidationError> {
    if planned_replications < 2 {
        return Err(ValidationError::InvalidConfiguration);
    }
    if truth_replications.len() != planned_replications
        || recovered_replications.len() != planned_replications
        || truth_replications.len() != recovered_replications.len()
    {
        return Err(ValidationError::InvalidInput);
    }

    let replication_rmse: Result<Vec<_>, _> = truth_replications
        .iter()
        .zip(recovered_replications)
        .map(|(truth, recovered)| root_mean_square_error(truth, recovered))
        .collect();
    let replication_rmse = replication_rmse?;
    let zero_truth = vec![0.0; replication_rmse.len()];

    claim::promote_scientific_recovery(
        candidate_head,
        protected_head,
        &zero_truth,
        &replication_rmse,
        max_rmse,
        se_multiplier,
    )
}
