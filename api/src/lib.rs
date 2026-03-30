//! HTTP API crate for the GigLog application.
//!
//! This crate assembles the backend server components used by GigLog,
//! including authentication flows, HTTP controllers, route composition, data
//! access repositories, and shared runtime infrastructure.
//!
//! # Modules
//!
//! - [`auth`] — Authentication and authorization primitives.
//! - [`controllers`] — HTTP request handlers mapped to API endpoints.
//! - [`core`] — Application bootstrap, configuration, errors, and logging.
//! - [`email`] — Email delivery clients and feature-specific senders.
//! - [`extractors`] — Custom Axum request extractors.
//! - [`repo`] — Database repository layer for SQLx queries.
//! - [`routes`] — Router construction and route group definitions.

/// Authentication and authorization primitives.
pub mod auth;
/// HTTP request handlers for API endpoints.
pub mod controllers;
/// Core application bootstrap, configuration, and error handling.
pub mod core;
/// Email delivery clients and sender implementations.
pub mod email;
/// Custom Axum request extractors.
pub mod extractors;
/// Database repository layer for SQLx operations.
pub mod repo;
/// Application route definitions and router composition.
pub mod routes;
