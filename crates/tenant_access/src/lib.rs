#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Tenant, role, and system-time lifetime adapters that refuse event-time
//! authorization.
//!
//! A grant binds one principal to one tenant role for a system-time window.
//! Event, document, availability, and knowledge-cutoff clocks cannot authorize
//! access, and blanket PII masking is not authorization (ADR 0002/0009).

mod adapter;
mod error;
mod grant;
mod identity;
mod role;

/// Clock families that may evaluate an access lifetime.
pub use role::AccessClock;
/// Closed access-control role vocabulary.
pub use role::AccessRole;
/// Parse the clock that may evaluate an access grant.
pub use role::access_clock_from_wire;
/// Explicit refusal to treat blanket PII masking as authorization.
pub use role::refuse_blanket_mask_as_access;
/// Fraction of recovered tenant/role pairs that match known truth.
pub use role::tenant_role_recovery_rate;

/// One tenant/role grant with a system-time lifetime.
pub use grant::AccessGrant;
/// One request to exercise a tenant/role grant.
pub use grant::AccessRequest;
/// Authorize a request when any stored grant matches.
pub use grant::authorize_with_grants;

/// In-memory adapter with no application-table coupling.
pub use adapter::InMemoryTenantAccessAdapter;
/// Evaluate an access request without reading TEPP application tables.
pub use adapter::TenantAccessAdapter;

/// Fail-closed tenant-access errors.
pub use error::TenantAccessError;
/// Opaque principal identity.
pub use identity::PrincipalId;
/// Opaque tenant identity.
pub use identity::TenantId;
