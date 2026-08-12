//! Purpose-bound export authorization without blanket PII masking.

use crate::ApiError;
use crate::wire::require_nonempty;
use serde::{Deserialize, Serialize};

/// Analytical purpose declared for an export or artifact disclosure.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticalPurpose {
    /// Scientific recovery / validation study.
    ScientificValidation,
    /// Operational monitoring that must not receive free-text PII fields.
    OperationalMonitoring,
    /// External partner disclosure under a named legal basis.
    PartnerDisclosure,
    /// Cross-service modular consumption by an approved CWL peer.
    ModularServiceConsumer,
}

/// Request to authorize exporting a versioned analytical artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportAuthorizationRequest {
    /// Opaque tenant/workspace identity.
    pub tenant_workspace_id: String,
    /// Opaque principal / role identity (not a password or token).
    pub principal_id: String,
    /// Declared analytical purpose.
    pub purpose: AnalyticalPurpose,
    /// Opaque artifact identity requested for export.
    pub artifact_id: String,
    /// Whether the export payload would include free-text source body fields.
    pub includes_source_text: bool,
}

/// Authorization decision that never masks scientific identity linkages.
///
/// Fields are private so callers cannot forge an allow decision. Construct only
/// via [`authorize_export`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportAuthorizationDecision {
    /// Whether the export may proceed.
    allowed: bool,
    /// Stable machine-readable decision code.
    decision_code: String,
    /// Content-redacting rationale safe for logs.
    rationale: String,
}

impl ExportAuthorizationDecision {
    /// Whether the export may proceed.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        self.allowed
    }

    /// Stable machine-readable decision code.
    #[must_use]
    pub fn decision_code(&self) -> &str {
        &self.decision_code
    }

    /// Content-redacting rationale safe for logs.
    #[must_use]
    pub fn rationale(&self) -> &str {
        &self.rationale
    }
}

impl AnalyticalPurpose {
    /// Return the stable wire name for this purpose.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::ScientificValidation => "scientific_validation",
            Self::OperationalMonitoring => "operational_monitoring",
            Self::PartnerDisclosure => "partner_disclosure",
            Self::ModularServiceConsumer => "modular_service_consumer",
        }
    }
}

/// Authorize an export under purpose-bound policy.
///
/// Policy is exhaustive over [`AnalyticalPurpose`]: a new purpose variant must
/// declare its free-text source-body rule or the build fails. Opaque analytical
/// identity linkages are never blanket-masked. Free-text source bodies are a
/// separate disclosure class: allowed for scientific validation and modular
/// service consumers; denied for operational monitoring and partner disclosure
/// without an elevated grant outside this gate.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for empty required identities.
pub fn authorize_export(
    request: &ExportAuthorizationRequest,
) -> Result<ExportAuthorizationDecision, ApiError> {
    require_nonempty(&request.tenant_workspace_id)?;
    require_nonempty(&request.principal_id)?;
    require_nonempty(&request.artifact_id)?;

    // Exhaustive dispatch: adding a purpose requires an explicit source-text policy.
    let denial = match (request.purpose, request.includes_source_text) {
        (AnalyticalPurpose::OperationalMonitoring, true) => Some((
            "source_text_denied_for_purpose",
            "operational monitoring may not export free-text source bodies",
        )),
        (AnalyticalPurpose::PartnerDisclosure, true) => Some((
            "partner_source_text_requires_elevated_grant",
            "partner disclosure of source text requires elevated grant outside this gate",
        )),
        (
            AnalyticalPurpose::ScientificValidation | AnalyticalPurpose::ModularServiceConsumer,
            true,
        )
        | (
            AnalyticalPurpose::ScientificValidation
            | AnalyticalPurpose::ModularServiceConsumer
            | AnalyticalPurpose::OperationalMonitoring
            | AnalyticalPurpose::PartnerDisclosure,
            false,
        ) => None,
    };

    if let Some((decision_code, rationale)) = denial {
        return Ok(ExportAuthorizationDecision {
            allowed: false,
            decision_code: decision_code.into(),
            rationale: rationale.into(),
        });
    }

    Ok(ExportAuthorizationDecision {
        allowed: true,
        decision_code: "purpose_bound_export_allowed".into(),
        rationale: format!("export allowed for purpose {}", request.purpose.wire_name()),
    })
}

