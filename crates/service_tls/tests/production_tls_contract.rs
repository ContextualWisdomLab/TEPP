//! Production TLS binds refuse plaintext, table-access hosts, and loopback claims.

use service_tls::{
    BindClass, BindDecision, TlsBindRequest, TlsError, authorize_orchestrator_live_port,
    authorize_production_tls, classify_bind_host, rustls_server_config, tls_policy_recovery_rate,
};

const CERTIFICATE_PEM: &str = include_str!("fixtures/tepp-tls-cert.pem");
const PRIVATE_KEY_PEM: &str = include_str!("fixtures/tepp-tls-key.pem");
const OTHER_KEY_PEM: &str = include_str!("fixtures/tepp-tls-other-key.pem");

fn request<'a>(
    host: &'a str,
    scheme: &'a str,
    certificate_pem: &'a str,
    private_key_pem: &'a str,
) -> TlsBindRequest<'a> {
    TlsBindRequest::new(host, 8443, scheme, certificate_pem, private_key_pem).expect("request")
}

fn decided_production(
    host: &str,
    scheme: &str,
    certificate_pem: &str,
    private_key_pem: &str,
) -> BindDecision {
    match TlsBindRequest::new(host, 8443, scheme, certificate_pem, private_key_pem) {
        Err(_) => BindDecision::Refused,
        Ok(request) => authorize_production_tls(&request)
            .map_or(BindDecision::Refused, |authorized| authorized.decision()),
    }
}

fn decided_orchestrator(
    host: &str,
    scheme: &str,
    certificate_pem: &str,
    private_key_pem: &str,
) -> BindDecision {
    match TlsBindRequest::new(host, 8443, scheme, certificate_pem, private_key_pem) {
        Err(_) => BindDecision::Refused,
        Ok(request) => authorize_orchestrator_live_port(&request)
            .map_or(BindDecision::Refused, |authorized| authorized.decision()),
    }
}

#[test]
fn plaintext_production_and_table_access_hosts_fail_closed() {
    assert_eq!(
        classify_bind_host("0.0.0.0").expect("public"),
        BindClass::ProductionRequired
    );
    assert_eq!(
        authorize_production_tls(&request(
            "0.0.0.0",
            "http",
            CERTIFICATE_PEM,
            PRIVATE_KEY_PEM
        )),
        Err(TlsError::PlaintextProductionBind)
    );
    assert_eq!(
        TlsBindRequest::new(
            "postgres.example",
            5432,
            "https",
            CERTIFICATE_PEM,
            PRIVATE_KEY_PEM
        ),
        Err(TlsError::TableAccessHost)
    );
    assert_eq!(
        classify_bind_host("jdbc-host"),
        Err(TlsError::TableAccessHost)
    );
    assert_eq!(
        TlsBindRequest::new("", 443, "https", CERTIFICATE_PEM, PRIVATE_KEY_PEM),
        Err(TlsError::InvalidBindPayload)
    );
    assert_eq!(
        TlsBindRequest::new("example.com", 443, "ftp", CERTIFICATE_PEM, PRIVATE_KEY_PEM),
        Err(TlsError::InvalidBindPayload)
    );
    assert_eq!(
        classify_bind_host("bad host"),
        Err(TlsError::InvalidBindPayload)
    );
    assert_eq!(
        classify_bind_host("\u{0007}host"),
        Err(TlsError::InvalidBindPayload)
    );
    assert_eq!(
        authorize_orchestrator_live_port(&request(
            "0.0.0.0",
            "http",
            CERTIFICATE_PEM,
            PRIVATE_KEY_PEM
        )),
        Err(TlsError::PlaintextProductionBind)
    );
}

#[test]
fn production_https_builds_rustls_and_loopback_http_is_not_orchestrator_live() {
    let production = request("203.0.113.10", "https", CERTIFICATE_PEM, PRIVATE_KEY_PEM);
    let authorized = authorize_production_tls(&production).expect("tls");
    assert_eq!(authorized.decision(), BindDecision::ProductionTls);
    assert_eq!(
        authorize_orchestrator_live_port(&production)
            .expect("live")
            .decision(),
        BindDecision::ProductionTls
    );
    rustls_server_config(CERTIFICATE_PEM, PRIVATE_KEY_PEM).expect("config");

    let loopback_https = request("127.0.0.1", "https", CERTIFICATE_PEM, PRIVATE_KEY_PEM);
    assert_eq!(
        authorize_production_tls(&loopback_https)
            .expect("loopback tls")
            .decision(),
        BindDecision::ProductionTls
    );
    assert_eq!(
        authorize_orchestrator_live_port(&loopback_https)
            .expect("loopback live")
            .decision(),
        BindDecision::ProductionTls
    );

    let loopback = request("localhost", "http", "", "");
    assert_eq!(
        authorize_production_tls(&loopback).expect("dev").decision(),
        BindDecision::DevelopmentOnly
    );
    assert_eq!(
        authorize_orchestrator_live_port(&loopback),
        Err(TlsError::LoopbackIsNotProductionTls)
    );
    assert_eq!(
        authorize_production_tls(&request("localhost", "https", "", "")),
        Err(TlsError::MissingCertificateMaterial)
    );
}

