//! Production and loopback bind classification.

use crate::{TlsError, rustls_server_config};

/// Whether a bind host is local development or a production TLS target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindClass {
    /// Loopback or localhost; plaintext is development-only.
    LoopbackDevelopment,
    /// Non-loopback host; TLS certificate material is required.
    ProductionRequired,
}

/// Known-truth outcome of a TLS bind policy decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindDecision {
    /// Loopback plaintext is allowed only as local development.
    DevelopmentOnly,
    /// A rustls server config was built for a production bind.
    ProductionTls,
    /// The bind was refused.
    Refused,
}

/// A requested service bind plus PEM material.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TlsBindRequest<'a> {
    bind_host: &'a str,
    bind_port: u16,
    scheme: &'a str,
    certificate_pem: &'a str,
    private_key_pem: &'a str,
}

/// A bind that passed production TLS or honest development classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedTlsBind {
    bind_host: String,
    bind_port: u16,
    decision: BindDecision,
}

impl<'a> TlsBindRequest<'a> {
    /// Construct a bind request after rejecting empty or hostile labels.
    ///
    /// # Errors
    ///
    /// Returns [`TlsError::InvalidBindPayload`] when the host or scheme is
    /// empty or contains control or delimiter characters, and
    /// [`TlsError::TableAccessHost`] when the host implies table access.
    pub fn new(
        bind_host: &'a str,
        bind_port: u16,
        scheme: &'a str,
        certificate_pem: &'a str,
        private_key_pem: &'a str,
    ) -> Result<Self, TlsError> {
        validate_scheme(scheme)?;
        classify_bind_host(bind_host)?;
        Ok(Self {
            bind_host,
            bind_port,
            scheme,
            certificate_pem,
            private_key_pem,
        })
    }

    /// Bind host as supplied after validation.
    #[must_use]
    pub const fn bind_host(self) -> &'a str {
        self.bind_host
    }

    /// Bind port as supplied.
    #[must_use]
    pub const fn bind_port(self) -> u16 {
        self.bind_port
    }

    /// Requested URL scheme (`http` or `https`).
    #[must_use]
    pub const fn scheme(self) -> &'a str {
        self.scheme
    }
}

impl AuthorizedTlsBind {
    /// Host authorized for this bind.
    #[must_use]
    pub fn bind_host(&self) -> &str {
        &self.bind_host
    }

    /// Port authorized for this bind.
    #[must_use]
    pub const fn bind_port(&self) -> u16 {
        self.bind_port
    }

    /// Policy decision recorded for recovery comparison.
    #[must_use]
    pub const fn decision(&self) -> BindDecision {
        self.decision
    }
}

/// Classify a bind host as loopback development or production TLS.
///
/// # Errors
///
/// Returns [`TlsError::InvalidBindPayload`] for an empty or hostile host and
/// [`TlsError::TableAccessHost`] when the host implies table access.
pub fn classify_bind_host(bind_host: &str) -> Result<BindClass, TlsError> {
    if bind_host.is_empty()
        || bind_host.chars().any(|ch| {
            ch.is_control() || matches!(ch, ' ' | '\'' | ';' | '\\' | '/' | '?' | '#' | '@')
        })
    {
        return Err(TlsError::InvalidBindPayload);
    }
    let lowered = bind_host.to_ascii_lowercase();
    if lowered.contains("postgres") || lowered.contains("jdbc") {
        return Err(TlsError::TableAccessHost);
    }
    Ok(bind_class(bind_host))
}

fn bind_class(bind_host: &str) -> BindClass {
    let lowered = bind_host.to_ascii_lowercase();
    if matches!(lowered.as_str(), "127.0.0.1" | "::1" | "localhost") {
        BindClass::LoopbackDevelopment
    } else {
        BindClass::ProductionRequired
    }
}

/// Authorize a production TLS bind, or classify loopback HTTP as development.
///
/// # Errors
///
/// Returns a [`TlsError`] when the host is hostile, production is plaintext,
/// or rustls cannot be built from the supplied PEM.
pub fn authorize_production_tls(
    request: &TlsBindRequest<'_>,
) -> Result<AuthorizedTlsBind, TlsError> {
    match bind_class(request.bind_host) {
        BindClass::LoopbackDevelopment => authorize_loopback(request),
        BindClass::ProductionRequired => authorize_production(request),
    }
}