/// Map a deny decision to [`ApiError::AuthorizationDenied`].
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] when the decision is not allowed.
pub fn require_export_allowed(decision: &ExportAuthorizationDecision) -> Result<(), ApiError> {
    if decision.is_allowed() {
        Ok(())
    } else {
        Err(ApiError::AuthorizationDenied)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AnalyticalPurpose, ExportAuthorizationRequest, authorize_export, require_export_allowed,
    };
    use crate::ApiError;

    fn base_request(
        purpose: AnalyticalPurpose,
        includes_source_text: bool,
    ) -> ExportAuthorizationRequest {
        ExportAuthorizationRequest {
            tenant_workspace_id: "tenant-ws-1".into(),
            principal_id: "principal-analyst-1".into(),
            purpose,
            artifact_id: "artifact-1".into(),
            includes_source_text,
        }
    }

    #[test]
    fn purpose_bound_export_gates_without_blanket_masking() {
        let allowed =
            authorize_export(&base_request(AnalyticalPurpose::ScientificValidation, true))
                .expect("sci");
        assert!(allowed.is_allowed());
        require_export_allowed(&allowed).expect("ok");
        assert!(allowed.rationale().contains("scientific_validation"));
        assert_eq!(allowed.decision_code(), "purpose_bound_export_allowed");

        let modular = authorize_export(&base_request(
            AnalyticalPurpose::ModularServiceConsumer,
            true,
        ))
        .expect("mod with source");
        assert!(modular.is_allowed());
        let modular_no_text = authorize_export(&base_request(
            AnalyticalPurpose::ModularServiceConsumer,
            false,
        ))
        .expect("mod");
        assert!(modular_no_text.is_allowed());

        let ops_denied = authorize_export(&base_request(
            AnalyticalPurpose::OperationalMonitoring,
            true,
        ))
        .expect("ops");
        assert!(!ops_denied.is_allowed());
        assert_eq!(ops_denied.decision_code(), "source_text_denied_for_purpose");
        assert_eq!(
            require_export_allowed(&ops_denied),
            Err(ApiError::AuthorizationDenied)
        );

        let ops_ok = authorize_export(&base_request(
            AnalyticalPurpose::OperationalMonitoring,
            false,
        ))
        .expect("ops ok");
        assert!(ops_ok.is_allowed());

        let partner_denied =
            authorize_export(&base_request(AnalyticalPurpose::PartnerDisclosure, true))
                .expect("partner");
        assert!(!partner_denied.is_allowed());
        assert_eq!(
            partner_denied.decision_code(),
            "partner_source_text_requires_elevated_grant"
        );

        let partner_ok =
            authorize_export(&base_request(AnalyticalPurpose::PartnerDisclosure, false))
                .expect("partner ok");
        assert!(partner_ok.is_allowed());

        assert_eq!(
            authorize_export(&ExportAuthorizationRequest {
                tenant_workspace_id: String::new(),
                ..base_request(AnalyticalPurpose::ScientificValidation, false)
            }),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            authorize_export(&ExportAuthorizationRequest {
                principal_id: String::new(),
                ..base_request(AnalyticalPurpose::ScientificValidation, false)
            }),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            authorize_export(&ExportAuthorizationRequest {
                artifact_id: String::new(),
                ..base_request(AnalyticalPurpose::ScientificValidation, false)
            }),
            Err(ApiError::InvalidWirePayload)
        );

        for purpose in [
            AnalyticalPurpose::ScientificValidation,
            AnalyticalPurpose::OperationalMonitoring,
            AnalyticalPurpose::PartnerDisclosure,
            AnalyticalPurpose::ModularServiceConsumer,
        ] {
            assert!(!purpose.wire_name().is_empty());
        }
    }
}
