//! Parent module for all driver implementations
//!
//! This module contains the driver implementations that are currently supported
//! by Stepper. Each sub-module is behind a feature gate, to allow users to only
//! enable the drivers they actually need. By default, all drivers are enabled.

#[cfg(feature = "a4988")]
pub mod a4988;

#[cfg(feature = "drv8825")]
pub mod drv8825;

#[cfg(feature = "stspin220")]
pub mod stspin220;

#[cfg(feature = "dq542ma")]
pub mod dq542ma;

#[cfg(feature = "tmc2209")]
pub mod tmc2209;

