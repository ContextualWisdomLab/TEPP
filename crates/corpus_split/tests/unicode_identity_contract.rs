//! Unicode NFC/NFD identity must co-partition canonically equivalent texts.

use corpus_split::{
    CanonicalTextIdentity, CorpusSplitError, LeakageLinkKind, assert_no_group_leakage,
    build_connected_groups, canonical_equivalence_links, texts_are_canonically_equivalent,
};
use std::collections::BTreeMap;
use uuid::Uuid;

const CAFE_NFC: &str = "caf\u{00e9}";
const CAFE_NFD: &str = "cafe\u{0301}";
const HANGUL_NFC: &str = "\u{ac01}";
const HANGUL_NFD: &str = "\u{1100}\u{1161}\u{11a8}";

#[test]
fn nfc_and_nfd_latin_and_hangul_are_equivalent() {
    assert!(texts_are_canonically_equivalent(CAFE_NFC, CAFE_NFD).expect("latin"));
    assert!(texts_are_canonically_equivalent(HANGUL_NFC, HANGUL_NFD).expect("hangul"));
    assert!(!texts_are_canonically_equivalent(CAFE_NFC, "cafe").expect("distinct"));
    let identity = CanonicalTextIdentity::from_text(CAFE_NFD).expect("identity");
    assert_eq!(identity.nfc_text(), CAFE_NFC);
    assert_eq!(
        CanonicalTextIdentity::from_text(CAFE_NFC).expect("stable"),
        identity
    );
}

#[test]
fn empty_text_fails_closed() {
    assert_eq!(
        CanonicalTextIdentity::from_text(""),
        Err(CorpusSplitError::EmptyCanonicalText)
    );
    assert_eq!(
        texts_are_canonically_equivalent("", CAFE_NFC),
        Err(CorpusSplitError::EmptyCanonicalText)
    );
    assert_eq!(
        texts_are_canonically_equivalent(CAFE_NFC, ""),
        Err(CorpusSplitError::EmptyCanonicalText)
    );
}

#[test]
fn nfc_nfd_pair_cannot_cross_partitions() {
    let composed_id = Uuid::now_v7();
    let decomposed_id = Uuid::now_v7();
    let other_id = Uuid::now_v7();
    let links = canonical_equivalence_links(&[
        (composed_id, CAFE_NFC),
        (decomposed_id, CAFE_NFD),
        (other_id, "other report"),
    ])
    .expect("links");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].kind, LeakageLinkKind::CanonicalEquivalent);
    assert!(
        (links[0].left == composed_id && links[0].right == decomposed_id)
            || (links[0].left == decomposed_id && links[0].right == composed_id)
    );

    let groups = build_connected_groups(&[composed_id, decomposed_id, other_id], &links);
    assert_eq!(groups.len(), 2);
    let mut leaked = BTreeMap::new();
    leaked.insert(composed_id, 0);
    leaked.insert(decomposed_id, 1);
    leaked.insert(other_id, 0);
    assert_eq!(
        assert_no_group_leakage(&groups, &leaked),
        Err(CorpusSplitError::RelationLeakage)
    );
    let mut held = BTreeMap::new();
    held.insert(composed_id, 0);
    held.insert(decomposed_id, 0);
    held.insert(other_id, 1);
    assert_no_group_leakage(&groups, &held).expect("co-partition");
}

#[test]
fn duplicate_identities_and_isolated_texts_fail_or_emit_nothing() {
    let only = Uuid::now_v7();
    let empty = canonical_equivalence_links(&[(only, CAFE_NFC)]).expect("isolated");
    assert!(empty.is_empty());
    assert_eq!(
        canonical_equivalence_links(&[(only, CAFE_NFC), (only, CAFE_NFD)]),
        Err(CorpusSplitError::DuplicateDocumentIdentity)
    );
    assert_eq!(
        canonical_equivalence_links(&[(only, "")]),
        Err(CorpusSplitError::EmptyCanonicalText)
    );
}
