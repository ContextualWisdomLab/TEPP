# Encrypted identity mapping envelope (doctoring)

## Scope

`encrypted_mapping` seals a source identity with an AES-256-GCM authenticated
encryption envelope. The operating system supplies a fresh 96-bit nonce for
each seal, and the analytical and key identifiers are authenticated as
associated data. Ordinary analytical, log, and model-artifact purposes cannot
recover plaintext. Opening requires an explicit re-identification purpose and
the matching key. Source identities are bounded to 1 MiB before encryption to
keep the trust boundary resource-bounded. Recovery is the computed share of
opened identities that match known truth.

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

AES-GCM is an authenticated-encryption construction specified by Dworkin
(2007). NIST SP 800-38D makes nonce/IV uniqueness a security requirement;
therefore the public sealing API does not accept a caller-provided nonce.
HMAC-SHA-256 remains only as the key-material normalization primitive, as
specified by Krawczyk et al. (1997) and tested against Nystrom (2005).
ISO/IEC 29100 treats purpose specification and use limitation as distinct
privacy controls (International Organization for Standardization &
International Electrotechnical Commission, 2011). These sources do **not**
certify TEPP.

Dworkin, M. (2007). *Recommendation for block cipher modes of operation:
Galois/Counter mode (GCM) and GMAC* (NIST Special Publication 800-38D).
National Institute of Standards and Technology.
https://doi.org/10.6028/NIST.SP.800-38D

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

- RFC 4231 case 1 and case 6 HMAC-SHA-256 vectors match for key-material
  normalization;
- AES-GCM ciphertext and tag round-trip with an OS-generated nonce;
- oversized source identities and unavailable randomness fail closed before
  encryption;
- changing the analytical identifier fails AEAD authentication because it is
  associated data;
- sealed ciphertext does not contain the source identity bytes;
- analytical, log, and model-artifact purposes fail closed;
- wrong key identity, wrong key bytes, tampered ciphertext, and tampered
  tags fail closed;
- recovered identities match known truth at a higher computed rate than
  collapsing every mapping to one name;
- persistence is refused until a later migration exists.
