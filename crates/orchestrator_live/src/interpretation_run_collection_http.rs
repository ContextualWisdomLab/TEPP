//! Provider-owned interpretation-run collection GET contracts.
//!
//! GAP-003A unique slice: `GET /v1/interpretation-runs` enumerates accepted
//! hypothetical interpretation runs on `OrchestratorLiveService` /
//! `tepp-orchestrator-loopback` so operators do not guess idempotency keys.
//! Collection rows stay metric-free identities with `claim_status=hypothetical`
//! and `scientific_authority=false`. `tepp.scientific_acceptance.v1` never
//! appears. The page does not infer causality or call a model provider. This
//! module does not duplicate interpretation-run CLI (#425), project-history
//! collection GET (#424), collection CLI (#428), GET-by-id (#429), retrieval
//! CLI (#431), analysis-run collection GET (#368), Leiden, or GAP-010
//! Figma/export. Persistence remains GAP-003B. Naruon and `LineageWeave` are
//! refused. `NaruonLiveService` stays POST-only.

use serde::{Deserialize, Serialize};

use crate::error::OrchestratorLiveError;
use crate::interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
use crate::mode::OrchestrationMode;
use crate::request::{
    DEFAULT_INTERPRETATION_BYTE_LIMIT, HYPOTHETICAL_CLAIM_STATUS,
    INTERPRETATION_RUN_CONTRACT_VERSION, INTERPRETATION_RUN_PATH, from_json,
    host_implies_table_access, require_byte_limit, require_contract_version, require_nonempty,
    to_json,
};

/// Default page size for loopback interpretation-run collection GET.
pub const INTERPRETATION_RUN_COLLECTION_DEFAULT_LIMIT: usize = 32;

/// Maximum page size accepted on loopback interpretation-run collection GET.
pub const INTERPRETATION_RUN_COLLECTION_MAX_LIMIT: usize = 64;

/// Maximum opaque cursor / idempotency-key length on the collection path.
pub const INTERPRETATION_RUN_COLLECTION_CURSOR_MAX_LEN: usize = 128;

const FORBIDDEN_COLLECTION_KEYS: [&str; 14] = [
    "rmse",
    "rmse_standard_error",
    "mean_bias",
    "bias_standard_error",
    "interval_coverage",
    "se_gate_accepted",
    "scientific_acceptance",
    "evidence_span_ids",
    "tenant_workspace_id",
    "compute_budget_tokens",
    "causal_score",
    "findings",
    "evidence_text",
    "report",
];

/// One metric-free collection row for an accepted interpretation run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterpretationRunCollectionItem {
    /// Server-assigned opaque interpretation-run identity.
    pub interpretation_run_id: String,
    /// Exact request idempotency key that minted the stored run.
    pub idempotency_key: String,
    /// Selected orchestration mode.
    pub orchestration_mode: OrchestrationMode,
    /// Fixed claim boundary: accepted output is hypothetical.
    pub claim_status: String,
    /// Always `false`; LLM output is never scientific authority.
    pub scientific_authority: bool,
}

impl InterpretationRunCollectionItem {
    /// Construct a validated metric-free collection row.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities or a non-hypothetical
    /// claim.
    pub fn new(
        interpretation_run_id: impl Into<String>,
        idempotency_key: impl Into<String>,
        orchestration_mode: OrchestrationMode,
        claim_status: impl Into<String>,
        scientific_authority: bool,
    ) -> Result<Self, OrchestratorLiveError> {
        let item = Self {
            interpretation_run_id: interpretation_run_id.into(),
            idempotency_key: idempotency_key.into(),
            orchestration_mode,
            claim_status: claim_status.into(),
            scientific_authority,
        };
        item.validate()?;
        Ok(item)
    }

