use core::convert::TryFrom;

/// Defines the step mode with a resolution of up to 256 microsteps
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StepMode256 {
    /// Full steps
    Full = 1,

    /// 2 microsteps per full step
    M2 = 2,

    /// 4 microsteps per full step
    M4 = 4,

    /// 8 microsteps per full step
    M8 = 8,

    /// 16 microsteps per full step
    M16 = 16,

    /// 32 microsteps per full step
    M32 = 32,

    /// 64 microsteps per full step
    M64 = 64,

    /// 128 microsteps per full step
    M128 = 128,

    /// 256 microsteps per full step
    M256 = 256,
}

impl From<StepMode256> for u16 {
    fn from(step_mode: StepMode256) -> Self {
        step_mode as Self
    }
}

impl TryFrom<u16> for StepMode256 {
    type Error = InvalidStepModeError;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(StepMode256::Full),
            2 => Ok(StepMode256::M2),
            4 => Ok(StepMode256::M4),
            8 => Ok(StepMode256::M8),
            16 => Ok(StepMode256::M16),
            32 => Ok(StepMode256::M32),
            64 => Ok(StepMode256::M64),
            128 => Ok(StepMode256::M128),
            256 => Ok(StepMode256::M256),

            _ => Err(InvalidStepModeError),
        }
    }
}

/// Indicates that a given step mode value did not represent a valid step mode
///
/// Valid values are 1, 2, 4, 8, 16, 32, 64, 128, and 256.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidStepModeError;
