//! Provider-owned analysis-run collection GET contracts.
//!
//! GAP-003A seventh slice: `GET /v1/analysis-runs` enumerates accepted,
//! running, cancelled, and terminal runs on the shared loopback listener so
//! operators do not guess run identities. Collection bodies stay metric-free.
//! `tepp.scientific_acceptance.v1` never appears on the list. GET-by-id (#359),
//! lifecycle POST (#360), cancel HTTP (#361), and loopback CLI (#362) remain
//! other live slices. Persistence remains GAP-003B.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target};
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{
    ANALYSIS_RUN_STATUS_PATH, AnalysisRunStatusState, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT,
};
use serde::{Deserialize, Serialize};

/// Supported analysis-run collection contract version.
pub const ANALYSIS_RUN_COLLECTION_CONTRACT_VERSION: u16 = 1;

/// Default page size for loopback collection GET.
pub const ANALYSIS_RUN_COLLECTION_DEFAULT_LIMIT: usize = 32;

/// Maximum page size accepted on loopback collection GET.
pub const ANALYSIS_RUN_COLLECTION_MAX_LIMIT: usize = 64;

/// Maximum opaque cursor / run-identity length on the collection path.
pub const ANALYSIS_RUN_COLLECTION_CURSOR_MAX_LEN: usize = 128;

const FORBIDDEN_COLLECTION_KEYS: [&str; 13] = [
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
];

/// One metric-free collection row for an analysis run.
///
/// The row names the durable identity and lifecycle state. It never carries a
/// terminal result or scientific-acceptance artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunCollectionItem {
    /// Opaque server-assigned run identity.
    pub run_id: String,
    /// Current lifecycle state.
    pub run_state: AnalysisRunStatusState,
    /// Exact request idempotency key.
    pub idempotency_key: String,
}

impl AnalysisRunCollectionItem {
    /// Construct a validated metric-free collection row.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities or an oversized run
    /// identity.
    pub fn new(
        run_id: impl Into<String>,
        run_state: AnalysisRunStatusState,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let item = Self {
            run_id: run_id.into(),
            run_state,
            idempotency_key: idempotency_key.into(),
        };
        item.validate()?;
        Ok(item)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.run_id.len() > ANALYSIS_RUN_COLLECTION_CURSOR_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        Ok(())
    }
}

/// Versioned metric-free analysis-run collection page.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunCollection {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Bounded page of metric-free run rows, sorted by `run_id`.
    pub runs: Vec<AnalysisRunCollectionItem>,
    /// Exclusive cursor for the next page when more rows remain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl AnalysisRunCollection {
    /// Construct a validated collection page.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error when a row is invalid, the page exceeds the
    /// maximum limit, or `next_cursor` is empty or oversized.
    pub fn new(
        runs: Vec<AnalysisRunCollectionItem>,
        next_cursor: Option<String>,
    ) -> Result<Self, ApiError> {
        let collection = Self {
            contract_version: ANALYSIS_RUN_COLLECTION_CONTRACT_VERSION,
            runs,
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
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
    }

    /// Parse and validate a collection payload with a caller-supplied limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        refuse_metrics_on_collection_payload(payload)?;
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
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_collection_payload(&payload)?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(
            self.contract_version,
            ANALYSIS_RUN_COLLECTION_CONTRACT_VERSION,
        )?;
        if self.runs.len() > ANALYSIS_RUN_COLLECTION_MAX_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        for item in &self.runs {
            item.validate()?;
        }
        if let Some(cursor) = &self.next_cursor {
            require_nonempty(cursor)?;
            if cursor.len() > ANALYSIS_RUN_COLLECTION_CURSOR_MAX_LEN {
                return Err(ApiError::LimitExceeded);
            }
        }
        Ok(())
    }
}

