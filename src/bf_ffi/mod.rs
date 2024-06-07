//!
//! Contains a stdout/stdin calling convention for BF programs, allowing for:
//! * Importing functions from the host (for a library or executable)
//! * Calling imported functions from BF
//! * Exporting functions from BF (for a library)
//! * Serializing/Deserializing common values
//!

pub mod enc;
pub mod host;
