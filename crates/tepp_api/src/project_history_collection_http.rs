//! Provider-owned project-history collection GET contracts.
//!
//! GAP-003A unique slice: `GET /v1/project-histories` enumerates accepted
//! cutoff-safe project-history projections on `AnalysisRunLiveService` /
//! `tepp-loopback` so operators do not guess idempotency keys. Collection
//! bodies stay metric-free and identity-opaque. `tepp.scientific_acceptance.v1`
//! never appears. The page does not include evidence text, findings, or a
//! causal score. This module does not duplicate project-history CLI (#420),
//! temporal-context CLI (#414), export CLI (#410), export-retrieval GET (#411),
//! analysis-run collection GET (#368), GET-by-id (#359), or GAP-010
//! Figma/export. Persistence remains GAP-003B.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target};
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{
    ApiError, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT, PROJECT_HISTORY_IDEMPOTENCY_KEY_MAX_LEN,
    PROJECT_HISTORY_PATH,
};
use serde::{Deserialize, Serialize};

/// Supported project-history collection contract version.
pub const PROJECT_HISTORY_COLLECTION_CONTRACT_VERSION: u16 = 1;

/// Default page size for loopback project-history collection GET.
pub const PROJECT_HISTORY_COLLECTION_DEFAULT_LIMIT: usize = 32;

/// Maximum page size accepted on loopback project-history collection GET.
pub const PROJECT_HISTORY_COLLECTION_MAX_LIMIT: usize = 64;

/// Maximum opaque cursor / idempotency-key length on the collection path.
pub const PROJECT_HISTORY_COLLECTION_CURSOR_MAX_LEN: usize =
    PROJECT_HISTORY_IDEMPOTENCY_KEY_MAX_LEN;

/// Fixed non-causal claim boundary echoed on every collection row.
pub const PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS: &str = "temporal_association_only";

const FORBIDDEN_COLLECTION_KEYS: [&str; 16] = [
    "rmse",
    "rmse_standard_error",
    "mean_bias",
    "bias_standard_error",
    "interval_coverage",
    "coverage_wilson_lower",
    "coverage_wilson_upper",
    "temporal_order_accuracy",
    "se_gate_accepted",
    "se_gate_k",
    "scientific_acceptance",
    "report",
    "terminal_result",
    "evidence_text",
    "findings",
    "causal_score",
];

/// One metric-free collection row for an accepted project-history projection.
///
/// The row names the durable project key and idempotency identity. It never
/// carries evidence text, findings, or scientific-acceptance artifacts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectHistoryCollectionItem {
    /// Consumer-owned stable project key.
    pub project_key: String,
    /// Exact request idempotency key that minted the stored projection.
    pub idempotency_key: String,
    /// Knowledge cutoff applied to the stored projection.
    pub knowledge_cutoff: String,
    /// Fixed claim boundary: sequence is association, not causation.
    pub inference_status: String,
}

impl ProjectHistoryCollectionItem {
    /// Construct a validated metric-free collection row.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities, an oversized
    /// idempotency key, or a causal inference status.
    pub fn new(
        project_key: impl Into<String>,
        idempotency_key: impl Into<String>,
        knowledge_cutoff: impl Into<String>,
        inference_status: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let item = Self {
            project_key: project_key.into(),
            idempotency_key: idempotency_key.into(),
            knowledge_cutoff: knowledge_cutoff.into(),
            inference_status: inference_status.into(),
        };
        item.validate()?;
        Ok(item)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_nonempty(&self.project_key)?;
        require_nonempty(&self.idempotency_key)?;
        require_nonempty(&self.knowledge_cutoff)?;
        if self.idempotency_key.len() > PROJECT_HISTORY_COLLECTION_CURSOR_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        if self.inference_status != PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }
}

