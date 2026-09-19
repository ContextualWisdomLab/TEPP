//! RED regressions for the consolidated project-history collection contract.

use tepp_api::{
    ApiError, PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS, ProjectHistoryCollection,
    ProjectHistoryCollectionItem, page_project_history_collection_items,
};

fn item(key: &str, project: &str) -> ProjectHistoryCollectionItem {
    ProjectHistoryCollectionItem::new(
        project,
        key,
        "2026-08-19T23:59:59Z",
        PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS,
    )
    .expect("valid item")
}

#[test]
fn unknown_cursor_fails_closed_instead_of_skipping_histories() {
    let error = page_project_history_collection_items(
        vec![item("idem-a", "project-a"), item("idem-b", "project-b")],
        Some("idem-between"),
        32,
    )
    .expect_err("unknown cursor must not become a lexical seek");
    assert_eq!(error, ApiError::InvalidWirePayload);
}

#[test]
fn collection_rejects_unsorted_duplicate_and_unbound_cursor_pages() {
    assert_eq!(
        ProjectHistoryCollection::new(
            vec![item("idem-b", "project-b"), item("idem-a", "project-a")],
            None,
        ),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ProjectHistoryCollection::new(
            vec![item("idem-a", "project-a"), item("idem-a", "project-b")],
            None,
        ),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ProjectHistoryCollection::new(vec![item("idem-a", "project-a")], Some("idem-z".into())),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ProjectHistoryCollection::new(Vec::new(), Some("idem-a".into())),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn known_cursor_pages_strictly_after_that_row() {
    let (page, next) = page_project_history_collection_items(
        vec![item("idem-c", "project-c"), item("idem-a", "project-a"), item("idem-b", "project-b")],
        Some("idem-a"),
        1,
    )
    .expect("known cursor");
    assert_eq!(page, vec![item("idem-b", "project-b")]);
    assert_eq!(next.as_deref(), Some("idem-b"));
}