    fn validate(&self) -> Result<(), OrchestratorLiveError> {
        require_nonempty(&self.interpretation_run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.idempotency_key.len() > INTERPRETATION_RUN_COLLECTION_CURSOR_MAX_LEN {
            return Err(OrchestratorLiveError::LimitExceeded);
        }
        if self.claim_status != HYPOTHETICAL_CLAIM_STATUS || self.scientific_authority {
            return Err(OrchestratorLiveError::ScientificAuthorityRefused);
        }
        Ok(())
    }
}

/// Metric-free interpretation-run collection page.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterpretationRunCollection {
    /// Semantic contract version.
    pub contract_version: u16,
    /// Metric-free rows on this page.
    pub items: Vec<InterpretationRunCollectionItem>,
    /// Exclusive cursor for the next page, when more rows exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl InterpretationRunCollection {
    /// Construct a validated collection page.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for an oversized page or a hostile cursor.
    pub fn new(
        items: Vec<InterpretationRunCollectionItem>,
        next_cursor: Option<String>,
    ) -> Result<Self, OrchestratorLiveError> {
        if items.len() > INTERPRETATION_RUN_COLLECTION_MAX_LIMIT {
            return Err(OrchestratorLiveError::LimitExceeded);
        }
        for item in &items {
            item.validate()?;
        }
        if let Some(cursor) = next_cursor.as_deref() {
            parse_interpretation_run_collection_page_cursor(Some(cursor))?;
        }
        Ok(Self {
            contract_version: INTERPRETATION_RUN_CONTRACT_VERSION,
            items,
            next_cursor,
        })
    }

    /// Parse a collection page.
    ///
    /// # Errors
    ///
    /// Returns a size, JSON, version, or claim-boundary error.
    pub fn from_json(payload: &str) -> Result<Self, OrchestratorLiveError> {
        require_byte_limit(payload, DEFAULT_INTERPRETATION_BYTE_LIMIT)?;
        refuse_metrics_on_interpretation_run_collection_payload(payload)?;
        let collection: Self = from_json(payload)?;
        require_contract_version(
            collection.contract_version,
            INTERPRETATION_RUN_CONTRACT_VERSION,
        )?;
        InterpretationRunCollection::new(collection.items, collection.next_cursor)
    }

    /// Serialize a validated collection page.
    ///
    /// # Errors
    ///
    /// Returns a validation or serialization error.
    pub fn to_json(&self) -> Result<String, OrchestratorLiveError> {
        require_contract_version(self.contract_version, INTERPRETATION_RUN_CONTRACT_VERSION)?;
        InterpretationRunCollection::new(self.items.clone(), self.next_cursor.clone())?;
        let payload = to_json(self)?;
        refuse_metrics_on_interpretation_run_collection_payload(&payload)?;
        Ok(payload)
    }
}

/// Typed GET exchange for interpretation-run collection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpretationRunCollectionHttpExchange {
    /// HTTP method, always `GET`.
    pub method: &'static str,
    /// Absolute HTTPS target ending in [`INTERPRETATION_RUN_PATH`].
    pub target_url: String,
    /// Exact version, consumer, and content headers. No credentials.
    pub headers: Vec<(String, String)>,
    /// GET body, always empty.
    pub body: String,
}

/// Whether a path is the interpretation-run collection resource.
#[must_use]
pub fn is_interpretation_run_collection_path(path: &str) -> bool {
    path == INTERPRETATION_RUN_PATH
}

