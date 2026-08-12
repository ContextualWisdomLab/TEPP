-- Rollback for 0001_bitemporal_foundation.

DROP TABLE IF EXISTS reproducibility_manifest;
DROP TABLE IF EXISTS audit_event;
DROP TABLE IF EXISTS membership_assignment;
DROP TABLE IF EXISTS event_relation;
DROP TABLE IF EXISTS event_mention;
DROP TABLE IF EXISTS event_instance;
DROP TABLE IF EXISTS document_record;
DROP TABLE IF EXISTS source_artifact;
DROP TABLE IF EXISTS tenant_record;
