#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Time-varying cross-classified and multiple-membership assignments.
//!
//! TEPP models documents and other observations as members of many simultaneous
//! contexts—authors, departments, customers, partners, competitors, projects,
//! templates, languages, locations, and episodes—without forcing a single
//! hierarchy. Membership weights and event-time validity intervals are first
//! class so multilevel and multiple-membership estimators can avoid atomistic
//! fallacy.

mod admission;
mod assignment;
mod design_wire;
mod error;
mod ess;
mod icc;
mod identifier;
mod network;
mod role;
mod support_wire;
mod weight;

/// Fail-closed admission for estimators that only support full single membership.
pub use admission::{SingleMembershipAdmission, admit_single_membership};
/// One weighted, role-typed, time-varying membership assignment.
pub use assignment::MembershipAssignment;
/// Membership-owned classification result, wire coordinate, and versioned provenance contracts.
pub use design_wire::{
    MEMBERSHIP_DESIGN_WIRE_VERSION, MEMBERSHIP_OBSERVATION_SUPPORT_DIGEST_VERSION,
    MembershipDesignClassification, MembershipDesignWire,
};
/// Issue a Membership-owned classification result from longitudinal canonical state.
pub use design_wire::classify_membership_observations_wire;
/// Fail-closed membership-domain validation errors.
pub use error::MembershipError;
/// Opaque analytical group identifier.
pub use identifier::GroupId;
/// Opaque analytical member identifier.
pub use identifier::MemberId;
/// In-memory multiple-membership network for estimation inputs.
pub use network::MembershipNetwork;
/// Contextual membership roles (not permanent entity classes).
pub use role::MembershipRole;
/// Reconstructable longitudinal support projection and wire contracts.
pub use support_wire::{
    MEMBERSHIP_OBSERVATION_SUPPORT_WIRE_VERSION, MembershipObservationSupportAssignmentWire,
    MembershipObservationSupportObservationWire, MembershipObservationSupportProjection,
    MembershipObservationSupportWire, project_membership_observations_wire,
};
/// Finite, strictly positive membership share in `(0, 1]`.
pub use weight::MembershipWeight;

/// Design effect `n / ESS` for membership weights.
pub use ess::design_effect;
/// Group-normalized Kish ESS for co-partitioned membership weights.
pub use ess::group_normalized_kish_ess;
/// Kish effective sample size for membership weights.
pub use ess::kish_effective_sample_size;
/// Membership design implied by active assignments.
pub use icc::MembershipDesign;
/// One member/event-time coordinate in longitudinal Membership support.
pub use icc::MembershipObservation;
/// Finite outcome used by the nested ICC estimator.
pub use icc::NestedOutcome;
/// Classify nested versus cross-classified versus multiple-membership designs at one event time.
pub use icc::classify_membership_design;
/// Classify Membership-owned design across observation-specific event times.
pub use icc::classify_membership_observations;
/// CPU `f64` nested ICC that refuses non-nested membership designs.
pub use icc::nested_intraclass_correlation;