#[test]
fn missing_or_mismatched_certificate_material_fails_closed() {
    assert!(matches!(
        rustls_server_config("", PRIVATE_KEY_PEM),
        Err(TlsError::MissingCertificateMaterial)
    ));
    assert!(matches!(
        rustls_server_config(CERTIFICATE_PEM, ""),
        Err(TlsError::MissingCertificateMaterial)
    ));
    assert!(matches!(
        rustls_server_config(
            "-----BEGIN CERTIFICATE-----\nnot-der\n-----END CERTIFICATE-----\n",
            PRIVATE_KEY_PEM
        ),
        Err(TlsError::InvalidCertificateMaterial)
    ));
    assert!(matches!(
        rustls_server_config(CERTIFICATE_PEM, OTHER_KEY_PEM),
        Err(TlsError::InvalidCertificateMaterial)
    ));
    assert!(matches!(
        rustls_server_config(CERTIFICATE_PEM, "not-a-key"),
        Err(TlsError::InvalidCertificateMaterial)
    ));
    assert!(matches!(
        rustls_server_config(
            CERTIFICATE_PEM,
            "-----BEGIN PRIVATE KEY-----\n@@@@\n-----END PRIVATE KEY-----\n"
        ),
        Err(TlsError::InvalidCertificateMaterial)
    ));
    assert_eq!(
        authorize_production_tls(&request("198.51.100.8", "https", "", "")),
        Err(TlsError::MissingCertificateMaterial)
    );
}

#[test]
fn bind_request_debug_redacts_certificate_and_private_key_pem() {
    let request = request("203.0.113.10", "https", CERTIFICATE_PEM, PRIVATE_KEY_PEM);
    let debug = format!("{request:?}");
    assert!(
        !debug.contains(CERTIFICATE_PEM),
        "certificate PEM must not appear in Debug"
    );
    assert!(
        !debug.contains(PRIVATE_KEY_PEM),
        "private key PEM must not appear in Debug"
    );
    assert!(
        !debug.contains("BEGIN CERTIFICATE"),
        "certificate PEM header must not appear in Debug"
    );
    assert!(
        !debug.contains("BEGIN PRIVATE KEY") && !debug.contains("BEGIN RSA PRIVATE KEY"),
        "private key PEM header must not appear in Debug"
    );
    assert!(debug.contains("<redacted>"), "PEM fields must be masked");
    assert!(
        debug.contains("203.0.113.10") && debug.contains("8443") && debug.contains("https"),
        "non-secret bind fields must remain visible"
    );
}

#[test]
fn recovered_tls_decisions_match_known_truth_better_than_a_collapsed_grant() {
    let cases = [
        (
            "203.0.113.10",
            "https",
            CERTIFICATE_PEM,
            PRIVATE_KEY_PEM,
            BindDecision::ProductionTls,
        ),
        ("localhost", "http", "", "", BindDecision::DevelopmentOnly),
        (
            "0.0.0.0",
            "http",
            CERTIFICATE_PEM,
            PRIVATE_KEY_PEM,
            BindDecision::Refused,
        ),
        (
            "postgres.example",
            "https",
            CERTIFICATE_PEM,
            PRIVATE_KEY_PEM,
            BindDecision::Refused,
        ),
        ("198.51.100.8", "https", "", "", BindDecision::Refused),
    ];
    let truth: Vec<BindDecision> = cases
        .iter()
        .map(|(_, _, _, _, expected)| *expected)
        .collect();
    let recovered: Vec<BindDecision> = cases
        .iter()
        .map(|(host, scheme, certificate_pem, private_key_pem, _)| {
            decided_production(host, scheme, certificate_pem, private_key_pem)
        })
        .collect();
    assert_eq!(recovered, truth);
    let collapsed = vec![BindDecision::ProductionTls; truth.len()];
    let recovered_rate = tls_policy_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = tls_policy_recovery_rate(&truth, &collapsed).expect("collapsed");
    assert!((recovered_rate - 1.0).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
    assert_eq!(
        decided_orchestrator("localhost", "http", "", ""),
        BindDecision::Refused
    );
    assert_eq!(
        decided_orchestrator("203.0.113.10", "https", CERTIFICATE_PEM, PRIVATE_KEY_PEM),
        BindDecision::ProductionTls
    );
    assert_eq!(
        tls_policy_recovery_rate(&truth, &truth[..2]),
        Err(TlsError::InvalidBindPayload)
    );
    assert_eq!(
        tls_policy_recovery_rate(&[], &[]),
        Err(TlsError::InvalidBindPayload)
    );
}
