-- TEPP tenant row-level security (ADR 0013 / physical ERD isolation).
-- Session GUC: tepp.current_tenant_record_id (UUID text).
-- Unset/empty GUC yields no visible rows. Application work uses role
-- tepp_app_runtime (NOSUPERUSER, NOBYPASSRLS).

CREATE ROLE tepp_app_runtime NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS;

GRANT USAGE ON SCHEMA public TO tepp_app_runtime;

GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE tenant_record TO tepp_app_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE source_artifact TO tepp_app_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE document_record TO tepp_app_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE event_instance TO tepp_app_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE event_mention TO tepp_app_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE event_relation TO tepp_app_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE membership_assignment TO tepp_app_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE audit_event TO tepp_app_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE reproducibility_manifest TO tepp_app_runtime;

GRANT tepp_app_runtime TO CURRENT_USER;

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

ALTER TABLE source_artifact ENABLE ROW LEVEL SECURITY;
ALTER TABLE source_artifact FORCE ROW LEVEL SECURITY;
CREATE POLICY source_artifact_tenant_isolation ON source_artifact
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE document_record ENABLE ROW LEVEL SECURITY;
ALTER TABLE document_record FORCE ROW LEVEL SECURITY;
CREATE POLICY document_record_tenant_isolation ON document_record
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE event_instance ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_instance FORCE ROW LEVEL SECURITY;
CREATE POLICY event_instance_tenant_isolation ON event_instance
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE event_mention ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_mention FORCE ROW LEVEL SECURITY;
CREATE POLICY event_mention_tenant_isolation ON event_mention
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE event_relation ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_relation FORCE ROW LEVEL SECURITY;
CREATE POLICY event_relation_tenant_isolation ON event_relation
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE membership_assignment ENABLE ROW LEVEL SECURITY;
ALTER TABLE membership_assignment FORCE ROW LEVEL SECURITY;
CREATE POLICY membership_assignment_tenant_isolation ON membership_assignment
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE audit_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_event FORCE ROW LEVEL SECURITY;
CREATE POLICY audit_event_tenant_isolation ON audit_event
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE reproducibility_manifest ENABLE ROW LEVEL SECURITY;
ALTER TABLE reproducibility_manifest FORCE ROW LEVEL SECURITY;
CREATE POLICY reproducibility_manifest_tenant_isolation ON reproducibility_manifest
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );
