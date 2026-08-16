//! Sealed mappings recover known identities only under re-identification.

use encrypted_mapping::{
    EncryptedMappingError, MappingKey, MappingPurpose, identity_recovery_rate, open_identity,
    refuse_blanket_mask_as_encryption, refuse_persistence_without_later_migration, seal_identity,
};

fn sample_key() -> MappingKey {
    MappingKey::new(7, [0x11; 32]).expect("nonzero key")
}

#[test]
fn sealed_identities_recover_known_truth_only_under_reidentification() {
    let key = sample_key();
    assert_eq!(key.key_id(), 7);
    let truth = [
        "alice@example.test",
        "author-42",
        "department-north-east-campus-overflow-block",
    ];
    let sealed = [
        seal_identity(1, truth[0].as_bytes(), &key, [1; 16]).expect("seal alice"),
        seal_identity(2, truth[1].as_bytes(), &key, [2; 16]).expect("seal author"),
        seal_identity(3, truth[2].as_bytes(), &key, [3; 16]).expect("seal long"),
    ];
    assert_eq!(sealed[0].analytical_id(), 1);
    assert_eq!(sealed[0].key_id(), 7);

    let opened: Vec<String> = sealed
        .iter()
        .map(|envelope| {
            String::from_utf8(
                open_identity(envelope, &key, MappingPurpose::ReidentificationExport)
                    .expect("open"),
            )
            .expect("utf8")
        })
        .collect();
    let recovered = identity_recovery_rate(&truth, &opened).expect("rate");
    let expected = {
        let mut matches = 0_u32;
        for (truth_identity, opened_identity) in truth.iter().zip(opened.iter()) {
            if truth_identity == opened_identity {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered - expected).abs() < f64::EPSILON);
    assert!((recovered - 1.0).abs() < f64::EPSILON);
    let collapsed = [
        "alice@example.test".to_owned(),
        "alice@example.test".to_owned(),
        "alice@example.test".to_owned(),
    ];
    let collapsed_rate = identity_recovery_rate(&truth, &collapsed).expect("collapsed");
    assert!(recovered > collapsed_rate);
}

#[test]
fn analytical_log_and_artifact_purposes_cannot_open_plaintext() {
    let key = sample_key();
    let sealed = seal_identity(3, b"partner-east", &key, [3; 16]).expect("seal");
    for purpose in [
        MappingPurpose::AnalyticalComputation,
        MappingPurpose::OperationalLog,
        MappingPurpose::ModelArtifact,
    ] {
        assert_eq!(
            open_identity(&sealed, &key, purpose),
            Err(EncryptedMappingError::UnauthorizedPurpose)
        );
    }
}

#[test]
fn wrong_key_or_tampered_envelope_fails_closed() {
    let key = sample_key();
    let sealed = seal_identity(4, b"competitor-west", &key, [4; 16]).expect("seal");
    let other_id = MappingKey::new(8, [0x11; 32]).expect("other id");
    let other_bytes = MappingKey::new(7, [0x22; 32]).expect("other bytes");
    assert_eq!(
        open_identity(&sealed, &other_id, MappingPurpose::ReidentificationExport),
        Err(EncryptedMappingError::KeyIdentityMismatch)
    );
    assert_eq!(
        open_identity(
            &sealed,
            &other_bytes,
            MappingPurpose::ReidentificationExport
        ),
        Err(EncryptedMappingError::AuthenticationFailed)
    );
}

#[test]
fn empty_payloads_masks_and_persistence_fail_closed() {
    let key = sample_key();
    assert_eq!(
        MappingKey::new(1, [0; 32]),
        Err(EncryptedMappingError::EmptyIdentity)
    );
    assert_eq!(
        MappingKey::from_material(1, b""),
        Err(EncryptedMappingError::EmptyIdentity)
    );
    assert_eq!(
        MappingKey::from_material(1, &[0_u8; 40]),
        Err(EncryptedMappingError::EmptyIdentity)
    );
    let from_exact = MappingKey::from_material(7, &[0x11; 32]).expect("exact");
    assert_eq!(from_exact.key_id(), 7);
    let from_long = MappingKey::from_material(9, &[0xaa; 131]).expect("long");
    let opened_with_long = {
        let sealed = seal_identity(6, b"role-partner", &from_long, [6; 16]).expect("seal long");
        open_identity(&sealed, &from_long, MappingPurpose::ReidentificationExport)
            .expect("open long")
    };
    assert_eq!(opened_with_long, b"role-partner");
    assert_eq!(
        seal_identity(1, b"", &key, [9; 16]),
        Err(EncryptedMappingError::EmptyIdentity)
    );
    assert_eq!(
        identity_recovery_rate(&[], &[]),
        Err(EncryptedMappingError::InvalidMappingPayload)
    );
    assert_eq!(
        identity_recovery_rate(&["a"], &[]),
        Err(EncryptedMappingError::InvalidMappingPayload)
    );
    assert_eq!(
        identity_recovery_rate(&["a", "b"], &["a".to_owned()]),
        Err(EncryptedMappingError::InvalidMappingPayload)
    );
    assert_eq!(
        refuse_blanket_mask_as_encryption(),
        Err(EncryptedMappingError::BlanketMaskIsNotEncryption)
    );
    assert_eq!(
        refuse_persistence_without_later_migration(),
        Err(EncryptedMappingError::PersistenceRequiresLaterMigration)
    );
}