/// Parse the optional `tepp-page-limit` header.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] for a non-integer or
/// zero limit, and [`OrchestratorLiveError::LimitExceeded`] above the maximum.
pub fn parse_interpretation_run_collection_page_limit(
    raw: Option<&str>,
) -> Result<usize, OrchestratorLiveError> {
    let Some(raw) = raw else {
        return Ok(INTERPRETATION_RUN_COLLECTION_DEFAULT_LIMIT);
    };
    require_nonempty(raw)?;
    let limit: usize = raw
        .parse()
        .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    if limit == 0 {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if limit > INTERPRETATION_RUN_COLLECTION_MAX_LIMIT {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    Ok(limit)
}

/// Parse the optional exclusive `tepp-page-cursor` header.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] for an empty cursor
/// and [`OrchestratorLiveError::LimitExceeded`] when oversized.
pub fn parse_interpretation_run_collection_page_cursor(
    raw: Option<&str>,
) -> Result<Option<String>, OrchestratorLiveError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    require_nonempty(raw)?;
    if raw.contains('/') || raw.contains('\0') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if raw.len() > INTERPRETATION_RUN_COLLECTION_CURSOR_MAX_LEN {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    Ok(Some(raw.to_owned()))
}

/// Page stored collection rows with an exclusive idempotency-key cursor.
#[must_use]
pub fn page_interpretation_run_collection_items(
    mut items: Vec<InterpretationRunCollectionItem>,
    cursor: Option<&str>,
    limit: usize,
) -> (Vec<InterpretationRunCollectionItem>, Option<String>) {
    items.sort_by(|left, right| left.idempotency_key.cmp(&right.idempotency_key));
    let start = cursor.map_or(0, |cursor| {
        items
            .iter()
            .position(|item| item.idempotency_key.as_str() > cursor)
            .unwrap_or(items.len())
    });
    let end = (start + limit).min(items.len());
    let next_cursor = if end < items.len() {
        Some(items[end - 1].idempotency_key.clone())
    } else {
        None
    };
    (items[start..end].to_vec(), next_cursor)
}

/// Refuse metric, evidence, and causal-score keys on collection JSON.
///
/// Empty payloads are admitted for the GET request body.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when a forbidden key
/// or `tepp.scientific_acceptance.v1` appears, or nonempty JSON is not an
/// object.
pub fn refuse_metrics_on_interpretation_run_collection_payload(
    payload: &str,
) -> Result<(), OrchestratorLiveError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    if payload.contains("tepp.scientific_acceptance.v1") {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    if !value.is_object() {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    refuse_metrics_on_json(&value)
}

fn refuse_metrics_on_json(value: &serde_json::Value) -> Result<(), OrchestratorLiveError> {
    match value {
        serde_json::Value::Object(object) => {
            if FORBIDDEN_COLLECTION_KEYS
                .iter()
                .any(|key| object.contains_key(*key))
            {
                return Err(OrchestratorLiveError::InvalidWirePayload);
            }
            for nested in object.values() {
                refuse_metrics_on_json(nested)?;
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for nested in items {
                refuse_metrics_on_json(nested)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Build a credential-free contextual-orchestrator collection GET exchange.
///
/// # Errors
///
/// Returns a fail-closed origin or pagination error.
pub fn contextual_orchestrator_interpretation_run_collection_exchange(
    origin: &str,
    page_cursor: Option<&str>,
    page_limit: Option<&str>,
) -> Result<InterpretationRunCollectionHttpExchange, OrchestratorLiveError> {
    require_nonempty(origin)?;
    if !origin.starts_with("https://") || origin.ends_with('/') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let rest = origin
        .strip_prefix("https://")
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    if rest.contains('@') || rest.contains('?') || rest.contains('#') || rest.contains('\\') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if host_implies_table_access(rest) {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    parse_interpretation_run_collection_page_cursor(page_cursor)?;
    parse_interpretation_run_collection_page_limit(page_limit)?;
    let mut headers = vec![
        ("content-type".into(), "application/json".into()),
        (
            "tepp-consumer".into(),
            CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE.into(),
        ),
        ("tepp-contract-version".into(), "1".into()),
    ];
    if let Some(cursor) = page_cursor {
        headers.push(("tepp-page-cursor".into(), cursor.to_owned()));
    }
    if let Some(limit) = page_limit {
        headers.push(("tepp-page-limit".into(), limit.to_owned()));
    }
    Ok(InterpretationRunCollectionHttpExchange {
        method: "GET",
        target_url: format!("{origin}{INTERPRETATION_RUN_PATH}"),
        headers,
        body: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        INTERPRETATION_RUN_COLLECTION_CURSOR_MAX_LEN, INTERPRETATION_RUN_COLLECTION_MAX_LIMIT,
        InterpretationRunCollection, InterpretationRunCollectionItem,
        contextual_orchestrator_interpretation_run_collection_exchange,
        is_interpretation_run_collection_path, page_interpretation_run_collection_items,
        parse_interpretation_run_collection_page_cursor,
        parse_interpretation_run_collection_page_limit,
        refuse_metrics_on_interpretation_run_collection_payload,
    };
    use crate::error::OrchestratorLiveError;
    use crate::mode::OrchestrationMode;
    use crate::request::INTERPRETATION_RUN_PATH;

    fn sample_item(id: &str, idem: &str) -> InterpretationRunCollectionItem {
        InterpretationRunCollectionItem::new(
            id,
            idem,
            OrchestrationMode::Direct,
            "hypothetical",
            false,
        )
        .expect("item")
    }

    #[test]
    fn collection_exchange_is_metric_free_get_without_credentials() {
        let exchange = contextual_orchestrator_interpretation_run_collection_exchange(
            "https://tepp.example.test",
            Some("idem-a"),
            Some("8"),
        )
        .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert!(exchange.target_url.ends_with(INTERPRETATION_RUN_PATH));
        assert!(exchange.body.is_empty());
        assert!(
            !exchange
                .headers
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
                    || name.eq_ignore_ascii_case("idempotency-key"))
        );
        assert!(is_interpretation_run_collection_path(
            INTERPRETATION_RUN_PATH
        ));
        assert!(!is_interpretation_run_collection_path(
            "/v1/interpretation-runs/extra"
        ));
        let json =
            InterpretationRunCollection::new(vec![sample_item("orch-run-1", "idem-a")], None)
                .expect("page")
                .to_json()
                .expect("json");
        assert!(!json.contains("rmse"));
        assert!(!json.contains("evidence_span_ids"));
        assert!(!json.contains("tepp.scientific_acceptance.v1"));
        InterpretationRunCollection::from_json(&json).expect("roundtrip");
    }

    #[test]
    fn collection_payloads_and_origins_fail_closed() {
        assert_eq!(
            refuse_metrics_on_interpretation_run_collection_payload(""),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_interpretation_run_collection_payload(r#"{"rmse":1.0}"#),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_interpretation_run_collection_payload(r#"{"causal_score":1}"#),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_interpretation_run_collection_payload(
                r#"{"schema_version":"tepp.scientific_acceptance.v1"}"#
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_interpretation_run_collection_payload("[1]"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_collection_exchange(
                "http://insecure.example",
                None,
                None
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_collection_exchange(
                "https://user:pass@tepp.example.test",
                None,
                None
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_collection_exchange(
                "https://postgres.example.test",
                None,
                None
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
    }

    #[test]
    fn collection_pagination_and_claim_boundary_fail_closed() {
        assert_eq!(
            parse_interpretation_run_collection_page_limit(Some("0")),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            parse_interpretation_run_collection_page_limit(Some("nope")),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            parse_interpretation_run_collection_page_limit(Some(
                &(INTERPRETATION_RUN_COLLECTION_MAX_LIMIT + 1).to_string()
            )),
            Err(OrchestratorLiveError::LimitExceeded)
        );
        assert_eq!(
            parse_interpretation_run_collection_page_cursor(Some("idem/slash")),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            parse_interpretation_run_collection_page_cursor(Some("idem\0nul")),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            parse_interpretation_run_collection_page_cursor(Some(
                &"k".repeat(INTERPRETATION_RUN_COLLECTION_CURSOR_MAX_LEN + 1)
            )),
            Err(OrchestratorLiveError::LimitExceeded)
        );
        assert_eq!(
            InterpretationRunCollectionItem::new(
                " ",
                "idem-a",
                OrchestrationMode::Direct,
                "hypothetical",
                false,
            )
            .expect_err("empty id"),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCollectionItem::new(
                "orch-run-1",
                "idem-a",
                OrchestrationMode::Direct,
                "accepted",
                false,
            )
            .expect_err("claim"),
            OrchestratorLiveError::ScientificAuthorityRefused
        );
        assert_eq!(
            InterpretationRunCollectionItem::new(
                "orch-run-1",
                "idem-a",
                OrchestrationMode::Direct,
                "hypothetical",
                true,
            )
            .expect_err("authority"),
            OrchestratorLiveError::ScientificAuthorityRefused
        );
        let first = sample_item("orch-run-1", "idem-a");
        let second = sample_item("orch-run-2", "idem-b");
        let (page, next) =
            page_interpretation_run_collection_items(vec![second.clone(), first.clone()], None, 1);
        assert_eq!(page, vec![first.clone()]);
        assert_eq!(next.as_deref(), Some("idem-a"));
        let (rest, done) = page_interpretation_run_collection_items(
            vec![first.clone(), second.clone()],
            Some("idem-a"),
            1,
        );
        assert_eq!(rest, vec![second.clone()]);
        assert_eq!(done, None);
        InterpretationRunCollection::new(vec![first], Some("idem-a".into())).expect("page");
        assert_eq!(
            InterpretationRunCollection::from_json(r#"{"contract_version":9,"items":[]}"#)
                .expect_err("version"),
            OrchestratorLiveError::UnsupportedContractVersion
        );
    }
}
