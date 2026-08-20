//! TF-IDF, BM25, and global stopword deletion are not inferential inputs.

use corpus_split::{
    CorpusSplitError, LeakageLink, LeakageLinkKind, TokenDeletionRule, WeightingScheme,
    build_connected_groups, group_normalized_weights, refuse_default_stopword_deletion,
    refuse_inferential_retrieval_weight,
};
use std::collections::BTreeMap;
use uuid::Uuid;

fn computed_rmse(truth: &[f64], recovered: &[f64]) -> f64 {
    assert_eq!(truth.len(), recovered.len());
    let n = f64::from(u32::try_from(truth.len()).expect("tiny fixture"));
    let sse: f64 = truth
        .iter()
        .zip(recovered)
        .map(|(truth_value, recovered_value)| {
            let residual = truth_value - recovered_value;
            residual * residual
        })
        .sum();
    (sse / n).sqrt()
}

fn l1_normalize(values: &[f64]) -> Vec<f64> {
    let total: f64 = values.iter().sum();
    assert!(total > 0.0);
    values.iter().map(|value| value / total).collect()
}

/// Classic summed TF-IDF retrieval scores used only as a negative surrogate.
fn tf_idf_document_scores(documents: &[&[&str]]) -> Vec<f64> {
    let document_count = f64::from(u32::try_from(documents.len()).expect("tiny fixture"));
    let mut document_frequency = BTreeMap::<&str, f64>::new();
    for document in documents {
        let mut seen = std::collections::BTreeSet::new();
        for token in *document {
            if seen.insert(*token) {
                *document_frequency.entry(*token).or_insert(0.0) += 1.0;
            }
        }
    }
    documents
        .iter()
        .map(|document| {
            let mut term_frequency = BTreeMap::<&str, f64>::new();
            for token in *document {
                *term_frequency.entry(*token).or_insert(0.0) += 1.0;
            }
            term_frequency
                .into_iter()
                .map(|(token, frequency)| {
                    let df = document_frequency.get(token).copied().unwrap_or(0.0);
                    frequency * (document_count / df).ln()
                })
                .sum()
        })
        .collect()
}

#[test]
fn allowed_observation_weights_pass_and_retrieval_scores_fail_closed() {
    refuse_inferential_retrieval_weight(WeightingScheme::GroupNormalizedEss).expect("ess");
    refuse_inferential_retrieval_weight(WeightingScheme::Uniform).expect("uniform");
    assert_eq!(
        refuse_inferential_retrieval_weight(WeightingScheme::TfIdf),
        Err(CorpusSplitError::InferentialRetrievalWeight)
    );
    assert_eq!(
        refuse_inferential_retrieval_weight(WeightingScheme::Bm25),
        Err(CorpusSplitError::InferentialRetrievalWeight)
    );
}

#[test]
fn global_stopword_deletion_is_not_the_default_rule() {
    refuse_default_stopword_deletion(TokenDeletionRule::PreserveAndModelBackground)
        .expect("preserve");
    assert_eq!(
        refuse_default_stopword_deletion(TokenDeletionRule::GlobalStopwordList),
        Err(CorpusSplitError::DefaultStopwordDeletion)
    );
}

#[test]
fn group_normalized_mass_recovers_true_shares_with_lower_rmse_than_tfidf() {
    let truth = [0.40_f64, 0.10, 0.30, 0.20];
    let observation_mass = [4.0_f64, 1.0, 3.0, 2.0];
    let documents: [&[&str]; 4] = [
        &["report", "report", "report", "event"],
        &["report", "unique"],
        &["report", "event", "event"],
        &["report", "report", "unique", "event"],
    ];
    let document_ids: Vec<Uuid> = (0..truth.len()).map(|_| Uuid::now_v7()).collect();
    let links: Vec<LeakageLink> = document_ids
        .windows(2)
        .map(|pair| LeakageLink {
            left: pair[0],
            right: pair[1],
            kind: LeakageLinkKind::SameEpisode,
        })
        .collect();
    let groups = build_connected_groups(&document_ids, &links);
    let normalized_by_id: BTreeMap<Uuid, f64> = group_normalized_weights(
        &groups,
        &document_ids
            .iter()
            .copied()
            .zip(observation_mass)
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .collect();
    let ess_recovered: Vec<f64> = document_ids
        .iter()
        .map(|document_id| *normalized_by_id.get(document_id).expect("normalized mass"))
        .collect();
    let tfidf_recovered = l1_normalize(&tf_idf_document_scores(&documents));
    let ess_rmse = computed_rmse(&truth, &ess_recovered);
    let tfidf_rmse = computed_rmse(&truth, &tfidf_recovered);
    assert!(
        ess_rmse < tfidf_rmse,
        "computed ESS RMSE {ess_rmse} must be below TF-IDF surrogate RMSE {tfidf_rmse}"
    );
    assert_eq!(
        refuse_inferential_retrieval_weight(WeightingScheme::TfIdf),
        Err(CorpusSplitError::InferentialRetrievalWeight)
    );
}

#[test]
fn wire_names_and_predicates_are_stable() {
    assert_eq!(
        WeightingScheme::GroupNormalizedEss.wire_name(),
        "group_normalized_ess"
    );
    assert_eq!(WeightingScheme::Uniform.wire_name(), "uniform");
    assert_eq!(WeightingScheme::TfIdf.wire_name(), "tf_idf");
    assert_eq!(WeightingScheme::Bm25.wire_name(), "bm25");
    assert!(WeightingScheme::GroupNormalizedEss.is_inferential_weight());
    assert!(WeightingScheme::Uniform.is_inferential_weight());
    assert!(!WeightingScheme::TfIdf.is_inferential_weight());
    assert!(!WeightingScheme::Bm25.is_inferential_weight());
    assert_eq!(
        TokenDeletionRule::PreserveAndModelBackground.wire_name(),
        "preserve_and_model_background"
    );
    assert_eq!(
        TokenDeletionRule::GlobalStopwordList.wire_name(),
        "global_stopword_list"
    );
    assert!(TokenDeletionRule::PreserveAndModelBackground.is_default_allowed());
    assert!(!TokenDeletionRule::GlobalStopwordList.is_default_allowed());
}
