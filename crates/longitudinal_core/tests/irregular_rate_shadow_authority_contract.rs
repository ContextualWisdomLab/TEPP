#![forbid(unsafe_code)]

//! Architecture fitness for one Longitudinal Modeling irregular-rate estimand.
//!
//! The crate-public path and its internal temporal composition must share one
//! numerical authority. A compatibility/public facade may re-export the named
//! operations, but it must not declare wrapper functions or carry a second
//! mean/log-rate implementation whose floating-point edge semantics can drift.

#[test]
fn shadowed_centered_rate_does_not_reintroduce_direct_ratio_rejection() {
    let source = include_str!("../src/irregular_residual.rs");

    assert!(
        !source.contains("let ratio = pair.later_residual() / pair.earlier_residual();"),
        "the internal centered-rate path still rejects finite log-rates solely because the direct residual ratio overflows or underflows"
    );
    assert!(
        source.contains("driver_same_sign_log_rate("),
        "the internal composition must reuse the Longitudinal Modeling same-sign log-rate primitive"
    );
}

#[test]
fn public_facade_reexports_the_canonical_recovery_functions_instead_of_wrapping_them() {
    let canonical = include_str!("../src/irregular_residual.rs");
    let facade = include_str!("../src/stable_irregular_rate.rs");

    assert!(
        canonical.contains("pub fn recover_centered_irregular_residual_log_rate("),
        "the canonical centered-rate function must remain re-exportable from the private implementation module"
    );
    assert!(
        canonical.contains("pub fn recover_within_unit_irregular_residual_log_rate("),
        "the canonical CWC irregular-rate function must remain re-exportable from the private implementation module"
    );
    assert!(
        facade.contains("pub use crate::irregular_residual::{"),
        "the public facade must re-export the canonical function identities rather than declare wrapper functions"
    );
    assert!(
        !facade.contains("pub fn recover_centered_irregular_residual_log_rate("),
        "stable_irregular_rate still declares a second centered-rate wrapper function"
    );
    assert!(
        !facade.contains("pub fn recover_within_unit_irregular_residual_log_rate("),
        "stable_irregular_rate still declares a second CWC irregular-rate wrapper function"
    );
}

#[test]
fn public_irregular_rate_facade_has_no_second_numerical_implementation() {
    let facade = include_str!("../src/stable_irregular_rate.rs");

    assert!(
        !facade.contains("fn stable_mean("),
        "stable_irregular_rate still owns a second floating-point mean implementation"
    );
    assert!(
        facade.contains("recover_centered_irregular_residual_log_rate"),
        "the public facade must expose the centered-rate estimand from the canonical Longitudinal Modeling implementation"
    );
    assert!(
        facade.contains("recover_within_unit_irregular_residual_log_rate"),
        "the public facade must expose the CWC irregular-rate estimand from the canonical Longitudinal Modeling implementation"
    );
}
