# Production TLS bind gates (doctoring)

## Scope

`service_tls` classifies bind hosts as loopback development or production
TLS. Non-loopback binds require `https` and rustls PEM material. Loopback
HTTP is development-only. An orchestrator live production port cannot be
loopback plaintext. Table-access host labels fail closed. Recovery is the
computed share of `authorize_production_tls` and
`authorize_orchestrator_live_port` decisions that match known truth.

This slice does not terminate a public listener, issue certificates, or
claim TLS deployment, CSAP, SOC 2, or legal sufficiency.

## Authority

### Normative TEPP contract

- `docs/adr/0011-standalone-modular-msa-boundary.md` — standalone/modular
  service ports without cross-service table access.
- `docs/API_CONTRACT.md` — orchestrator live production binds use
  `authorize_orchestrator_live_port`.

### Supporting literature

TLS 1.3 is the IETF transport-security protocol used here as the rustls
configuration target (Rescorla, 2018). It does **not** certify TEPP
deployment.

NIST SP 800-52 Revision 2 recommends TLS for protecting network
connections (McKay & Cooper, 2019). TEPP policy, not that publication,
refuses plaintext production service binds.

Rescorla, E. (2018). *The transport layer security (TLS) protocol version
1.3* (RFC 8446). Internet Engineering Task Force.
https://doi.org/10.17487/RFC8446

McKay, K. A., & Cooper, D. A. (2019). *Guidelines for the selection,
configuration, and use of Transport Layer Security (TLS) implementations*
(NIST Special Publication 800-52 Revision 2). National Institute of
Standards and Technology. https://doi.org/10.6028/NIST.SP.800-52r2
