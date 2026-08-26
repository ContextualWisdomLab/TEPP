//! Synthetic exact recovery for the exhaustive case-deletion runner.

use analysis_engine::{
    CaseDeletionDocument, CaseDeletionFitContext, CaseDeletionRefitter,
    ExhaustiveCaseDeletionError, fit_exhaustive_case_deletion,
};

struct MeanFitter;

struct RefusingFitter;

impl CaseDeletionRefitter<f64, f64> for MeanFitter {
    type Error = ();

    fn fit(
        &self,
        retained_documents: &[&CaseDeletionDocument<f64>],
        _context: &CaseDeletionFitContext,
    ) -> Result<f64, Self::Error> {
        let sum = retained_documents
            .iter()
            .map(|document| document.evidence)
            .sum::<f64>();
        let count = u32::try_from(retained_documents.len()).map_err(|_| ())?;
        Ok(sum / f64::from(count))
    }
}

impl CaseDeletionRefitter<f64, f64> for RefusingFitter {
    type Error = &'static str;

    fn fit(
        &self,
        _retained_documents: &[&CaseDeletionDocument<f64>],
        _context: &CaseDeletionFitContext,
    ) -> Result<f64, Self::Error> {
        Err("synthetic refusal")
    }
}

fn documents() -> Vec<CaseDeletionDocument<f64>> {
    vec![
        CaseDeletionDocument {
            document_id: "document-a".into(),
            evidence: 1.0,
        },
        CaseDeletionDocument {
            document_id: "document-b".into(),
            evidence: 3.0,
        },
        CaseDeletionDocument {
            document_id: "document-c".into(),
            evidence: 8.0,
        },
    ]
}

#[test]
fn recovers_every_known_deletion_fit_with_independent_domains() {
    let result = fit_exhaustive_case_deletion(&documents(), "topic-model-run", &MeanFitter)
        .expect("synthetic fitter must succeed");
    assert_eq!(result.full_posterior.to_bits(), 4.0_f64.to_bits());
    assert_eq!(result.full_seed_domain, "topic-model-run:full");
    assert_eq!(result.deletion_refits.len(), 3);
    assert_eq!(
        result.deletion_refits[0].posterior.to_bits(),
        5.5_f64.to_bits()
    );
    assert_eq!(
        result.deletion_refits[1].posterior.to_bits(),
        4.5_f64.to_bits()
    );
    assert_eq!(
        result.deletion_refits[2].posterior.to_bits(),
        2.0_f64.to_bits()
    );
    assert_eq!(
        result.deletion_refits[1].retained_document_ids,
        vec!["document-a", "document-c"]
    );
    let domains = result
        .deletion_refits
        .iter()
        .map(|fit| fit.seed_domain.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(domains.len(), 3);
    assert!(!domains.contains(result.full_seed_domain.as_str()));
}

#[test]
fn invalid_corpora_fail_before_fitting() {
    assert_eq!(
        fit_exhaustive_case_deletion(&documents()[..1], "run", &MeanFitter),
        Err(ExhaustiveCaseDeletionError::InvalidInput)
    );
    let mut duplicate = documents();
    duplicate[1].document_id = duplicate[0].document_id.clone();
    assert_eq!(
        fit_exhaustive_case_deletion(&duplicate, "run", &MeanFitter),
        Err(ExhaustiveCaseDeletionError::InvalidInput)
    );
    assert_eq!(
        fit_exhaustive_case_deletion(&documents(), " run ", &MeanFitter),
        Err(ExhaustiveCaseDeletionError::InvalidInput)
    );
    assert_eq!(
        fit_exhaustive_case_deletion(&documents(), "run", &RefusingFitter),
        Err(ExhaustiveCaseDeletionError::Fit("synthetic refusal"))
    );
}
