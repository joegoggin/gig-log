//! TUI components for the API tester feature.
//!
//! This module groups reusable and screen-specific components that render
//! route lists, editors, response details, and global event handling.
//!
//! # Modules
//!
//! - [`body_editor`] — External editor integration for request bodies.
//! - [`core`] — Reusable primitive input and style components.
//! - [`global_listener`] — Global keyboard, mouse, and resize event mapper.
//! - [`response_viewer`] — Response detail rendering components.
//! - [`route_editor`] — Route editor form field components.
//! - [`route_list`] — Grouped route list and selection component.
//! - [`variable_manager`] — Variable table and input components.

pub mod body_editor;
pub mod core;
pub mod global_listener;
pub mod response_viewer;
pub mod route_editor;
pub mod route_list;
pub mod variable_manager;