/// Refuse collection JSON that already carries scientific-metric keys.
///
/// Empty payloads fail closed: collection GET has an empty request body and a
/// nonempty object response. Non-object JSON fails closed as invalid wire.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric key is
/// present or the payload is a non-object.
pub fn refuse_metrics_on_collection_payload(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
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
/// Absent header uses [`ANALYSIS_RUN_COLLECTION_DEFAULT_LIMIT`]. Zero, a
/// non-integer, or a value above [`ANALYSIS_RUN_COLLECTION_MAX_LIMIT`] fail
/// closed.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-integer and
/// [`ApiError::LimitExceeded`] when the requested page is larger than the
/// maximum.
pub fn parse_collection_page_limit(raw: Option<&str>) -> Result<usize, ApiError> {
    let Some(raw) = raw else {
        return Ok(ANALYSIS_RUN_COLLECTION_DEFAULT_LIMIT);
    };
    let limit: usize = raw.parse().map_err(|_| ApiError::InvalidWirePayload)?;
    if limit == 0 {
        return Err(ApiError::InvalidWirePayload);
    }
    if limit > ANALYSIS_RUN_COLLECTION_MAX_LIMIT {
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
/// [`ANALYSIS_RUN_COLLECTION_CURSOR_MAX_LEN`].
pub fn parse_collection_page_cursor(raw: Option<&str>) -> Result<Option<String>, ApiError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    require_nonempty(raw)?;
    if raw.len() > ANALYSIS_RUN_COLLECTION_CURSOR_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(Some(raw.to_owned()))
}

/// Return whether `path` is exactly the analysis-run collection resource.
#[must_use]
pub fn is_analysis_run_collection_path(path: &str) -> bool {
    path == ANALYSIS_RUN_STATUS_PATH
}

/// Build a provider-owned `GET` analysis-run collection exchange.
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
pub fn naruon_analysis_run_collection_exchange(
    origin: &str,
    cursor: Option<&str>,
    limit: Option<&str>,
) -> Result<NaruonHttpExchange, ApiError> {
    let _ = parse_collection_page_limit(limit)?;
    let _ = parse_collection_page_cursor(cursor)?;
    let target_url = compose_https_target(origin, ANALYSIS_RUN_STATUS_PATH)?;
    let mut headers = vec![
        ("content-type".into(), "application/json".into()),
        ("tepp-consumer".into(), "naruon".into()),
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
    use super::*;

    fn sample_item() -> AnalysisRunCollectionItem {
        AnalysisRunCollectionItem::new("tepp-run-1", AnalysisRunStatusState::Accepted, "idem-1")
            .expect("item")
    }

    #[test]
    fn collection_round_trips_and_refuses_hostile_shapes() {
        let collection = AnalysisRunCollection::new(vec![sample_item()], None).expect("page");
        let json = collection.to_json().expect("json");
        assert_eq!(
            AnalysisRunCollection::from_json(&json).expect("decode"),
            collection
        );
        assert!(!json.contains("rmse"));
        assert!(!json.contains("scientific_acceptance"));
        assert!(!json.contains("terminal_result"));
        assert!(!json.contains("next_cursor"));

        assert_eq!(
            AnalysisRunCollectionItem::new("", AnalysisRunStatusState::Accepted, "idem-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCollectionItem::new("tepp-run-1", AnalysisRunStatusState::Accepted, ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCollectionItem::new(
                "a".repeat(ANALYSIS_RUN_COLLECTION_CURSOR_MAX_LEN + 1),
                AnalysisRunStatusState::Accepted,
                "idem-1",
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
            AnalysisRunCollection::from_json(r#"{"contract_version":9,"runs":[]}"#),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunCollection::from_json(r#"{"contract_version":1,"runs":[],"extra":true}"#),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCollection::from_json_with_limit(&json, 8),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            AnalysisRunCollection::from_json("[1,2,3]"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCollection::new(vec![sample_item()], Some(String::new())),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCollection::new(
                vec![sample_item()],
                Some("a".repeat(ANALYSIS_RUN_COLLECTION_CURSOR_MAX_LEN + 1)),
            ),
            Err(ApiError::LimitExceeded)
        );
        let oversized = vec![sample_item(); ANALYSIS_RUN_COLLECTION_MAX_LIMIT + 1];
        assert_eq!(
            AnalysisRunCollection::new(oversized, None),
            Err(ApiError::LimitExceeded)
        );
        let with_cursor =
            AnalysisRunCollection::new(vec![sample_item()], Some("tepp-run-1".into()))
                .expect("cursor page");
        assert!(
            with_cursor
                .to_json()
                .expect("cursor json")
                .contains("next_cursor")
        );
    }

    #[test]
    fn collection_payloads_refuse_scientific_metric_keys() {
        assert_eq!(refuse_metrics_on_collection_payload(""), Ok(()));
        assert_eq!(refuse_metrics_on_collection_payload("   "), Ok(()));
        assert_eq!(
            refuse_metrics_on_collection_payload(r#"{"runs":[]}"#),
            Ok(())
        );
        for key in FORBIDDEN_COLLECTION_KEYS {
            let payload = format!(r#"{{"{key}":1,"runs":[]}}"#);
            assert_eq!(
                refuse_metrics_on_collection_payload(&payload),
                Err(ApiError::InvalidWirePayload),
                "key={key}"
            );
            let nested = format!(r#"{{"contract_version":1,"runs":[{{"{key}":0}}]}}"#);
            assert_eq!(
                refuse_metrics_on_collection_payload(&nested),
                Err(ApiError::InvalidWirePayload),
                "nested key={key}"
            );
        }
        assert_eq!(
            AnalysisRunCollection::from_json(
                r#"{"contract_version":1,"runs":[],"scientific_acceptance":{}}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn collection_path_is_exact_and_page_headers_are_bounded() {
        assert!(is_analysis_run_collection_path("/v1/analysis-runs"));
        assert!(!is_analysis_run_collection_path("/v1/analysis-runs/"));
        assert!(!is_analysis_run_collection_path(
            "/v1/analysis-runs/tepp-run-1"
        ));
        assert!(!is_analysis_run_collection_path(
            "/v1/analysis-runs/tepp-run-1/cancel"
        ));
        assert_eq!(parse_collection_page_limit(None).expect("default"), 32);
        assert_eq!(parse_collection_page_limit(Some("1")).expect("one"), 1);
        assert_eq!(parse_collection_page_limit(Some("64")).expect("max"), 64);
        assert_eq!(
            parse_collection_page_limit(Some("0")),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_collection_page_limit(Some("65")),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            parse_collection_page_limit(Some("two")),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(parse_collection_page_cursor(None).expect("none"), None);
        assert_eq!(
            parse_collection_page_cursor(Some("tepp-run-1")).expect("cursor"),
            Some("tepp-run-1".into())
        );
        assert_eq!(
            parse_collection_page_cursor(Some("")),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_collection_page_cursor(Some(
                &"a".repeat(ANALYSIS_RUN_COLLECTION_CURSOR_MAX_LEN + 1)
            )),
            Err(ApiError::LimitExceeded)
        );
    }

    #[test]
    fn collection_exchange_gets_https_path_without_credentials() {
        let exchange =
            naruon_analysis_run_collection_exchange("https://tepp.example.com", None, None)
                .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert_eq!(
            exchange.target_url,
            "https://tepp.example.com/v1/analysis-runs"
        );
        assert!(exchange.body.is_empty());
        assert!(
            exchange
                .headers
                .iter()
                .any(|(name, value)| name == "tepp-consumer" && value == "naruon")
        );
        assert!(
            !exchange
                .headers
                .iter()
                .any(|(name, _)| name.contains("authorization")
                    || name.contains("copilot")
                    || name.contains("idempotency"))
        );

        let paged = naruon_analysis_run_collection_exchange(
            "https://tepp.example.com",
            Some("tepp-run-1"),
            Some("8"),
        )
        .expect("paged");
        assert!(
            paged
                .headers
                .iter()
                .any(|(name, value)| name == "tepp-page-cursor" && value == "tepp-run-1")
        );
        assert!(
            paged
                .headers
                .iter()
                .any(|(name, value)| name == "tepp-page-limit" && value == "8")
        );

        assert_eq!(
            naruon_analysis_run_collection_exchange("http://tepp.example.com", None, None),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_analysis_run_collection_exchange("https://tepp.example.com", Some(""), None),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_analysis_run_collection_exchange("https://tepp.example.com", None, Some("99")),
            Err(ApiError::LimitExceeded)
        );
    }
}
