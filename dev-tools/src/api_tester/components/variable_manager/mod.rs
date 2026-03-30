//! Variable manager components for global and scoped variables.
//!
//! # Modules
//!
//! - [`key_input`] — Variable key input field.
//! - [`mode_selector`] — Placeholder/hidden mode selector.
//! - [`table`] — Variable list and selection table.
//! - [`value_input`] — Variable value input field.

mod key_input;
mod mode_selector;
mod table;
mod value_input;

/// Key input component used by the variable manager.
pub use key_input::VariableKeyInput;
/// Mode selector component used by the variable manager.
pub use mode_selector::VariableModeSelector;
/// Variable table component used by the variable manager.
pub use table::VariableTable;
/// Value input component used by the variable manager.
pub use value_input::VariableValueInput;
