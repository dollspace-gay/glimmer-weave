//! Common imports for no_std compatibility
//!
//! This module re-exports commonly used types and macros from alloc
//! so that the rest of the codebase can work in both std and no_std modes.

// Suppress warnings for unused imports - not all modules need all types
#[allow(unused_imports)]
pub use alloc::{
    string::{String, ToString},
    vec::Vec,
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    borrow::ToOwned,
};

// Suppress warnings for unused macros
#[allow(unused_imports)]
pub use alloc::{vec, format};
