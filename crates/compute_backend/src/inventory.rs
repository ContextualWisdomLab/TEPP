//! Measured device inventory and usable VRAM budget.

use crate::error::ComputeBackendError;
use crate::profile::VramProfile;

/// Observed accelerator inventory for one planning decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceInventory {
    profile: VramProfile,
    available_bytes: u64,
    device_present: bool,
}

/// Reserved bytes that working tensors must not consume.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SafetyReserve {
    bytes: u64,
}

/// Usable VRAM remaining after the safety reserve.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VramBudget {
    profile: VramProfile,
    available_bytes: u64,
    safety_bytes: u64,
    usable_bytes: u64,
}

impl DeviceInventory {
    /// Construct a present GPU inventory that cannot exceed its profile.
    ///
    /// # Errors
    ///
    /// Returns [`ComputeBackendError::InvalidBudget`] when `available_bytes` is
    /// zero or larger than the profile capacity.
    pub const fn gpu(
        profile: VramProfile,
        available_bytes: u64,
    ) -> Result<Self, ComputeBackendError> {
        if available_bytes == 0 || available_bytes > profile.bytes() {
            return Err(ComputeBackendError::InvalidBudget);
        }
        Ok(Self {
            profile,
            available_bytes,
            device_present: true,
        })
    }

    /// Construct a CPU-only inventory with no accelerator.
    #[must_use]
    pub const fn cpu_only(profile: VramProfile) -> Self {
        Self {
            profile,
            available_bytes: 0,
            device_present: false,
        }
    }

    /// Return the governing profile.
    #[must_use]
    pub const fn profile(self) -> VramProfile {
        self.profile
    }

    /// Return currently free device bytes.
    #[must_use]
    pub const fn available_bytes(self) -> u64 {
        self.available_bytes
    }

    /// Return whether an accelerator is present.
    #[must_use]
    pub const fn device_present(self) -> bool {
        self.device_present
    }

    /// Return the reserved safety headroom.
    #[must_use]
    pub const fn safety_reserve(self) -> SafetyReserve {
        SafetyReserve {
            bytes: self.profile.safety_bytes(),
        }
    }

    /// Return the usable budget after reserving safety memory.
    #[must_use]
    pub const fn budget(self) -> VramBudget {
        let safety_bytes = self.profile.safety_bytes();
        let usable_bytes = if self.device_present && self.available_bytes > safety_bytes {
            self.available_bytes - safety_bytes
        } else {
            0
        };
        VramBudget {
            profile: self.profile,
            available_bytes: self.available_bytes,
            safety_bytes,
            usable_bytes,
        }
    }
}

impl SafetyReserve {
    /// Return reserved bytes.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        self.bytes
    }
}

impl VramBudget {
    /// Return the governing profile.
    #[must_use]
    pub const fn profile(self) -> VramProfile {
        self.profile
    }

    /// Return observed free bytes.
    #[must_use]
    pub const fn available_bytes(self) -> u64 {
        self.available_bytes
    }

    /// Return reserved safety bytes.
    #[must_use]
    pub const fn safety_bytes(self) -> u64 {
        self.safety_bytes
    }

    /// Return bytes available for working tensors.
    #[must_use]
    pub const fn usable_bytes(self) -> u64 {
        self.usable_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceInventory;
    use crate::error::ComputeBackendError;
    use crate::profile::VramProfile;

    #[test]
    fn gpu_inventory_rejects_empty_and_oversize_availability() {
        assert_eq!(
            DeviceInventory::gpu(VramProfile::Gib4, 0),
            Err(ComputeBackendError::InvalidBudget)
        );
        assert_eq!(
            DeviceInventory::gpu(VramProfile::Gib4, VramProfile::Gib4.bytes() + 1),
            Err(ComputeBackendError::InvalidBudget)
        );
        let inventory =
            DeviceInventory::gpu(VramProfile::Gib4, VramProfile::Gib4.bytes()).expect("full 4 GiB");
        assert!(inventory.device_present());
        assert_eq!(inventory.profile(), VramProfile::Gib4);
        assert_eq!(inventory.available_bytes(), VramProfile::Gib4.bytes());
        assert_eq!(
            inventory.safety_reserve().bytes(),
            VramProfile::Gib4.safety_bytes()
        );
        let budget = inventory.budget();
        assert_eq!(budget.profile(), VramProfile::Gib4);
        assert_eq!(budget.available_bytes(), VramProfile::Gib4.bytes());
        assert_eq!(budget.safety_bytes(), VramProfile::Gib4.safety_bytes());
        assert_eq!(
            budget.usable_bytes(),
            VramProfile::Gib4.bytes() - VramProfile::Gib4.safety_bytes()
        );
    }

    #[test]
    fn cpu_only_inventory_has_zero_usable_bytes() {
        let inventory = DeviceInventory::cpu_only(VramProfile::Gib8);
        assert!(!inventory.device_present());
        assert_eq!(inventory.available_bytes(), 0);
        assert_eq!(inventory.budget().usable_bytes(), 0);
    }

    #[test]
    fn availability_at_or_below_reserve_is_unusable() {
        let reserve = VramProfile::Gib6.safety_bytes();
        let inventory = DeviceInventory::gpu(VramProfile::Gib6, reserve).expect("at reserve");
        assert_eq!(inventory.budget().usable_bytes(), 0);
    }
}
