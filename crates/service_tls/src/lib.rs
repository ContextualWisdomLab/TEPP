#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Production TLS bind gates for standalone and modular service ports.
//!
//! Non-loopback binds require rustls certificate material. Loopback HTTP is
//! development-only and cannot be claimed as an orchestrator live production
//! port. Table-access host labels fail closed (ADR 0011).

mod bind;
mod config;
mod error;

/// A bind that passed production TLS or honest development classification.
pub use bind::AuthorizedTlsBind;
/// Loopback versus production classification of a bind host.
pub use bind::BindClass;
/// Known-truth outcome of a TLS bind policy decision.
pub use bind::BindDecision;
/// A requested service bind plus PEM material.
pub use bind::TlsBindRequest;
/// Authorize an orchestrator live port that cannot be loopback plaintext.
pub use bind::authorize_orchestrator_live_port;
/// Authorize a production TLS bind or classify loopback HTTP as development.
pub use bind::authorize_production_tls;
/// Classify a bind host as loopback development or production TLS.
pub use bind::classify_bind_host;
/// Fraction of bind decisions that match known truth.
pub use bind::tls_policy_recovery_rate;
/// Build a rustls server config from PEM material.
pub use config::rustls_server_config;
/// Fail-closed production TLS bind errors.
pub use error::TlsError;
