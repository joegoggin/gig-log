//! Shared data models used across the GigLog application.

/// User accounts and authentication request models.
pub mod user;
/// Company entities and CRUD request models.
pub mod company;
/// Job entities, payment types, and CRUD request models.
pub mod job;
/// Payment tracking entities and CRUD request models.
pub mod payment;
/// Work session time-tracking models.
pub mod work_session;
/// Appearance preferences and color palette models.
pub mod appearance;
/// Health check response model.
pub mod health;
/// API error and validation error models.
pub mod error;
/// Generic response models.
pub mod generic;
