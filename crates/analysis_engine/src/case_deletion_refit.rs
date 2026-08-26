//! Exhaustive producer-owned case-deletion refit execution.
//!
//! The runner invokes the same scientific fitter once on the complete corpus
//! and once on every actual `D \ {i}` corpus. It supplies independent,
//! domain-separated randomness identities and never substitutes full-data
//! reweighting, fixed-posterior deletion, or a diagonal approximation.

use std::collections::BTreeSet;

/// One opaque document and the fitter-owned evidence it carries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseDeletionDocument<D> {
    /// Stable opaque document identity.
    pub document_id: String,
    /// Scientific fitter input for this document.
    pub evidence: D,
}

/// Context supplied to each independent fit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseDeletionFitContext {
    /// Domain-separated randomness identity.
    pub seed_domain: String,
    /// Deleted document for a refit, or `None` for the full fit.
    pub deleted_document_id: Option<String>,
}

/// Scientific fitter invoked by the exhaustive runner.
pub trait CaseDeletionRefitter<D, P> {
    /// Fitter-owned failure type.
    type Error;

    /// Fit one posterior to exactly the supplied retained documents.
    ///
    /// # Errors
    ///
    /// Returns the scientific fitter's own error when that exact corpus cannot
    /// be fitted under the supplied provenance context.
    fn fit(
        &self,
        retained_documents: &[&CaseDeletionDocument<D>],
        context: &CaseDeletionFitContext,
    ) -> Result<P, Self::Error>;
}

/// One actual deleted-data posterior fit.
#[derive(Clone, Debug, PartialEq)]
pub struct DeletedDocumentRefit<P> {
    /// Exactly one deleted document identity.
    pub deleted_document_id: String,
    /// Exact retained identities in admitted order.
    pub retained_document_ids: Vec<String>,
    /// Domain-separated randomness identity used by the fitter.
    pub seed_domain: String,
    /// Fitter-owned deleted-data posterior.
    pub posterior: P,
}

/// Full fit and exhaustive actual case-deleted refits.
#[derive(Clone, Debug, PartialEq)]
pub struct ExhaustiveCaseDeletionFits<P> {
    /// Domain-separated randomness identity for the full fit.
    pub full_seed_domain: String,
    /// Full-data posterior fit.
    pub full_posterior: P,
    /// One actual refit for every admitted document, in admitted order.
    pub deletion_refits: Vec<DeletedDocumentRefit<P>>,
}

/// Fail-closed exhaustive-refit errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExhaustiveCaseDeletionError<E> {
    /// The corpus, document identity, or seed-domain base was invalid.
    InvalidInput,
    /// The scientific fitter refused one full or deleted-data fit.
    Fit(E),
}

/// Run the same fitter on `D` and every actual `D \ {i}` corpus.
///
/// # Errors
///
/// Fails closed for fewer than two documents, blank or duplicate identities,
/// a blank seed-domain base, or any fitter failure.
pub fn fit_exhaustive_case_deletion<D, P, F>(
    documents: &[CaseDeletionDocument<D>],
    seed_domain_base: &str,
    fitter: &F,
) -> Result<ExhaustiveCaseDeletionFits<P>, ExhaustiveCaseDeletionError<F::Error>>
where
    F: CaseDeletionRefitter<D, P>,
{
    let document_ids = documents
        .iter()
        .map(|document| document.document_id.as_str())
        .collect::<BTreeSet<_>>();
    if documents.len() < 2
        || seed_domain_base.is_empty()
        || seed_domain_base.trim() != seed_domain_base
        || document_ids.len() != documents.len()
        || document_ids
            .iter()
            .any(|identity| identity.is_empty() || identity.trim() != *identity)
    {
        return Err(ExhaustiveCaseDeletionError::InvalidInput);
    }

    let full_seed_domain = format!("{seed_domain_base}:full");
    let all = documents.iter().collect::<Vec<_>>();
    let full_posterior = fitter
        .fit(
            &all,
            &CaseDeletionFitContext {
                seed_domain: full_seed_domain.clone(),
                deleted_document_id: None,
            },
        )
        .map_err(ExhaustiveCaseDeletionError::Fit)?;

    let mut deletion_refits = Vec::with_capacity(documents.len());
    for deleted in documents {
        let retained = documents
            .iter()
            .filter(|document| document.document_id != deleted.document_id)
            .collect::<Vec<_>>();
        let seed_domain = format!("{seed_domain_base}:delete:{}", deleted.document_id);
        let posterior = fitter
            .fit(
                &retained,
                &CaseDeletionFitContext {
                    seed_domain: seed_domain.clone(),
                    deleted_document_id: Some(deleted.document_id.clone()),
                },
            )
            .map_err(ExhaustiveCaseDeletionError::Fit)?;
        deletion_refits.push(DeletedDocumentRefit {
            deleted_document_id: deleted.document_id.clone(),
            retained_document_ids: retained
                .iter()
                .map(|document| document.document_id.clone())
                .collect(),
            seed_domain,
            posterior,
        });
    }

    Ok(ExhaustiveCaseDeletionFits {
        full_seed_domain,
        full_posterior,
        deletion_refits,
    })
}
