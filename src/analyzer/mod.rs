//! Rust code analyzer module

pub mod parser;
pub mod dependency;
pub mod registry;
pub mod types;

pub use parser::RustAnalyzer;
pub use dependency::{DependencyAnalyzer, CrateInfo, DependencyInfo, DependencyKind};
pub use registry::{CrateRegistry, InstalledCrate};
pub use types::*;
