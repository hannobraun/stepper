//! Step/Dir - Library for controlling stepper motors
//!
//! Step/Dir provides an abstract interface over drivers libraries for stepper
//! motor drivers. It also contains a reference implementation for the STSPIN220
//! stepper motor driver.
//!
//! It is intended to be a low-level interface to typical stepper motor drivers
//! that are controlled using STEP and DIR pins. It does not contain any higher-
//! level features like acceleration ramps. Rather, it is designed as a low-
//! level building block to be used by higher-level control code.

#![no_std]
#![deny(missing_docs)]

pub extern crate embedded_hal;
pub extern crate embedded_time;
