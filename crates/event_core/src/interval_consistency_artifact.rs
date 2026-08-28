//! Durable, digest-bound export of bounded interval-consistency results.

use crate::{EventError, IntervalConsistencyNetwork};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use temporal_core::{AllenRelation, RelationSet, TemporalVariableId};

/// Model-artifact type used by the ADR-0013 persistence chain.
pub const INTERVAL_CONSISTENCY_ARTIFACT_TYPE: &str = "tdt_chronos_interval_consistency_v1";
const SCHEMA_VERSION: &str = "tepp.tdt_chronos_interval_consistency.v1";
const MAX_RELATIONS: usize = 100_000;
const MAX_JSON_BYTES: usize = 4 * 1024 * 1024;

/// One observed or closure-derived ordered interval relation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IntervalConsistencyArtifactRelation {
    /// Opaque source event identity for the left interval.
    pub left_event_id: String,
    /// Opaque source event identity for the right interval.
    pub right_event_id: String,
    /// Remaining Allen relations in stable reasoner order.
    pub allen_relations: Vec<AllenRelation>,
    /// Whether this ordered pair has a direct accepted assertion.
    pub observed: bool,
    /// Accepted-assertion ordinals conservatively supporting this result.
    pub support_assertion_ordinals: Vec<usize>,
}

/// Versioned bounded reasoner result suitable for immutable artifact storage.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IntervalConsistencyArtifact {
    /// Exact typed schema identity.
    pub schema_version: String,
    /// Opaque analysis-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Lowercase SHA-256 of the exact admitted input bytes.
    pub input_digest_sha256: String,
    /// Non-causal observed and closure-derived temporal relations.
    pub relations: Vec<IntervalConsistencyArtifactRelation>,
}

impl IntervalConsistencyArtifact {
    /// Project a closed network into a canonical artifact.
    ///
    /// Universal unconstrained pairs and identity pairs are omitted.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire or reasoner error for invalid bindings.
    pub fn from_network(
        run_id: impl Into<String>,
        snapshot_id: impl Into<String>,
        input_digest_sha256: impl Into<String>,
        network: &IntervalConsistencyNetwork,
        variables: &[(String, TemporalVariableId)],
    ) -> Result<Self, EventError> {
        let mut identities = BTreeSet::new();
        if variables.len() < 2
            || variables
                .iter()
                .any(|(identity, _)| identity.trim().is_empty() || !identities.insert(identity))
        {
            return Err(EventError::InvalidWirePayload);
        }
        let mut ordered_variables = variables.iter().collect::<Vec<_>>();
        ordered_variables.sort_by(|left, right| left.0.cmp(&right.0));
        let mut relations = Vec::new();
        for (left_index, (left_identity, left)) in ordered_variables.iter().enumerate() {
            for (right_identity, right) in ordered_variables.iter().skip(left_index + 1) {
                let derived = network.derived_relation(*left, *right)?;
                if derived.relations() == RelationSet::all() {
                    continue;
                }
                let inverse = network.derived_relation(*right, *left)?;
                relations.push(IntervalConsistencyArtifactRelation {
                    left_event_id: left_identity.clone(),
                    right_event_id: right_identity.clone(),
                    allen_relations: derived.relations().iter().collect(),
                    // Observation is orientation-independent even though the
                    // export retains the stable variable ordering.
                    observed: derived.is_observed() || inverse.is_observed(),
                    support_assertion_ordinals: derived
                        .support()
                        .iter()
                        .map(|identifier| identifier.assertion_ordinal())
                        .collect(),
                });
            }
        }
        relations.sort_by(|left, right| {
            (&left.left_event_id, &left.right_event_id)
                .cmp(&(&right.left_event_id, &right.right_event_id))
        });
        let artifact = Self {
            schema_version: SCHEMA_VERSION.to_owned(),
            run_id: run_id.into(),
            snapshot_id: snapshot_id.into(),
            input_digest_sha256: input_digest_sha256.into(),
            relations,
        };
        artifact.validate()?;
        let _canonical_json = artifact.to_json()?;
        Ok(artifact)
    }

    /// Parse and validate canonical JSON.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::InvalidWirePayload`] for malformed input.
    pub fn from_json(payload: &str) -> Result<Self, EventError> {
        if payload.len() > MAX_JSON_BYTES {
            return Err(EventError::InvalidWirePayload);
        }
        let artifact: Self =
            serde_json::from_str(payload).map_err(|_| EventError::InvalidWirePayload)?;
        artifact.validate()?;
        if artifact.to_json()? != payload {
            return Err(EventError::InvalidWirePayload);
        }
        Ok(artifact)
    }