/// Authorize an orchestrator live port. Loopback plaintext is not production.
///
/// # Errors
///
/// Returns [`TlsError::LoopbackIsNotProductionTls`] for loopback HTTP and the
/// same production TLS failures as [`authorize_production_tls`].
pub fn authorize_orchestrator_live_port(
    request: &TlsBindRequest<'_>,
) -> Result<AuthorizedTlsBind, TlsError> {
    let authorized = authorize_production_tls(request)?;
    if authorized.decision == BindDecision::DevelopmentOnly {
        return Err(TlsError::LoopbackIsNotProductionTls);
    }
    Ok(authorized)
}

/// Fraction of bind decisions that match known truth.
///
/// # Errors
///
/// Returns [`TlsError::InvalidBindPayload`] when either slice is empty or the
/// lengths differ.
pub fn tls_policy_recovery_rate(
    truth: &[BindDecision],
    decided: &[BindDecision],
) -> Result<f64, TlsError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(TlsError::InvalidBindPayload);
    }
    let mut matches = 0_u32;
    for (truth_decision, decided_decision) in truth.iter().zip(decided) {
        if truth_decision == decided_decision {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

fn authorize_loopback(request: &TlsBindRequest<'_>) -> Result<AuthorizedTlsBind, TlsError> {
    if request.scheme == "http" {
        return Ok(authorized(request, BindDecision::DevelopmentOnly));
    }
    authorize_https_material(request)
}

fn authorize_production(request: &TlsBindRequest<'_>) -> Result<AuthorizedTlsBind, TlsError> {
    if request.scheme != "https" {
        return Err(TlsError::PlaintextProductionBind);
    }
    authorize_https_material(request)
}

fn authorize_https_material(request: &TlsBindRequest<'_>) -> Result<AuthorizedTlsBind, TlsError> {
    rustls_server_config(request.certificate_pem, request.private_key_pem)?;
    Ok(authorized(request, BindDecision::ProductionTls))
}

fn authorized(request: &TlsBindRequest<'_>, decision: BindDecision) -> AuthorizedTlsBind {
    AuthorizedTlsBind {
        bind_host: request.bind_host.to_owned(),
        bind_port: request.bind_port,
        decision,
    }
}

fn validate_scheme(scheme: &str) -> Result<(), TlsError> {
    if matches!(scheme, "http" | "https") {
        Ok(())
    } else {
        Err(TlsError::InvalidBindPayload)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BindClass, BindDecision, TlsBindRequest, authorize_production_tls, classify_bind_host,
        tls_policy_recovery_rate,
    };
    use crate::TlsError;

    #[test]
    fn accessors_and_empty_recovery_cover_local_branches() {
        let request =
            TlsBindRequest::new("127.0.0.1", 0, "http", "", "").expect("loopback request");
        assert_eq!(request.bind_host(), "127.0.0.1");
        assert_eq!(request.bind_port(), 0);
        assert_eq!(request.scheme(), "http");
        let authorized = authorize_production_tls(&request).expect("dev");
        assert_eq!(authorized.bind_host(), "127.0.0.1");
        assert_eq!(authorized.bind_port(), 0);
        assert_eq!(authorized.decision(), BindDecision::DevelopmentOnly);
        assert_eq!(
            classify_bind_host("::1").expect("ipv6"),
            BindClass::LoopbackDevelopment
        );
        assert_eq!(
            classify_bind_host("localhost").expect("name"),
            BindClass::LoopbackDevelopment
        );
        assert_eq!(
            tls_policy_recovery_rate(&[], &[]),
            Err(TlsError::InvalidBindPayload)
        );
        assert_eq!(
            tls_policy_recovery_rate(&[BindDecision::Refused], &[]),
            Err(TlsError::InvalidBindPayload)
        );
        for host in ["'", ";", "\\", "/", "?", "#", "@", "user@host"] {
            assert_eq!(classify_bind_host(host), Err(TlsError::InvalidBindPayload));
        }
        let cloned = authorized.clone();
        assert_eq!(cloned, authorized);
        let _ = format!("{authorized:?}");
    }
}
