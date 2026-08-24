# Encrypted identity mapping envelope (doctoring)

## Scope

`encrypted_mapping` seals a source identity behind a keyed `SHA-256` HMAC
envelope so ordinary analytical, log, and model-artifact purposes cannot
recover plaintext. Opening requires an explicit re-identification purpose
and the matching key. Recovery is the computed share of opened identities
that match known truth. The keystream nonce is derived inside the crate as
an HMAC synthetic value bound to the key, analytical identifier, and exact
identity bytes, so distinct identities never share keystream material and
callers cannot trigger two-time-pad reuse by repeating a nonce. Sealing is
deterministic: identical input reproduces an identical envelope, so equal
source identities are visible as equal envelopes while contents stay sealed.

This slice does not persist the mapping, allocate migration `0008`, operate
a KMS/HSM, or claim CSAP, SOC 2, FIPS validation, or legal sufficiency.

## Authority

### Normative TEPP contract

- `docs/adr/0009-purpose-bound-pii-governance.md` — purpose-bound
  authorization, opaque analytical identifiers, separately protected
  identity mapping, and encryption.
- `docs/PRIVACY_DATA_GOVERNANCE.md` — re-identification is a privileged
  operation distinct from analytical use.

### Supporting literature

HMAC-SHA-256 is the keyed message-authentication construction specified by
Krawczyk et al. (1997) and tested against Nystrom (2005). ISO/IEC 29100
treats purpose specification and use limitation as distinct privacy controls
(International Organization for Standardization & International
Electrotechnical Commission, 2011). They do **not** authorize substituting a
global mask for encryption, and they do not certify TEPP.

Krawczyk, H., Bellare, M., & Canetti, R. (1997). *HMAC: Keyed-hashing for
message authentication* (RFC 2104). RFC Editor.
https://doi.org/10.17487/RFC2104

Nystrom, M. (2005). *Identifiers and Test Vectors for HMAC-SHA-224,
HMAC-SHA-256, HMAC-SHA-384, and HMAC-SHA-512* (RFC 4231). RFC Editor.
https://doi.org/10.17487/RFC4231

International Organization for Standardization and International
Electrotechnical Commission. (2011). *Information technology—Security
techniques—Privacy framework* (ISO/IEC Standard No. 29100:2011).

## Verification

- RFC 4231 case 1 and case 6 HMAC-SHA-256 vectors match;
- sealed ciphertext does not contain the source identity bytes;
- analytical, log, and model-artifact purposes fail closed;
- wrong key identity, wrong key bytes, tampered ciphertext, tampered tags,
  and substituted nonces fail closed;
- distinct identities sealed under one key never share keystream material,
  including repeated analytical identifiers, so no caller-repeated value can
  reuse a keystream;
- recovered identities match known truth at a higher computed rate than
  collapsing every mapping to one name;
- persistence is refused until a later migration exists.
