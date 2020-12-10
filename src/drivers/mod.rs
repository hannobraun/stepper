//! Parent module for all driver implementations
//!
//! This module contains the driver implementations that are currently supported
//! by Step/Dir. Each sub-module is behind a feature gate, to allow users to
//! only enable the drivers they actually need. By default, all drivers are
//! enabled.

#[cfg(feature = "drv8825")]
pub mod drv8825;

#[cfg(feature = "stspin220")]
pub mod stspin220;