/// Versioned metric-free project-history collection page.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectHistoryCollection {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Bounded page of metric-free rows, sorted by `idempotency_key`.
    pub histories: Vec<ProjectHistoryCollectionItem>,
    /// Exclusive cursor for the next page when more rows remain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl ProjectHistoryCollection {
    /// Construct a validated collection page.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error when a row is invalid, the page exceeds the
    /// maximum limit, or `next_cursor` is empty or oversized.
    pub fn new(
        histories: Vec<ProjectHistoryCollectionItem>,
        next_cursor: Option<String>,
    ) -> Result<Self, ApiError> {
        let collection = Self {
            contract_version: PROJECT_HISTORY_COLLECTION_CONTRACT_VERSION,
            histories,
            next_cursor,
        };
        collection.validate()?;
        Ok(collection)
    }

    /// Parse and validate a collection payload with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)
    }

    /// Parse and validate a collection payload with a caller-supplied limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        refuse_metrics_on_project_history_collection_payload(payload)?;
        let collection: Self = from_json(payload)?;
        collection.validate()?;
        Ok(collection)
    }

    /// Serialize this collection after complete validation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)?;
        refuse_metrics_on_project_history_collection_payload(&payload)?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(
            self.contract_version,
            PROJECT_HISTORY_COLLECTION_CONTRACT_VERSION,
        )?;
        if self.histories.len() > PROJECT_HISTORY_COLLECTION_MAX_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        for item in &self.histories {
            item.validate()?;
        }
        if let Some(cursor) = &self.next_cursor {
            require_nonempty(cursor)?;
            if cursor.len() > PROJECT_HISTORY_COLLECTION_CURSOR_MAX_LEN {
                return Err(ApiError::LimitExceeded);
            }
        }
        Ok(())
    }
}

