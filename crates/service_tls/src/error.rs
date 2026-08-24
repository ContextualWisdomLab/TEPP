//! Fail-closed production TLS bind errors.

use std::fmt;

/// A fail-closed production TLS bind error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TlsError {
    /// A non-loopback bind used `http` or another plaintext scheme.
    PlaintextProductionBind,
    /// Certificate or private-key PEM was empty.
    MissingCertificateMaterial,
    /// Certificate or private-key PEM could not build a rustls server config.
    InvalidCertificateMaterial,
    /// Bind host labels implied direct table access.
    TableAccessHost,
    /// Loopback plaintext was treated as a production TLS live port.
    LoopbackIsNotProductionTls,
    /// Host, scheme, or recovery slices were empty or malformed.
    InvalidBindPayload,
}

impl fmt::Display for TlsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::PlaintextProductionBind => "production binds require TLS",
            Self::MissingCertificateMaterial => "TLS certificate material is required",
            Self::InvalidCertificateMaterial => "TLS certificate material is invalid",
            Self::TableAccessHost => "TLS bind host cannot imply table access",
            Self::LoopbackIsNotProductionTls => "loopback plaintext is not production TLS",
            Self::InvalidBindPayload => "invalid TLS bind payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TlsError {}

#[cfg(test)]
mod tests {
    use super::TlsError;

    #[test]
    fn error_messages_are_stable_and_redacted() {
        for (error, message) in [
            (
                TlsError::PlaintextProductionBind,
                "production binds require TLS",
            ),
            (
                TlsError::MissingCertificateMaterial,
                "TLS certificate material is required",
            ),
            (
                TlsError::InvalidCertificateMaterial,
                "TLS certificate material is invalid",
            ),
            (
                TlsError::TableAccessHost,
                "TLS bind host cannot imply table access",
            ),
            (
                TlsError::LoopbackIsNotProductionTls,
                "loopback plaintext is not production TLS",
            ),
            (TlsError::InvalidBindPayload, "invalid TLS bind payload"),
        ] {
            assert_eq!(error.to_string(), message);
            assert!(!error.to_string().contains("token"));
        }
    }
}