    /// Serialize canonical validated JSON.
    ///
    /// # Errors
    ///
    /// Returns a wire error when fields or size are invalid.
    pub fn to_json(&self) -> Result<String, EventError> {
        self.validate()?;
        let payload = serde_json::to_string(self).map_err(|_| EventError::InvalidWirePayload)?;
        if payload.len() > MAX_JSON_BYTES {
            return Err(EventError::InvalidWirePayload);
        }
        Ok(payload)
    }

    /// Return the lowercase SHA-256 of canonical JSON bytes.
    ///
    /// # Errors
    ///
    /// Returns a wire error when the artifact is invalid.
    pub fn sha256(&self) -> Result<String, EventError> {
        Ok(format!("{:x}", Sha256::digest(self.to_json()?.as_bytes())))
    }

    /// Render typed `GraphML` with observation and support provenance.
    ///
    /// # Errors
    ///
    /// Returns a wire error when the artifact is invalid.
    pub fn to_graphml(&self) -> Result<String, EventError> {
        self.validate()?;
        let mut nodes = BTreeSet::new();
        for relation in &self.relations {
            nodes.insert(&relation.left_event_id);
            nodes.insert(&relation.right_event_id);
        }
        let mut output = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">\n<key id=\"schema\" for=\"graph\" attr.name=\"schema_version\" attr.type=\"string\"/>\n<key id=\"snapshot\" for=\"graph\" attr.name=\"snapshot_id\" attr.type=\"string\"/>\n<key id=\"input_digest\" for=\"graph\" attr.name=\"input_digest_sha256\" attr.type=\"string\"/>\n<key id=\"relations\" for=\"edge\" attr.name=\"allen_relations\" attr.type=\"string\"/>\n<key id=\"observed\" for=\"edge\" attr.name=\"observed\" attr.type=\"boolean\"/>\n<key id=\"support\" for=\"edge\" attr.name=\"support_assertion_ordinals\" attr.type=\"string\"/>\n<graph id=\"",
        );
        output.push_str(&xml_escape(&self.run_id));
        output.push_str("\" edgedefault=\"directed\">\n");
        writeln!(
            output,
            "<data key=\"schema\">{}</data><data key=\"snapshot\">{}</data><data key=\"input_digest\">{}</data>",
            xml_escape(&self.schema_version),
            xml_escape(&self.snapshot_id),
            self.input_digest_sha256
        )
        .expect("writing to String cannot fail");
        for node in nodes {
            output.push_str("<node id=\"");
            output.push_str(&xml_escape(node));
            output.push_str("\"/>\n");
        }
        for (index, relation) in self.relations.iter().enumerate() {
            append_edge(&mut output, index, relation);
        }
        output.push_str("</graph>\n</graphml>\n");
        Ok(output)
    }

    fn validate(&self) -> Result<(), EventError> {
        if self.schema_version != SCHEMA_VERSION
            || self.run_id.trim().is_empty()
            || self.snapshot_id.trim().is_empty()
            || !valid_digest(&self.input_digest_sha256)
            || self.relations.is_empty()
            || self.relations.len() > MAX_RELATIONS
        {
            return Err(EventError::InvalidWirePayload);
        }
        let mut previous = None;
        for relation in &self.relations {
            let key = (&relation.left_event_id, &relation.right_event_id);
            if relation.left_event_id.trim().is_empty()
                || relation.right_event_id.trim().is_empty()
                || relation.left_event_id == relation.right_event_id
                || relation.allen_relations.is_empty()
                || relation.allen_relations.len() == AllenRelation::ALL.len()
                || relation.support_assertion_ordinals.is_empty()
                || !strictly_increasing(&relation.allen_relations)
                || !strictly_increasing(&relation.support_assertion_ordinals)
                || previous.is_some_and(|old| old >= key)
            {
                return Err(EventError::InvalidWirePayload);
            }
            previous = Some(key);
        }
        Ok(())
    }
}

fn append_edge(output: &mut String, index: usize, relation: &IntervalConsistencyArtifactRelation) {
    let kinds = relation
        .allen_relations
        .iter()
        .map(|value| serde_json::to_string(value).expect("Allen relation serialization"))
        .map(|value| value.trim_matches('"').to_owned())
        .collect::<Vec<_>>()
        .join(",");
    let support = relation
        .support_assertion_ordinals
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    writeln!(
        output,
        "<edge id=\"e{index}\" source=\"{}\" target=\"{}\"><data key=\"relations\">{}</data><data key=\"observed\">{}</data><data key=\"support\">{support}</data></edge>",
        xml_escape(&relation.left_event_id),
        xml_escape(&relation.right_event_id),
        xml_escape(&kinds),
        relation.observed
    )
    .expect("writing to String cannot fail");
}

fn strictly_increasing<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
