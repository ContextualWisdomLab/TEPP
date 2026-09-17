use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn rls_catalog(role_sql: &str) -> MigrationCatalog {
    let up_sql = format!(
        r#"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL
        );
        {role_sql}
        ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
        CREATE POLICY tenant_record_tenant_isolation ON tenant_record
            FOR ALL
            USING (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            )
            WITH CHECK (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            );
        "#
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;")
}

#[test]
fn runtime_role_cannot_bypass_rls_or_be_superuser() {
    for role_sql in [
        "CREATE ROLE tepp_app_runtime BYPASSRLS;",
        "CREATE ROLE tepp_app_runtime SUPERUSER;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nALTER ROLE tepp_app_runtime BYPASSRLS;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nALTER USER tepp_app_runtime SUPERUSER;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nALTER GROUP tepp_app_runtime BYPASSRLS;",
        "CREATE ROLE staged_runtime_role BYPASSRLS;\nALTER ROLE staged_runtime_role RENAME TO tepp_app_runtime;",
    ] {
        let catalog = rls_catalog(role_sql);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "{role_sql}"
        );
    }
}

#[test]
fn runtime_role_cannot_gain_a_set_role_path_around_rls() {
    for role_sql in [
        "CREATE ROLE rls_bypass_operator BYPASSRLS;\nCREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nGRANT rls_bypass_operator TO tepp_app_runtime;",
        "CREATE ROLE rls_bypass_operator BYPASSRLS;\nCREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nGRANT rls_bypass_operator TO tepp_app_runtime WITH SET TRUE;",
        "CREATE ROLE rls_bypass_operator BYPASSRLS;\nCREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nGRANT rls_bypass_operator TO tepp_app_runtime WITH SET OPTION;",
        "CREATE ROLE rls_bypass_operator BYPASSRLS;\nCREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS IN ROLE rls_bypass_operator;",
    ] {
        let catalog = rls_catalog(role_sql);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "{role_sql}"
        );
    }
}

#[test]
fn set_false_membership_and_existing_grant_direction_remain_rls_safe() {
    for role_sql in [
        "CREATE ROLE reporting_operator BYPASSRLS;\nCREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nGRANT reporting_operator TO tepp_app_runtime WITH SET FALSE;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nGRANT tepp_app_runtime TO CURRENT_USER;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nGRANT SELECT ON TABLE tenant_record TO tepp_app_runtime;",
    ] {
        let catalog = rls_catalog(role_sql);
        assert_eq!(validate_migration_catalog(&catalog), Ok(()), "{role_sql}");
    }
}

#[test]
fn final_runtime_membership_state_may_explicitly_restore_rls_safety() {
    for role_sql in [
        "CREATE ROLE reporting_operator BYPASSRLS;\nCREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nGRANT reporting_operator TO tepp_app_runtime;\nGRANT reporting_operator TO tepp_app_runtime WITH SET FALSE;",
        "CREATE ROLE reporting_operator BYPASSRLS;\nCREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nGRANT reporting_operator TO tepp_app_runtime;\nREVOKE SET OPTION FOR reporting_operator FROM tepp_app_runtime;",
        "CREATE ROLE reporting_operator BYPASSRLS;\nCREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nGRANT reporting_operator TO tepp_app_runtime;\nREVOKE reporting_operator FROM tepp_app_runtime;",
    ] {
        let catalog = rls_catalog(role_sql);
        assert_eq!(validate_migration_catalog(&catalog), Ok(()), "{role_sql}");
    }
}

#[test]
fn malformed_role_lifecycle_statements_fail_closed_without_panicking() {
    for role_sql in [
        "CREATE ROLE;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nDROP ROLE;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nDROP ROLE , other_role;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nDROP ROLE other_role,;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nDROP ROLE other_role,,another_role;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nDROP ROLE other_role another_role;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nALTER ROLE;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nALTER ROLE tepp_app_runtime;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nALTER ROLE tepp_app_runtime RENAME;",
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nALTER ROLE tepp_app_runtime RENAME TO;",
    ] {
        let catalog = rls_catalog(role_sql);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "{role_sql}"
        );
    }
}

#[test]
fn final_runtime_role_state_may_explicitly_restore_rls_safety() {
    let catalog = rls_catalog(
        "CREATE ROLE tepp_app_runtime SUPERUSER BYPASSRLS;\nALTER ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;",
    );

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
