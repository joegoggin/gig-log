//! Reusable frontend modules for the GigLog web client.
//!
//! This library crate exposes shared modules used by the browser entry point
//! in `main.rs`, including routing components, page layouts, API client
//! wrappers, and application contexts.
//!
//! # Modules
//!
//! - [`api_client`] — HTTP client wrappers and request runners.
//! - [`components`] — Reusable UI components.
//! - [`contexts`] — Reactive context providers and helpers.
//! - [`layouts`] — Shared page layout components.
//! - [`pages`] — Route-level page components.
//! - [`utils`] — Frontend helper utilities.

/// Provides API client wrappers and request helpers.
pub mod api_client;
/// Provides reusable UI components used across pages.
pub mod components;
/// Provides Leptos context state containers and accessors.
pub mod contexts;
/// Provides shared layout components for route pages.
pub mod layouts;
/// Provides route-level page components.
pub mod pages;
/// Provides small utility helpers shared by components.
pub mod utils;
