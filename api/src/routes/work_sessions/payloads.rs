//! Request and response payloads for work session endpoints.
//!
//! This module contains all data structures used for serializing and
//! deserializing request and response bodies in work session handlers.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::work_session::WorkSession;

/// Request body for starting a new work session.
///
/// See [`start_work_session`](super::handlers::start_work_session) for the
/// handler that processes this request.
#[derive(Debug, Deserialize, Validate)]
pub struct StartWorkSessionRequest {
    /// Job identifier to track time against.
    pub job_id: Uuid,
}

/// Response body wrapping a single work session.
///
/// See [`start_work_session`](super::handlers::start_work_session),
/// [`pause_work_session`](super::handlers::pause_work_session),
/// [`resume_work_session`](super::handlers::resume_work_session),
/// [`complete_work_session`](super::handlers::complete_work_session), and
/// [`get_active_work_session`](super::handlers::get_active_work_session) for
/// handlers that produce this response.
#[derive(Debug, Serialize)]
pub struct WorkSessionResponse {
    /// Work session resource payload.
    pub work_session: WorkSession,
}
