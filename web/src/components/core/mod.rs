//! Core components used by app-wide layouts and routing.

/// Provides the shared loading spinner component.
pub mod loading_spinner;
/// Provides navigation bar and menu components.
pub mod nav;
/// Provides toast-style notification rendering.
pub mod notifications;
/// Provides authenticated route guards.
pub mod private_route;

pub use loading_spinner::LoadingSpinner;
pub use notifications::Notifications;
pub use private_route::PrivateRoute;
