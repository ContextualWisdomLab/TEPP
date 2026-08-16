//! rustls server-config construction from PEM material.

use crate::TlsError;
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use std::io::Cursor;

/// Build a rustls `ServerConfig` from PEM certificate and private-key material.
///
/// # Errors
///
/// Returns [`TlsError::MissingCertificateMaterial`] when either PEM is empty
/// and [`TlsError::InvalidCertificateMaterial`] when rustls cannot parse or
/// pair the material.
pub fn rustls_server_config(
    certificate_pem: &str,
    private_key_pem: &str,
) -> Result<ServerConfig, TlsError> {
    if certificate_pem.trim().is_empty() || private_key_pem.trim().is_empty() {
        return Err(TlsError::MissingCertificateMaterial);
    }
    let _ = rustls::crypto::ring::default_provider().install_default();
    let certificates = parse_certificates(certificate_pem)?;
    let key = parse_private_key(private_key_pem)?;
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificates, key)
        .map_err(|_| TlsError::InvalidCertificateMaterial)
}

fn parse_certificates(certificate_pem: &str) -> Result<Vec<CertificateDer<'static>>, TlsError> {
    let mut reader = Cursor::new(certificate_pem.as_bytes());
    certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .ok()
        .filter(|certificates| !certificates.is_empty())
        .ok_or(TlsError::InvalidCertificateMaterial)
}

fn parse_private_key(private_key_pem: &str) -> Result<PrivateKeyDer<'static>, TlsError> {
    let mut reader = Cursor::new(private_key_pem.as_bytes());
    private_key(&mut reader)
        .ok()
        .flatten()
        .ok_or(TlsError::InvalidCertificateMaterial)
}

#[cfg(test)]
mod tests {
    use super::{parse_certificates, parse_private_key, rustls_server_config};
    use crate::TlsError;

    #[test]
    fn empty_and_non_pem_material_fail_closed() {
        assert!(matches!(
            rustls_server_config("   ", "key"),
            Err(TlsError::MissingCertificateMaterial)
        ));
        assert!(matches!(
            rustls_server_config("cert", "   "),
            Err(TlsError::MissingCertificateMaterial)
        ));
        assert!(matches!(
            parse_certificates("not-pem"),
            Err(TlsError::InvalidCertificateMaterial)
        ));
        assert!(matches!(
            parse_private_key("not-pem"),
            Err(TlsError::InvalidCertificateMaterial)
        ));
        assert!(matches!(
            parse_certificates("-----BEGIN PRIVATE KEY-----\n-----END PRIVATE KEY-----\n"),
            Err(TlsError::InvalidCertificateMaterial)
        ));
        assert!(matches!(
            parse_private_key("-----BEGIN CERTIFICATE-----\n-----END CERTIFICATE-----\n"),
            Err(TlsError::InvalidCertificateMaterial)
        ));
    }
}
