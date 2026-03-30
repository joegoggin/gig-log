//! SVG icon components used across the frontend.

/// Provides the close icon component.
pub mod close;
/// Provides the company icon component.
pub mod company;
/// Provides the error icon component.
pub mod error;
/// Provides the hidden-password icon component.
pub mod eye_closed;
/// Provides the visible-password icon component.
pub mod eye_open;
/// Provides the hamburger menu icon component.
pub mod hamburger;
/// Provides the home icon component.
pub mod home;
/// Provides the info icon component.
pub mod info;
/// Provides the jobs icon component.
pub mod job;
/// Provides the log-out icon component.
pub mod log_out;
/// Provides the GigLog logo icon component.
pub mod logo;
/// Provides the payments icon component.
pub mod payment;
/// Provides the settings icon component.
pub mod settings;
/// Provides the success icon component.
pub mod success;
/// Provides the warning icon component.
pub mod warning;

pub use close::CloseIcon;
pub use company::CompanyIcon;
pub use error::ErrorIcon;
pub use eye_closed::EyeClosedIcon;
pub use eye_open::EyeOpenIcon;
pub use hamburger::HamburgerIcon;
pub use home::HomeIcon;
pub use info::InfoIcon;
pub use job::JobIcon;
pub use log_out::LogOutIcon;
pub use logo::LogoIcon;
pub use payment::PaymentIcon;
pub use success::SuccessIcon;
pub use warning::WarningIcon;
