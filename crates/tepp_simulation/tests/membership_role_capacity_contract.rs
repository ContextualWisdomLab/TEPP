//! Simulation configuration must not outgrow its owned membership-role vocabulary.

use tepp_simulation::{SimulationConfig, SimulationError, generate};

fn config(membership_targets: u32) -> Result<SimulationConfig, SimulationError> {
    SimulationConfig::new(
        690,
        2,
        1,
        membership_targets,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    )
}

#[test]
fn six_unique_membership_roles_are_the_supported_configuration_boundary() {
    let maximum = config(6).expect("six owned role labels are representable");
    let manifest = generate(maximum).expect("six-role simulation");
    manifest
        .verify_invariants()
        .expect("maximum supported role vocabulary remains internally valid");
    assert!(
        manifest
            .documents()
            .iter()
            .all(|document| document.memberships().len() == 6)
    );

    assert_eq!(config(7), Err(SimulationError::InvalidConfiguration));
}
