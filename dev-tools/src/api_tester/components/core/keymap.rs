use tuirealm::event::{Key, KeyEvent, KeyModifiers};

pub fn is_plain_g(key: &KeyEvent) -> bool {
    key.code == Key::Char('g') && key.modifiers == KeyModifiers::NONE
}

pub fn is_jump_to_end(key: &KeyEvent) -> bool {
    (key.code == Key::Char('G')
        && (key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT))
        || (key.code == Key::Char('g') && key.modifiers == KeyModifiers::SHIFT)
}
