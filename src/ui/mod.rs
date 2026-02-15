//! User interface module

pub mod components;
pub mod theme;
pub mod app;
pub mod search;
pub mod inspector;
pub mod dependency_view;
pub mod animation;
pub mod splash;

pub use app::{OracleUi, Tab, Focus};
pub use search::{SearchBar, SearchCompletion, CompletionCandidate, CandidateKind, filter_candidates};
pub use inspector::InspectorPanel;
pub use dependency_view::DependencyView;
pub use animation::{AnimationState, SmoothScroll, Easing};
