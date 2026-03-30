//! Email delivery for the GigLog API.
//!
//! This module provides email sending capabilities through the
//! [Resend](https://resend.com) API. It is split into a low-level client and
//! higher-level sender abstractions that compose emails for specific features.
//!
//! # Modules
//!
//! - [`client`](crate::email::client) — Core HTTP client for the Resend API.
//! - [`senders`](crate::email::senders) — Specialized email sender implementations.

pub mod client;
pub mod senders;
