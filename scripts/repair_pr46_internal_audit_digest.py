"""Move re-identification audit digest construction inside the TEPP trust boundary."""

from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    """Replace exactly one source fragment or fail closed."""
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)


cargo_path = Path("crates/tepp_api/Cargo.toml")
cargo = cargo_path.read_text(encoding="utf-8")
cargo = replace_once(
    cargo,
    "serde_json = { workspace = true }\ntemporal_core = { path = \"../temporal_core\" }\n",
    "serde_json = { workspace = true }\nsha2 = { workspace = true }\ntemporal_core = { path = \"../temporal_core\" }\n",
    "tepp_api sha2 dependency",
)
cargo_path.write_text(cargo, encoding="utf-8")

provider_path = Path("crates/tepp_api/src/provider_payload.rs")
provider = provider_path.read_text(encoding="utf-8")
provider = replace_once(
    provider,
    "use crate::wire::require_nonempty;\nuse std::fmt;\n",
    "use crate::wire::require_nonempty;\nuse sha2::{Digest, Sha256};\nuse std::fmt;\n",
    "provider digest imports",
)
provider = replace_once(
    provider,
    "    decision_time: &str,\n    decision_digest: &str,\n    audit_sink: &mut S,\n",
    "    decision_time: &str,\n    audit_sink: &mut S,\n",
    "re-identification signature",
)
provider = replace_once(
    provider,
    "    require_rfc3339_utc(decision_time)?;\n    require_sha256_digest(decision_digest)?;\n\n    let allowed =",
    "    require_rfc3339_utc(decision_time)?;\n\n    let allowed =",
    "caller-supplied digest validation",
)
provider = replace_once(
    provider,
    """    let audit_record = ReidentificationAuditRecord {
        tenant_workspace_id: grant.tenant_workspace_id.clone(),
        principal_id: grant.principal_id.clone(),
        purpose_wire_name: grant.purpose.wire_name().into(),
        action_code: \"reidentify_identity_mapping\",
        opaque_analytical_id: mapping.opaque_analytical_id.clone(),
        decision_time: decision_time.into(),
        outcome: if allowed {
            ReidentificationAuditOutcome::Allowed
        } else {
            ReidentificationAuditOutcome::Denied
        },
        decision_digest: decision_digest.into(),
    };
""",
    """    let outcome = if allowed {
        ReidentificationAuditOutcome::Allowed
    } else {
        ReidentificationAuditOutcome::Denied
    };
    let audit_record = ReidentificationAuditRecord {
        tenant_workspace_id: grant.tenant_workspace_id.clone(),
        principal_id: grant.principal_id.clone(),
        purpose_wire_name: grant.purpose.wire_name().into(),
        action_code: \"reidentify_identity_mapping\",
        opaque_analytical_id: mapping.opaque_analytical_id.clone(),
        decision_time: decision_time.into(),
        outcome,
        decision_digest: reidentification_decision_digest(
            grant,
            mapping,
            decision_time,
            outcome,
        )?,
    };
""",
    "audit record construction",
)
old_digest_validator = """fn require_sha256_digest(value: &str) -> Result<(), ApiError> {
    let Some(digest) = value.strip_prefix(\"sha256:\") else {
        return Err(ApiError::InvalidWirePayload);
    };
    if digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

"""
new_digest_builder = """const REIDENTIFICATION_AUDIT_DIGEST_VERSION: &str = \"tepp.reidentification.audit.v1\";

fn reidentification_decision_digest(
    grant: &PurposeGrant,
    mapping: &IdentityMappingRecord,
    decision_time: &str,
    outcome: ReidentificationAuditOutcome,
) -> Result<String, ApiError> {
    let mut hasher = Sha256::new();
    for value in [
        REIDENTIFICATION_AUDIT_DIGEST_VERSION,
        \"reidentify_identity_mapping\",
        &grant.tenant_workspace_id,
        &grant.principal_id,
        grant.purpose.wire_name(),
        &grant.valid_from,
        if grant.valid_to.is_some() { \"1\" } else { \"0\" },
        grant.valid_to.as_deref().unwrap_or(\"\"),
        if grant.reidentification_authorized { \"1\" } else { \"0\" },
        &mapping.tenant_workspace_id,
        &mapping.opaque_analytical_id,
        &mapping.direct_identity,
        decision_time,
        outcome.wire_name(),
    ] {
        update_audit_digest_field(&mut hasher, value)?;
    }
    Ok(format!(\"sha256:{:x}\", hasher.finalize()))
}

fn update_audit_digest_field(hasher: &mut Sha256, value: &str) -> Result<(), ApiError> {
    let length = u64::try_from(value.len()).map_err(|_| ApiError::LimitExceeded)?;
    hasher.update(length.to_be_bytes());
    hasher.update(value.as_bytes());
    Ok(())
}

"""
provider = replace_once(
    provider,
    old_digest_validator,
    new_digest_builder,
    "internal audit digest builder",
)
provider = provider.replace(
    "            decision_time,\n            \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\n            &mut audit_sink,\n",
    "            decision_time,\n            &mut audit_sink,\n",
)
provider_path.write_text(provider, encoding="utf-8")

provider_contract_path = Path("crates/tepp_api/tests/provider_payload_contract.rs")
provider_contract = provider_contract_path.read_text(encoding="utf-8")
provider_contract = replace_once(
    provider_contract,
    """        decision_time,
        \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",
        &mut sink,
""",
    """        decision_time,
        &mut sink,
""",
    "provider contract disclosure helper",
)
provider_contract_path.write_text(provider_contract, encoding="utf-8")

denial_path = Path("crates/tepp_api/tests/reidentification_audit_denial_matrix.rs")
denial = denial_path.read_text(encoding="utf-8")
denial = denial.replace(
    "const DECISION_DIGEST: &str =\n    \"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\";\n\n",
    "",
)
denial = replace_once(
    denial,
    "        disclose_identity_mapping(grant, mapping, decision_time, DECISION_DIGEST, sink),\n",
    "        disclose_identity_mapping(grant, mapping, decision_time, sink),\n",
    "denial matrix disclosure call",
)
denial = replace_once(
    denial,
    "    assert_eq!(record.decision_digest(), DECISION_DIGEST);\n",
    "    assert!(record.decision_digest().starts_with(\"sha256:\"));\n    assert_eq!(record.decision_digest().len(), 71);\n",
    "denial matrix digest assertion",
)
denial_path.write_text(denial, encoding="utf-8")

research_path = Path("docs/research/provider-payload-minimization.md")
research = research_path.read_text(encoding="utf-8").rstrip()
research += """

## Re-identification audit digest authority

The caller cannot provide or select the decision digest. TEPP computes a
versioned, length-delimited SHA-256 digest inside the trust boundary from the
purpose grant, protected mapping, decision instant, and allow/deny outcome.
Direct identity contributes to the digest but never appears in the redacted
audit record or ordinary logs. This makes append-only replay evidence bind the
actual governed decision rather than an arbitrary caller assertion.
"""
research_path.write_text(research.rstrip() + "\n", encoding="utf-8")
