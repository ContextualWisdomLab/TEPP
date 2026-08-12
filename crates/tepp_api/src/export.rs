//! Versioned export envelopes for reproducibility, `JSON-LD`, and `GraphML`.

use crate::ApiError;
use crate::wire::{from_json, require_contract_version, require_nonempty, to_json};
use serde::{Deserialize, Serialize};

/// Supported export/reproducibility contract version.
pub const EXPORT_CONTRACT_VERSION: u16 = 1;

/// Immutable reproducibility manifest bound to a completed analytical artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReproducibilityManifest {
    /// Semantic contract version.
    pub contract_version: u16,
    /// Opaque artifact identity.
    pub artifact_id: String,
    /// Immutable snapshot identity used for the run.
    pub snapshot_id: String,
    /// Knowledge cutoff applied during estimation.
    pub knowledge_cutoff: String,
    /// Model/backend contract version.
    pub model_contract_version: String,
    /// Engine/package version string.
    pub engine_version: String,
    /// Hex-encoded content digest of the primary artifact payload.
    pub artifact_digest_sha256: String,
}

/// Minimal `JSON-LD` document envelope for CWL consumers.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JsonLdExport {
    /// Semantic contract version.
    pub contract_version: u16,
    /// `JSON-LD` context URI or term map serialized as string for v1.
    pub context: String,
    /// Primary node identifier.
    pub id: String,
    /// Node type label.
    pub type_name: String,
    /// Opaque payload digest the consumer must verify separately.
    pub artifact_digest_sha256: String,
}

/// Directed `GraphML` export built from normalized edge identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphMlExport {
    /// Graph identifier (XML-escaped opaque token).
    pub graph_id: String,
    /// Directed edge pairs as opaque endpoint labels.
    pub edges: Vec<(String, String)>,
}

impl ReproducibilityManifest {
    /// Construct and validate a reproducibility manifest.
    ///
    /// # Errors
    ///
    /// Returns field-validation errors for empty identities.
    pub fn new(
        artifact_id: impl Into<String>,
        snapshot_id: impl Into<String>,
        knowledge_cutoff: impl Into<String>,
        model_contract_version: impl Into<String>,
        engine_version: impl Into<String>,
        artifact_digest_sha256: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let manifest = Self {
            contract_version: EXPORT_CONTRACT_VERSION,
            artifact_id: artifact_id.into(),
            snapshot_id: snapshot_id.into(),
            knowledge_cutoff: knowledge_cutoff.into(),
            model_contract_version: model_contract_version.into(),
            engine_version: engine_version.into(),
            artifact_digest_sha256: artifact_digest_sha256.into(),
        };
        manifest.validate()?;
        Ok(manifest)
    }

    /// Parse JSON reproducibility manifest.
    ///
    /// # Errors
    ///
    /// Returns wire, version, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        let manifest: Self = from_json(payload)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Serialize to JSON.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json(self)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, EXPORT_CONTRACT_VERSION)?;
        require_nonempty(&self.artifact_id)?;
        require_nonempty(&self.snapshot_id)?;
        require_nonempty(&self.knowledge_cutoff)?;
        require_nonempty(&self.model_contract_version)?;
        require_nonempty(&self.engine_version)?;
        require_nonempty(&self.artifact_digest_sha256)?;
        Ok(())
    }
}

impl JsonLdExport {
    /// Construct a validated `JSON-LD` export envelope.
    ///
    /// # Errors
    ///
    /// Returns field-validation errors for empty values.
    pub fn new(
        context: impl Into<String>,
        id: impl Into<String>,
        type_name: impl Into<String>,
        artifact_digest_sha256: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let export = Self {
            contract_version: EXPORT_CONTRACT_VERSION,
            context: context.into(),
            id: id.into(),
            type_name: type_name.into(),
            artifact_digest_sha256: artifact_digest_sha256.into(),
        };
        export.validate()?;
        Ok(export)
    }

    /// Parse `JSON-LD` export envelope.
    ///
    /// # Errors
    ///
    /// Returns wire, version, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        let export: Self = from_json(payload)?;
        export.validate()?;
        Ok(export)
    }

    /// Serialize to JSON.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json(self)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, EXPORT_CONTRACT_VERSION)?;
        require_nonempty(&self.context)?;
        require_nonempty(&self.id)?;
        require_nonempty(&self.type_name)?;
        require_nonempty(&self.artifact_digest_sha256)?;
        Ok(())
    }
}

impl GraphMlExport {
    /// Construct a `GraphML` export with non-empty graph identity and edges.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when the graph id is empty, any
    /// endpoint is empty, or `edges` is empty. Returns [`ApiError::LimitExceeded`]
    /// only when `edges.len()` exceeds `maximum_edges`.
    pub fn new(
        graph_id: impl Into<String>,
        edges: Vec<(String, String)>,
        maximum_edges: usize,
    ) -> Result<Self, ApiError> {
        let graph_id = graph_id.into();
        require_nonempty(&graph_id)?;
        if edges.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        if edges.len() > maximum_edges {
            return Err(ApiError::LimitExceeded);
        }
        for (left, right) in &edges {
            require_nonempty(left)?;
            require_nonempty(right)?;
        }
        Ok(Self { graph_id, edges })
    }

