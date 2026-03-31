//! Shared data models used across the GigLog application.

/// Appearance preferences and color palette models.
pub mod appearance;
/// Company entities and CRUD request models.
pub mod company;
/// API error and validation error models.
pub mod error;
/// Generic response models.
pub mod generic;
/// Health check response model.
pub mod health;
/// Job entities, payment types, and CRUD request models.
pub mod job;
/// Payment tracking entities and CRUD request models.
pub mod payment;
/// User accounts and authentication request models.
pub mod user;
/// Work session time-tracking models.
pub mod work_session;
