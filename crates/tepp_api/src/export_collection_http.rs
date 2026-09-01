//! Provider-owned export collection GET contracts.
//!
//! GAP-003A unique slice: `GET /v1/exports` enumerates metric-free identities
//! of purpose-bound exports that `AnalysisRunLiveService` / `tepp-loopback`
//! already authorized. Operators do not guess `export_id` values. This module
//! does not duplicate export retrieval GET (#411), export-retrieval CLI
//! (#417), export-authorize CLI (#410), interpretation-run collection GET
//! (#433), project-history collection GET (#424), GET-by-id (#359), Leiden,
//! or GAP-010 Figma/export. Persistence remains GAP-003B. `LineageWeave` is
//! refused. `NaruonLiveService` stays POST-only.

use serde::{Deserialize, Serialize};

use crate::export_http::{
    refuse_metrics_on_export_retrieval_payload, ExportRetrieval, EXPORT_RETRIEVAL_ID_MAX_LEN,
};
use crate::naruon_http::{compose_https_target, NaruonHttpExchange, NARUON_EXPORT_PATH};
use crate::wire::{require_byte_limit, require_nonempty, to_json};
use crate::{ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT};

/// Default page size for export collection GET.
pub const EXPORT_COLLECTION_DEFAULT_LIMIT: usize = 32;
/// Maximum page size for export collection GET.
pub const EXPORT_COLLECTION_MAX_LIMIT: usize = 64;
/// Maximum opaque cursor length on export collection GET.
pub const EXPORT_COLLECTION_CURSOR_MAX_LEN: usize = EXPORT_RETRIEVAL_ID_MAX_LEN;

/// Metric-free export collection page.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportCollection {
    /// Metric-free authorized export identities on this page.
    pub items: Vec<ExportRetrieval>,
    /// Exclusive `export_id` cursor for the next page, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl ExportCollection {
    /// Construct a validated collection page.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for oversized pages or hostile cursors.
    pub fn new(items: Vec<ExportRetrieval>, next_cursor: Option<String>) -> Result<Self, ApiError> {
        if items.len() > EXPORT_COLLECTION_MAX_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        if let Some(cursor) = next_cursor.as_deref() {
            require_nonempty(cursor)?;
            if cursor.len() > EXPORT_COLLECTION_CURSOR_MAX_LEN {
                return Err(ApiError::LimitExceeded);
            }
            if cursor.contains('/') || cursor.contains('\0') {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        let collection = Self { items, next_cursor };
        let payload = to_json(&collection)?;
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_export_retrieval_payload(&payload)?;
        Ok(collection)
    }

    /// Serialize this collection after metric refusal.
    ///
    /// # Errors
    ///
    /// Returns a validation or metric-key error.
    pub fn to_json(&self) -> Result<String, ApiError> {
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_export_retrieval_payload(&payload)?;
        Ok(payload)
    }
}

/// Whether a path is the export collection resource.
#[must_use]
pub fn is_export_collection_path(path: &str) -> bool {
    path == NARUON_EXPORT_PATH
}

/// Parse the optional `tepp-page-limit` header.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-integer and
/// [`ApiError::LimitExceeded`] when above [`EXPORT_COLLECTION_MAX_LIMIT`].
pub fn parse_export_collection_page_limit(raw: Option<&str>) -> Result<usize, ApiError> {
    let Some(raw) = raw else {
        return Ok(EXPORT_COLLECTION_DEFAULT_LIMIT);
    };
    require_nonempty(raw)?;
    let limit: usize = raw.parse().map_err(|_| ApiError::InvalidWirePayload)?;
    if limit == 0 {
        return Err(ApiError::InvalidWirePayload);
    }
    if limit > EXPORT_COLLECTION_MAX_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
    Ok(limit)
}

/// Parse the optional exclusive `tepp-page-cursor` header.
///
/// # Errors
///
/// Returns a fail-closed error for empty, slash, NUL, or oversized cursors.
pub fn parse_export_collection_page_cursor(raw: Option<&str>) -> Result<Option<String>, ApiError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    require_nonempty(raw)?;
    if raw.contains('/') || raw.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    if raw.len() > EXPORT_COLLECTION_CURSOR_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(Some(raw.to_owned()))
}

/// Page stored collection rows with an exclusive `export_id` cursor.
#[must_use]
pub fn page_export_collection_items(
    mut items: Vec<ExportRetrieval>,
    cursor: Option<&str>,
    limit: usize,
) -> (Vec<ExportRetrieval>, Option<String>) {
    items.sort_by(|left, right| left.export_id.cmp(&right.export_id));
    let start = cursor.map_or(0, |cursor| {
        items
            .iter()
            .position(|item| item.export_id.as_str() > cursor)
            .unwrap_or(items.len())
    });
    let end = (start + limit).min(items.len());
    let next_cursor = (end < items.len()).then(|| items[end - 1].export_id.clone());
    (items[start..end].to_vec(), next_cursor)
}

/// Build a credential-free naruon collection GET exchange.
///
/// # Errors
///
/// Returns a fail-closed origin error.
pub fn naruon_export_collection_exchange(origin: &str) -> Result<NaruonHttpExchange, ApiError> {
    let target_url = compose_https_target(origin, NARUON_EXPORT_PATH)?;
    Ok(NaruonHttpExchange {
        method: "GET",
        target_url,
        headers: vec![
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), "naruon".into()),
            ("tepp-contract-version".into(), "1".into()),
        ],
        body: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        is_export_collection_path, naruon_export_collection_exchange,
        parse_export_collection_page_cursor, parse_export_collection_page_limit,
        EXPORT_COLLECTION_MAX_LIMIT,
    };
    use crate::naruon_http::NARUON_EXPORT_PATH;
    use crate::ApiError;

    #[test]
    fn collection_exchange_is_metric_free_get_without_credentials() {
        assert!(is_export_collection_path(NARUON_EXPORT_PATH));
        assert!(!is_export_collection_path("/v1/exports/export-1"));
        let exchange =
            naruon_export_collection_exchange("https://tepp.example.test").expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert!(exchange.target_url.ends_with("/v1/exports"));
        assert!(exchange.body.is_empty());
        assert!(!exchange
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
                || name.eq_ignore_ascii_case("idempotency-key")));
        assert_eq!(
            naruon_export_collection_exchange("http://tepp.example.test"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_export_collection_page_limit(None).expect("default"),
            32
        );
        assert_eq!(
            parse_export_collection_page_limit(Some("99")),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            parse_export_collection_page_cursor(Some("a/b")),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(EXPORT_COLLECTION_MAX_LIMIT, 64);
    }
}
