//! Purpose-bound field grants for selective disclosure.

use crate::SelectiveDisclosureError;

/// Closed field: author or authorship role linkage.
pub const FIELD_AUTHOR_ROLE: u16 = 1;
/// Closed field: event or valid time.
pub const FIELD_EVENT_TIME: u16 = 2;
/// Closed field: membership or contextual role.
pub const FIELD_MEMBERSHIP_ROLE: u16 = 3;
/// Closed field: direct source identity.
pub const FIELD_DIRECT_IDENTITY: u16 = 4;
/// Closed field: raw source text.
pub const FIELD_SOURCE_TEXT: u16 = 5;
/// Closed field: opaque analytical identifier.
pub const FIELD_OPAQUE_ID: u16 = 6;

/// Closed processing purpose for a disclosure decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisclosurePurpose {
    /// Scientific or psychometric export that must keep measurement linkage.
    ScientificValidation,
    /// Operational telemetry that must not receive identity or source text.
    OperationalMonitoring,
    /// Explicit re-identification export of identity or source text.
    ReidentificationExport,
}

/// One purpose-bound set of disclosed field codes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisclosedFieldSet {
    purpose: DisclosurePurpose,
    fields: Vec<u16>,
}

impl DisclosedFieldSet {
    /// Bind a purpose to an already-validated, sorted field list.
    ///
    /// # Errors
    ///
    /// Returns [`SelectiveDisclosureError::InvalidDisclosurePayload`] when the
    /// field list is empty, contains an unknown code, or contains duplicates.
    pub fn new(
        purpose: DisclosurePurpose,
        fields: &[u16],
    ) -> Result<Self, SelectiveDisclosureError> {
        Ok(Self {
            purpose,
            fields: validated_fields(fields)?,
        })
    }

    /// Processing purpose that authorized this field set.
    #[must_use]
    pub const fn purpose(&self) -> DisclosurePurpose {
        self.purpose
    }

    /// Sorted unique field codes that may be emitted.
    #[must_use]
    pub fn fields(&self) -> &[u16] {
        &self.fields
    }
}

/// Disclose requested fields under a purpose-bound grant.
///
/// # Errors
///
/// Returns a fail-closed [`SelectiveDisclosureError`] when the payload is
/// invalid, a requested field is absent, identity/source text is unauthorized,
/// or a scientific purpose would drop required linkage.
pub fn disclose(
    purpose: DisclosurePurpose,
    source_fields: &[u16],
    requested_fields: &[u16],
) -> Result<DisclosedFieldSet, SelectiveDisclosureError> {
    let source = validated_fields(source_fields)?;
    let requested = validated_fields(requested_fields)?;
    let source_bits = field_bits(&source);
    let requested_bits = field_bits(&requested);
    for &code in &requested {
        if source_bits & field_bit(code) == 0 {
            return Err(SelectiveDisclosureError::MissingSourceField);
        }
        if is_identity_or_source(code)
            && !matches!(purpose, DisclosurePurpose::ReidentificationExport)
        {
            return Err(SelectiveDisclosureError::UnauthorizedField);
        }
    }
    if matches!(purpose, DisclosurePurpose::ScientificValidation) {
        for &code in &source {
            if is_scientific_linkage(code) && requested_bits & field_bit(code) == 0 {
                return Err(SelectiveDisclosureError::BlanketMaskDestroysMeasurement);
            }
        }
    }
    DisclosedFieldSet::new(purpose, &requested)
}

/// Refuse to treat a blanket PII mask as a disclosure grant.
///
/// # Errors
///
/// Always returns [`SelectiveDisclosureError::BlanketMaskDestroysMeasurement`].
pub fn refuse_blanket_mask() -> Result<(), SelectiveDisclosureError> {
    Err(SelectiveDisclosureError::BlanketMaskDestroysMeasurement)
}

