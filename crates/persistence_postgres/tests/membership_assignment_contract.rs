//! Typed membership-assignment SQL must replace polymorphic targets.

use persistence_postgres::{MigrationCatalog, validate_migration_catalog};

fn normalized(sql: &str) -> String {
    sql.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

#[test]
fn embedded_catalog_declares_typed_exactly_one_membership_targets() {
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    validate_migration_catalog(&catalog).expect("embedded migration contract");

    let up_sql = normalized(catalog.up_sql());
    assert!(
        up_sql.contains("create table entity_record"),
        "typed entity targets must exist before membership foreign keys"
    );
    assert!(
        up_sql.contains("create table project_record"),
        "typed project targets must exist before membership foreign keys"
    );
    assert!(
        up_sql.contains("create table text_segment"),
        "document-or-segment observed units require a text_segment table"
    );
    assert!(
        up_sql.contains("constraint membership_assignment_observed_unit_exactly_one"),
        "observed unit must be exactly one typed foreign key"
    );
    assert!(
        up_sql.contains("constraint membership_assignment_target_exactly_one"),
        "membership target must be exactly one typed foreign key"
    );
    assert!(
        up_sql.contains("target_entity_id uuid") && up_sql.contains("target_project_id uuid"),
        "ERD forbids an untyped membership_target_id"
    );
    assert!(
        up_sql.contains("valid_from_window tstzrange")
            && up_sql.contains("valid_to_window tstzrange"),
        "membership validity must use explicit windows, not coerced timestamps"
    );
}

#[test]
fn embedded_catalog_bounds_membership_weights_to_unit_interval() {
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    let up_sql = normalized(catalog.up_sql());
    let down_sql = normalized(catalog.down_sql());

    let add_constraint = "constraint membership_assignment_weight_unit_interval check (membership_weight > 0 and membership_weight <= 1) not valid";
    let validate_constraint =
        "validate constraint membership_assignment_weight_unit_interval";
    let add_position = up_sql
        .find(add_constraint)
        .expect("unit-interval check must be added without scanning existing rows under the initial DDL lock");
    let validate_position = up_sql
        .find(validate_constraint)
        .expect("successor migration must validate pre-existing rows after fail-closed admission is installed");
    assert!(
        add_position < validate_position,
        "NOT VALID admission must precede the lower-lock validation scan"
    );
    assert!(
        down_sql.contains("drop constraint membership_assignment_weight_unit_interval"),
        "rollback must remove only the successor unit-interval constraint"
    );
}

#[test]
fn rollback_restores_foundation_membership_stub() {
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    let down_sql = normalized(catalog.down_sql());
    assert!(
        down_sql.contains("'text_segment'"),
        "typed membership rollback must drop text_segment"
    );
    assert!(
        down_sql.contains("'entity_record'"),
        "typed membership rollback must drop entity_record"
    );
    assert!(
        down_sql.contains("observation_document_id uuid not null"),
        "rollback must restore the 0001 membership stub so 0001 down remains valid"
    );
}