    /// Render a minimal `GraphML` document string with XML escaping.
    ///
    /// `graph_id` and endpoint labels are XML-escaped for `&`, `<`, `>`, and `"`.
    #[must_use]
    pub fn to_graphml(&self) -> String {
        let mut out = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">\n\
             <graph id=\"",
        );
        out.push_str(&xml_escape(&self.graph_id));
        out.push_str("\" edgedefault=\"directed\">\n");
        let mut nodes = self
            .edges
            .iter()
            .flat_map(|(a, b)| [a.clone(), b.clone()])
            .collect::<Vec<_>>();
        nodes.sort();
        nodes.dedup();
        for node in nodes {
            out.push_str("  <node id=\"");
            out.push_str(&xml_escape(&node));
            out.push_str("\"/>\n");
        }
        for (index, (left, right)) in self.edges.iter().enumerate() {
            out.push_str("  <edge id=\"e");
            out.push_str(&index.to_string());
            out.push_str("\" source=\"");
            out.push_str(&xml_escape(left));
            out.push_str("\" target=\"");
            out.push_str(&xml_escape(right));
            out.push_str("\"/>\n");
        }
        out.push_str("</graph>\n</graphml>\n");
        out
    }
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::{EXPORT_CONTRACT_VERSION, GraphMlExport, JsonLdExport, ReproducibilityManifest};
    use crate::ApiError;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn export_contracts_round_trip_and_fail_closed() {
        let manifest = ReproducibilityManifest::new(
            "art-1",
            "snap-1",
            "2026-08-01T00:00:00Z",
            "model-v1",
            "0.1.0",
            "abc123",
        )
        .expect("manifest");
        let json = manifest.to_json().expect("json");
        assert_eq!(
            ReproducibilityManifest::from_json(&json).expect("d"),
            manifest
        );
        assert_eq!(
            ReproducibilityManifest::new("", "s", "k", "m", "e", "d"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ReproducibilityManifest::new("a", "", "k", "m", "e", "d"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ReproducibilityManifest::new("a", "s", "", "m", "e", "d"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ReproducibilityManifest::new("a", "s", "k", "", "e", "d"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ReproducibilityManifest::new("a", "s", "k", "m", "", "d"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ReproducibilityManifest::new("a", "s", "k", "m", "e", ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ReproducibilityManifest::from_json(
                r#"{"contract_version":9,"artifact_id":"a","snapshot_id":"s","knowledge_cutoff":"k","model_contract_version":"m","engine_version":"e","artifact_digest_sha256":"d"}"#
            ),
            Err(ApiError::UnsupportedContractVersion)
        );

        let jsonld = JsonLdExport::new(
            "https://example.org/tepp/context.jsonld",
            "urn:tepp:artifact:1",
            "ValidationReport",
            "abc123",
        )
        .expect("jsonld");
        assert_eq!(jsonld.contract_version, EXPORT_CONTRACT_VERSION);
        let j = jsonld.to_json().expect("j");
        assert_eq!(JsonLdExport::from_json(&j).expect("d"), jsonld);
        assert_eq!(
            JsonLdExport::new("", "i", "t", "d"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            JsonLdExport::new("c", "", "t", "d"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            JsonLdExport::new("c", "i", "", "d"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            JsonLdExport::new("c", "i", "t", ""),
            Err(ApiError::InvalidWirePayload)
        );

        let graph = GraphMlExport::new(
            "g1",
            vec![("a".into(), "b".into()), ("b".into(), "c&d".into())],
            10,
        )
        .expect("graph");
        let xml = graph.to_graphml();
        assert!(xml.contains("<node id=\"a\"/>"));
        assert!(xml.contains("c&amp;d"));
        assert_eq!(
            GraphMlExport::new("", vec![("a".into(), "b".into())], 10),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            GraphMlExport::new("g", Vec::new(), 10),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            GraphMlExport::new("g", vec![("a".into(), "b".into())], 0),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            GraphMlExport::new("g", vec![(String::new(), "b".into())], 10),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            GraphMlExport::new("g", vec![("a".into(), String::new())], 10),
            Err(ApiError::InvalidWirePayload)
        );
        // escape remaining arms
        let escaped =
            GraphMlExport::new("g\"x", vec![("a<b".into(), "c>d".into())], 5).expect("esc");
        let xml = escaped.to_graphml();
        assert!(xml.contains("&lt;"));
        assert!(xml.contains("&gt;"));
        assert!(xml.contains("&quot;"));
    }
}