/// Fraction of disclosed field sets that match known truth.
///
/// # Errors
///
/// Returns [`SelectiveDisclosureError::InvalidDisclosurePayload`] when either
/// slice is empty or the lengths differ.
pub fn disclosure_recovery_rate(
    truth: &[DisclosedFieldSet],
    decided: &[DisclosedFieldSet],
) -> Result<f64, SelectiveDisclosureError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(SelectiveDisclosureError::InvalidDisclosurePayload);
    }
    let mut matches = 0_u32;
    for (truth_record, decided_record) in truth.iter().zip(decided) {
        if truth_record == decided_record {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

fn validated_fields(fields: &[u16]) -> Result<Vec<u16>, SelectiveDisclosureError> {
    if fields.is_empty() {
        return Err(SelectiveDisclosureError::InvalidDisclosurePayload);
    }
    let mut seen = 0_u16;
    let mut sorted = Vec::with_capacity(fields.len());
    for &code in fields {
        if !is_known_field(code) {
            return Err(SelectiveDisclosureError::InvalidDisclosurePayload);
        }
        let bit = field_bit(code);
        if seen & bit != 0 {
            return Err(SelectiveDisclosureError::InvalidDisclosurePayload);
        }
        seen |= bit;
        sorted.push(code);
    }
    sorted.sort_unstable();
    Ok(sorted)
}

const fn is_known_field(code: u16) -> bool {
    matches!(
        code,
        FIELD_AUTHOR_ROLE
            | FIELD_EVENT_TIME
            | FIELD_MEMBERSHIP_ROLE
            | FIELD_DIRECT_IDENTITY
            | FIELD_SOURCE_TEXT
            | FIELD_OPAQUE_ID
    )
}

const fn is_scientific_linkage(code: u16) -> bool {
    matches!(
        code,
        FIELD_AUTHOR_ROLE | FIELD_EVENT_TIME | FIELD_MEMBERSHIP_ROLE
    )
}

const fn is_identity_or_source(code: u16) -> bool {
    matches!(code, FIELD_DIRECT_IDENTITY | FIELD_SOURCE_TEXT)
}

const fn field_bit(code: u16) -> u16 {
    1_u16 << (code - 1)
}

fn field_bits(fields: &[u16]) -> u16 {
    fields
        .iter()
        .fold(0_u16, |bits, &code| bits | field_bit(code))
}

#[cfg(test)]
mod tests {
    use super::{
        DisclosedFieldSet, DisclosurePurpose, FIELD_AUTHOR_ROLE, FIELD_DIRECT_IDENTITY,
        FIELD_EVENT_TIME, FIELD_MEMBERSHIP_ROLE, FIELD_OPAQUE_ID, FIELD_SOURCE_TEXT, disclose,
        disclosure_recovery_rate, refuse_blanket_mask,
    };
    use crate::SelectiveDisclosureError;

    #[test]
    fn local_branches_cover_authorized_paths() {
        let scientific_source = [
            FIELD_AUTHOR_ROLE,
            FIELD_EVENT_TIME,
            FIELD_MEMBERSHIP_ROLE,
            FIELD_DIRECT_IDENTITY,
            FIELD_SOURCE_TEXT,
            FIELD_OPAQUE_ID,
        ];
        let kept = disclose(
            DisclosurePurpose::ScientificValidation,
            &scientific_source,
            &[
                FIELD_MEMBERSHIP_ROLE,
                FIELD_AUTHOR_ROLE,
                FIELD_EVENT_TIME,
                FIELD_OPAQUE_ID,
            ],
        )
        .expect("scientific");
        assert_eq!(kept.purpose(), DisclosurePurpose::ScientificValidation);
        assert_eq!(
            kept.fields(),
            &[
                FIELD_AUTHOR_ROLE,
                FIELD_EVENT_TIME,
                FIELD_MEMBERSHIP_ROLE,
                FIELD_OPAQUE_ID
            ]
        );
        let opaque_only = disclose(
            DisclosurePurpose::ScientificValidation,
            &[FIELD_OPAQUE_ID],
            &[FIELD_OPAQUE_ID],
        )
        .expect("no linkage present");
        assert_eq!(opaque_only.fields(), &[FIELD_OPAQUE_ID]);
        let ops = disclose(
            DisclosurePurpose::OperationalMonitoring,
            &[FIELD_AUTHOR_ROLE, FIELD_OPAQUE_ID],
            &[FIELD_OPAQUE_ID],
        )
        .expect("ops");
        assert_eq!(ops.purpose(), DisclosurePurpose::OperationalMonitoring);
        let exported = disclose(
            DisclosurePurpose::ReidentificationExport,
            &scientific_source,
            &[FIELD_DIRECT_IDENTITY],
        )
        .expect("re-id");
        assert_eq!(exported.fields(), &[FIELD_DIRECT_IDENTITY]);
        let truth = [kept];
        let matched = disclosure_recovery_rate(&truth, &truth).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        let missed = disclosure_recovery_rate(&truth, &[exported]).expect("miss");
        assert!((missed - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn local_branches_cover_fail_closed_paths() {
        assert_eq!(
            disclose(
                DisclosurePurpose::ScientificValidation,
                &[FIELD_AUTHOR_ROLE],
                &[FIELD_OPAQUE_ID],
            ),
            Err(SelectiveDisclosureError::MissingSourceField)
        );
        assert_eq!(
            disclose(
                DisclosurePurpose::OperationalMonitoring,
                &[FIELD_DIRECT_IDENTITY],
                &[FIELD_DIRECT_IDENTITY],
            ),
            Err(SelectiveDisclosureError::UnauthorizedField)
        );
        assert_eq!(
            disclose(
                DisclosurePurpose::ScientificValidation,
                &[FIELD_AUTHOR_ROLE, FIELD_EVENT_TIME, FIELD_OPAQUE_ID],
                &[FIELD_AUTHOR_ROLE, FIELD_OPAQUE_ID],
            ),
            Err(SelectiveDisclosureError::BlanketMaskDestroysMeasurement)
        );
        assert_eq!(
            refuse_blanket_mask(),
            Err(SelectiveDisclosureError::BlanketMaskDestroysMeasurement)
        );
        assert_eq!(
            DisclosedFieldSet::new(DisclosurePurpose::OperationalMonitoring, &[]),
            Err(SelectiveDisclosureError::InvalidDisclosurePayload)
        );
        assert_eq!(
            DisclosedFieldSet::new(DisclosurePurpose::OperationalMonitoring, &[99]),
            Err(SelectiveDisclosureError::InvalidDisclosurePayload)
        );
        assert_eq!(
            disclose(
                DisclosurePurpose::ScientificValidation,
                &[FIELD_OPAQUE_ID, FIELD_OPAQUE_ID],
                &[FIELD_OPAQUE_ID],
            ),
            Err(SelectiveDisclosureError::InvalidDisclosurePayload)
        );
        let truth =
            [
                DisclosedFieldSet::new(DisclosurePurpose::ScientificValidation, &[FIELD_OPAQUE_ID])
                    .expect("truth"),
            ];
        assert_eq!(
            disclosure_recovery_rate(&[], &[]),
            Err(SelectiveDisclosureError::InvalidDisclosurePayload)
        );
        assert_eq!(
            disclosure_recovery_rate(&truth, &[]),
            Err(SelectiveDisclosureError::InvalidDisclosurePayload)
        );
    }
}
