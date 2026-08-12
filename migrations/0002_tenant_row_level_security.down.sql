-- Rollback for 0002_tenant_row_level_security.

DROP POLICY IF EXISTS reproducibility_manifest_tenant_isolation ON reproducibility_manifest;
ALTER TABLE reproducibility_manifest NO FORCE ROW LEVEL SECURITY;
ALTER TABLE reproducibility_manifest DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS audit_event_tenant_isolation ON audit_event;
ALTER TABLE audit_event NO FORCE ROW LEVEL SECURITY;
ALTER TABLE audit_event DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS membership_assignment_tenant_isolation ON membership_assignment;
ALTER TABLE membership_assignment NO FORCE ROW LEVEL SECURITY;
ALTER TABLE membership_assignment DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS event_relation_tenant_isolation ON event_relation;
ALTER TABLE event_relation NO FORCE ROW LEVEL SECURITY;
ALTER TABLE event_relation DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS event_mention_tenant_isolation ON event_mention;
ALTER TABLE event_mention NO FORCE ROW LEVEL SECURITY;
ALTER TABLE event_mention DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS event_instance_tenant_isolation ON event_instance;
ALTER TABLE event_instance NO FORCE ROW LEVEL SECURITY;
ALTER TABLE event_instance DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS document_record_tenant_isolation ON document_record;
ALTER TABLE document_record NO FORCE ROW LEVEL SECURITY;
ALTER TABLE document_record DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS source_artifact_tenant_isolation ON source_artifact;
ALTER TABLE source_artifact NO FORCE ROW LEVEL SECURITY;
ALTER TABLE source_artifact DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_record_tenant_isolation ON tenant_record;
ALTER TABLE tenant_record NO FORCE ROW LEVEL SECURITY;
ALTER TABLE tenant_record DISABLE ROW LEVEL SECURITY;

REVOKE ALL ON TABLE reproducibility_manifest FROM tepp_app_runtime;
REVOKE ALL ON TABLE audit_event FROM tepp_app_runtime;
REVOKE ALL ON TABLE membership_assignment FROM tepp_app_runtime;
REVOKE ALL ON TABLE event_relation FROM tepp_app_runtime;
REVOKE ALL ON TABLE event_mention FROM tepp_app_runtime;
REVOKE ALL ON TABLE event_instance FROM tepp_app_runtime;
REVOKE ALL ON TABLE document_record FROM tepp_app_runtime;
REVOKE ALL ON TABLE source_artifact FROM tepp_app_runtime;
REVOKE ALL ON TABLE tenant_record FROM tepp_app_runtime;
REVOKE USAGE ON SCHEMA public FROM tepp_app_runtime;

DROP ROLE IF EXISTS tepp_app_runtime;