/// Refuse collection JSON that already carries scientific-metric or evidence keys.
///
/// Empty payloads fail closed as valid request bodies. Non-object JSON fails
/// closed as invalid wire when nonempty.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric, evidence,
/// or causal-score key is present.
pub fn refuse_metrics_on_project_history_collection_payload(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    if payload.contains("tepp.scientific_acceptance.v1") {
        return Err(ApiError::InvalidWirePayload);
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    refuse_metrics_on_json(&value)
}

fn refuse_metrics_on_json(value: &serde_json::Value) -> Result<(), ApiError> {
    match value {
        serde_json::Value::Object(object) => {
            if FORBIDDEN_COLLECTION_KEYS
                .iter()
                .any(|key| object.contains_key(*key))
            {
                return Err(ApiError::InvalidWirePayload);
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

/// Parse the optional `tepp-page-limit` header.
///
/// Absent header uses [`PROJECT_HISTORY_COLLECTION_DEFAULT_LIMIT`]. Zero, a
/// non-integer, or a value above [`PROJECT_HISTORY_COLLECTION_MAX_LIMIT`] fail
/// closed.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-integer and
/// [`ApiError::LimitExceeded`] when the requested page is larger than the
/// maximum.
pub fn parse_project_history_collection_page_limit(raw: Option<&str>) -> Result<usize, ApiError> {
    let Some(raw) = raw else {
        return Ok(PROJECT_HISTORY_COLLECTION_DEFAULT_LIMIT);
    };
    let limit: usize = raw.parse().map_err(|_| ApiError::InvalidWirePayload)?;
    if limit == 0 {
        return Err(ApiError::InvalidWirePayload);
    }
    if limit > PROJECT_HISTORY_COLLECTION_MAX_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
    Ok(limit)
}

/// Parse the optional exclusive `tepp-page-cursor` header.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for an empty cursor and
/// [`ApiError::LimitExceeded`] when the cursor exceeds
/// [`PROJECT_HISTORY_COLLECTION_CURSOR_MAX_LEN`].
pub fn parse_project_history_collection_page_cursor(
    raw: Option<&str>,
) -> Result<Option<String>, ApiError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    require_nonempty(raw)?;
    if raw.len() > PROJECT_HISTORY_COLLECTION_CURSOR_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(Some(raw.to_owned()))
}

/// Return whether `path` is exactly the project-history collection resource.
#[must_use]
pub fn is_project_history_collection_path(path: &str) -> bool {
    path == PROJECT_HISTORY_PATH
}

/// Page stored rows after an exclusive cursor, sorted by idempotency key.
#[must_use]
pub fn page_project_history_collection_items(
    mut items: Vec<ProjectHistoryCollectionItem>,
    cursor: Option<&str>,
    limit: usize,
) -> (Vec<ProjectHistoryCollectionItem>, Option<String>) {
    items.sort_by(|left, right| left.idempotency_key.cmp(&right.idempotency_key));
    if let Some(cursor) = cursor {
        items.retain(|item| item.idempotency_key.as_str() > cursor);
    }
    let next_cursor = if items.len() > limit {
        Some(items[limit - 1].idempotency_key.clone())
    } else {
        None
    };
    items.truncate(limit);
    (items, next_cursor)
}

/// Build a provider-owned `GET` project-history collection exchange.
///
/// The builder refuses non-`https` origins and does not inject credentials.
/// Loopback pagination uses `tepp-page-cursor` and `tepp-page-limit` headers
/// because the shared request-line parser fails closed on query strings.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin or an
/// empty cursor, and [`ApiError::LimitExceeded`] when limit or cursor bounds
/// are exceeded.
pub fn lineageweave_project_history_collection_exchange(
    origin: &str,
    cursor: Option<&str>,
    limit: Option<&str>,
) -> Result<NaruonHttpExchange, ApiError> {
    let _ = parse_project_history_collection_page_limit(limit)?;
    let _ = parse_project_history_collection_page_cursor(cursor)?;
    let target_url = compose_https_target(origin, PROJECT_HISTORY_PATH)?;
    let mut headers = vec![
        ("content-type".into(), "application/json".into()),
        ("tepp-consumer".into(), "lineageweave".into()),
        ("tepp-contract-version".into(), "1".into()),
    ];
    if let Some(cursor) = cursor {
        headers.push(("tepp-page-cursor".into(), cursor.to_owned()));
    }
    if let Some(limit) = limit {
        headers.push(("tepp-page-limit".into(), limit.to_owned()));
    }
    Ok(NaruonHttpExchange {
        method: "GET",
        target_url,
        headers,
        body: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        PROJECT_HISTORY_COLLECTION_CURSOR_MAX_LEN, PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS,
        PROJECT_HISTORY_COLLECTION_MAX_LIMIT, ProjectHistoryCollection,
        ProjectHistoryCollectionItem, is_project_history_collection_path,
        lineageweave_project_history_collection_exchange, page_project_history_collection_items,
        parse_project_history_collection_page_cursor, parse_project_history_collection_page_limit,
        refuse_metrics_on_project_history_collection_payload,
    };
    use crate::ApiError;

    fn sample_item() -> ProjectHistoryCollectionItem {
        ProjectHistoryCollectionItem::new(
            "project",
            "idem-1",
            "2026-08-19T23:59:59Z",
            PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS,
        )
        .expect("item")
    }

    #[test]
    fn collection_round_trips_and_refuses_hostile_shapes() {
        let collection = ProjectHistoryCollection::new(vec![sample_item()], None).expect("page");
        let json = collection.to_json().expect("json");
        assert_eq!(
            ProjectHistoryCollection::from_json(&json).expect("decode"),
            collection
        );
        assert!(!json.contains("rmse"));
        assert!(!json.contains("scientific_acceptance"));
        assert!(!json.contains("evidence_text"));
        assert!(!json.contains("findings"));
        assert!(!json.contains("next_cursor"));
        assert!(!json.contains("causal_score"));

        assert_eq!(
            ProjectHistoryCollectionItem::new(
                "",
                "idem-1",
                "2026-08-19T23:59:59Z",
                PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ProjectHistoryCollectionItem::new(
                "project",
                "",
                "2026-08-19T23:59:59Z",
                PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ProjectHistoryCollectionItem::new(
                "project",
                "idem-1",
                "2026-08-19T23:59:59Z",
                "causal_score"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ProjectHistoryCollectionItem::new(
                "project",
                "a".repeat(PROJECT_HISTORY_COLLECTION_CURSOR_MAX_LEN + 1),
                "2026-08-19T23:59:59Z",
                PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS,
            ),
            Err(ApiError::LimitExceeded)
        );

        let mut unsupported = collection.clone();
        unsupported.contract_version = 9;
        assert_eq!(
            unsupported.to_json(),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            ProjectHistoryCollection::from_json(r#"{"contract_version":9,"histories":[]}"#),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            ProjectHistoryCollection::from_json(
                r#"{"contract_version":1,"histories":[],"extra":true}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ProjectHistoryCollection::from_json_with_limit(&json, 8),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            ProjectHistoryCollection::from_json("[1,2,3]"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ProjectHistoryCollection::new(vec![sample_item()], Some(String::new())),
            Err(ApiError::InvalidWirePayload)
        );
        let oversized = vec![sample_item(); PROJECT_HISTORY_COLLECTION_MAX_LIMIT + 1];
        assert_eq!(
            ProjectHistoryCollection::new(oversized, None),
            Err(ApiError::LimitExceeded)
        );
    }

    #[test]
    fn collection_payloads_refuse_scientific_metric_and_evidence_keys() {
        assert_eq!(
            refuse_metrics_on_project_history_collection_payload(""),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_project_history_collection_payload(r#"{"histories":[]}"#),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_project_history_collection_payload(r#"{"rmse":1.0}"#),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_project_history_collection_payload(
                r#"{"histories":[{"evidence_text":"secret"}]}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_project_history_collection_payload(r#"{"causal_score":1}"#),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_project_history_collection_payload(
                r#"{"schema_version":"tepp.scientific_acceptance.v1"}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert!(
            ProjectHistoryCollection::from_json(
                r#"{"contract_version":1,"histories":[],"scientific_acceptance":true}"#
            )
            .is_err()
        );
    }

    #[test]
    fn pagination_and_exchange_fail_closed() {
        assert_eq!(
            parse_project_history_collection_page_limit(None).expect("default"),
            32
        );
        assert_eq!(
            parse_project_history_collection_page_limit(Some("0")),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_project_history_collection_page_limit(Some("65")),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            parse_project_history_collection_page_cursor(Some("")),
            Err(ApiError::InvalidWirePayload)
        );
        assert!(is_project_history_collection_path("/v1/project-histories"));
        assert!(!is_project_history_collection_path("/v1/analysis-runs"));
        assert!(!is_project_history_collection_path("/v1/temporal-context"));

        let first = sample_item();
        let second = ProjectHistoryCollectionItem::new(
            "project-b",
            "idem-2",
            "2026-08-19T23:59:59Z",
            PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS,
        )
        .expect("second");
        let (page, cursor) =
            page_project_history_collection_items(vec![second.clone(), first.clone()], None, 1);
        assert_eq!(page, vec![first.clone()]);
        assert_eq!(cursor.as_deref(), Some("idem-1"));
        let (rest, done) =
            page_project_history_collection_items(vec![second.clone(), first], Some("idem-1"), 32);
        assert_eq!(rest, vec![second]);
        assert_eq!(done, None);

        let exchange = lineageweave_project_history_collection_exchange(
            "https://tepp.example.test",
            Some("idem-1"),
            Some("8"),
        )
        .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert!(exchange.target_url.ends_with("/v1/project-histories"));
        assert!(
            !exchange
                .headers
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case("authorization"))
        );
        assert!(exchange.body.is_empty());
        assert_eq!(
            lineageweave_project_history_collection_exchange("http://insecure.example", None, None),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
