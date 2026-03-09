use std::hash::Hash;

use tuirealm::event::KeyEvent;
use tuirealm::{Sub, SubClause, SubEventClause};

pub struct SubUtils;

impl SubUtils {
    pub fn event_subs<ComponentId, UserEvent, I>(events: I) -> Vec<Sub<ComponentId, UserEvent>>
    where
        ComponentId: Eq + PartialEq + Clone + Hash,
        UserEvent: Eq + PartialEq + Clone,
        I: IntoIterator<Item = SubEventClause<UserEvent>>,
    {
        events
            .into_iter()
            .map(|event| Sub::new(event, SubClause::Always))
            .collect()
    }

    pub fn key_subs<ComponentId, UserEvent, I>(keys: I) -> Vec<Sub<ComponentId, UserEvent>>
    where
        ComponentId: Eq + PartialEq + Clone + Hash,
        UserEvent: Eq + PartialEq + Clone,
        I: IntoIterator<Item = KeyEvent>,
    {
        Self::event_subs(
            keys.into_iter()
                .map(SubEventClause::Keyboard)
                .collect::<Vec<_>>(),
        )
    }
}
