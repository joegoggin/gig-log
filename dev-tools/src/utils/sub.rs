use std::hash::Hash;

use tuirealm::event::KeyEvent;
use tuirealm::{Sub, SubClause, SubEventClause};

pub struct SubUtils;

impl SubUtils {
    pub fn key_subs<ComponentId, UserEvent, I>(keys: I) -> Vec<Sub<ComponentId, UserEvent>>
    where
        ComponentId: Eq + PartialEq + Clone + Hash,
        UserEvent: Eq + PartialEq + Clone,
        I: IntoIterator<Item = KeyEvent>,
    {
        keys.into_iter()
            .map(|key| Sub::new(SubEventClause::Keyboard(key), SubClause::Always))
            .collect()
    }
}
