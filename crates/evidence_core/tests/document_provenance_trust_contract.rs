//! Trust-boundary regression for owner-created document provenance.

use evidence_core::{DocumentRecord, EvidenceId};

#[test]
fn owner_document_refuses_unbacked_source_identity() {
    let unattached_source_id = EvidenceId::new();

    assert!(
        DocumentRecord::from_text(unattached_source_id, "document").is_err(),
        "a syntactically valid but unbacked EvidenceId must not become owner document provenance"
    );
}
