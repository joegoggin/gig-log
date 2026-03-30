//! Icon component for `LogoIcon`.

use leptos::prelude::*;

/// Renders the `LogoIcon` component.
///
/// # Returns
///
/// A Leptos view for the `LogoIcon` UI.
#[component]
pub fn LogoIcon() -> impl IntoView {
    const LOGO_SIZE: u16 = 48;

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox=format!("0 0 {} {}", LOGO_SIZE, LOGO_SIZE)
            width=LOGO_SIZE
            height=LOGO_SIZE
            fill="none"
        >
            <g transform="translate(2 2) scale(1.8)">
                <path
                    d="M7.53 2.5h10.83a1 1 0 0 1 .86 1.51l-1.65 2.84a1 1 0 0 1-.86.49H5.88a1 1 0 0 1-.87-1.49l1.66-2.85a1 1 0 0 1 .86-.5Z"
                    fill="#7aa2f7"
                />
                <path
                    d="M17.83 6.3h3.28a1 1 0 0 1 .86 1.5L16.6 17a1 1 0 0 1-.86.5h-3.28a1 1 0 0 1-.86-1.5l5.36-9.2a1 1 0 0 1 .87-.5Z"
                    fill="#7dcfff"
                />
                <path
                    d="M13.53 16.56h3.29a1 1 0 0 1 .86 1.49l-1.72 2.95a1 1 0 0 1-.86.5H4.27a1 1 0 0 1-.86-1.5l1.72-2.94a1 1 0 0 1 .86-.5Z"
                    fill="#e0af68"
                />
                <path
                    d="M2.9 14.43 8.27 5.24a1 1 0 0 1 .86-.5h3.3a1 1 0 0 1 .85 1.5l-5.35 9.2a1 1 0 0 1-.87.5h-3.3a1 1 0 0 1-.86-1.5Z"
                    fill="#bb9af7"
                />
                <path
                    d="M2.03 13.05 3.75 10.1a1 1 0 0 1 .86-.5h3.28a1 1 0 0 1 .86 1.5l-1.72 2.95a1 1 0 0 1-.86.5H2.9a1 1 0 0 1-.87-1.5Z"
                    fill="#9ece6a"
                />
                <path
                    d="M11.57 10.24h8.75a1 1 0 0 1 .86 1.5l-1.72 2.95a1 1 0 0 1-.86.5h-8.76a1 1 0 0 1-.86-1.5l1.72-2.95a1 1 0 0 1 .87-.5Z"
                    fill="#9ece6a"
                />
                <path
                    d="M6.53 3.8h10.74"
                    stroke="#95b5f9"
                    stroke-width="0.8"
                    stroke-linecap="round"
                    opacity="0.8"
                />
                <path
                    d="M3.18 13.33h3.6"
                    stroke="#c5e2a6"
                    stroke-width="0.7"
                    stroke-linecap="round"
                    opacity="0.9"
                />
                <path
                    d="M4.3 20.12h10.72"
                    stroke="#eccfa4"
                    stroke-width="0.8"
                    stroke-linecap="round"
                    opacity="0.7"
                />
            </g>
        </svg>
    }
}
