//! Shared key-combination predicates for Vim-style navigation.

use tuirealm::event::{Key, KeyEvent, KeyModifiers};

/// Returns whether a key event matches plain lowercase `g`.
///
/// # Arguments
///
/// * `key` — Keyboard event to inspect.
///
/// # Returns
///
/// A [`bool`] indicating whether the event is plain `g`.
pub fn is_plain_g(key: &KeyEvent) -> bool {
    key.code == Key::Char('g') && key.modifiers == KeyModifiers::NONE
}

/// Returns whether a key event matches jump-to-end bindings.
///
/// Accepts `G` and shifted `g` variants used in different terminals.
///
/// # Arguments
///
/// * `key` — Keyboard event to inspect.
///
/// # Returns
///
/// A [`bool`] indicating whether the event means "jump to end".
pub fn is_jump_to_end(key: &KeyEvent) -> bool {
    (key.code == Key::Char('G')
        && (key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT))
        || (key.code == Key::Char('g') && key.modifiers == KeyModifiers::SHIFT)
}
