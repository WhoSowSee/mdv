//! Hash-based keyboard and mouse event registration.

use super::{InputClassifier, InputEvent};
use crate::PagerState;
use crossterm::event::{Event, MouseEvent};
use std::{
    collections::HashMap, collections::hash_map::RandomState, hash::BuildHasher, hash::Hash,
    sync::Arc,
};

type EventReturnType = Arc<dyn Fn(Event, &PagerState) -> InputEvent + Send + Sync>;

#[derive(Clone, Eq)]
enum EventWrapper {
    ExactMatchEvent(Event),
    WildEvent,
}

impl From<Event> for EventWrapper {
    fn from(e: Event) -> Self {
        Self::ExactMatchEvent(e)
    }
}

impl From<&Event> for EventWrapper {
    fn from(e: &Event) -> Self {
        Self::ExactMatchEvent(e.clone())
    }
}

impl PartialEq for EventWrapper {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::ExactMatchEvent(Event::Mouse(MouseEvent {
                    kind, modifiers, ..
                })),
                Self::ExactMatchEvent(Event::Mouse(MouseEvent {
                    kind: o_kind,
                    modifiers: o_modifiers,
                    ..
                })),
            ) => kind == o_kind && modifiers == o_modifiers,
            (
                Self::ExactMatchEvent(Event::Resize(..)),
                Self::ExactMatchEvent(Event::Resize(..)),
            )
            | (Self::WildEvent, Self::WildEvent) => true,
            (Self::ExactMatchEvent(ev), Self::ExactMatchEvent(o_ev)) => ev == o_ev,
            _ => false,
        }
    }
}

impl Hash for EventWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let tag = std::mem::discriminant(self);
        tag.hash(state);
        match self {
            Self::ExactMatchEvent(Event::Mouse(MouseEvent {
                kind, modifiers, ..
            })) => {
                kind.hash(state);
                modifiers.hash(state);
            }
            Self::WildEvent | Self::ExactMatchEvent(Event::Resize(..)) => {}
            Self::ExactMatchEvent(v) => {
                v.hash(state);
            }
        }
    }
}

/// Maps terminal events to pager callbacks.
pub struct HashedEventRegister<S>(HashMap<EventWrapper, EventReturnType, S>);

impl HashedEventRegister<RandomState> {
    /// Creates an empty register with the default hasher.
    #[must_use]
    pub fn with_default_hasher() -> Self {
        Self::new(RandomState::new())
    }
}

impl Default for HashedEventRegister<RandomState> {
    fn default() -> Self {
        let mut event_register = Self::new(RandomState::new());
        super::generate_default_bindings(&mut event_register);
        event_register
    }
}

impl<S> InputClassifier for HashedEventRegister<S>
where
    S: BuildHasher,
{
    fn classify_input(&self, ev: Event, ps: &crate::PagerState) -> Option<InputEvent> {
        self.get(&ev).map(|c| c(ev, ps))
    }
}

impl<S> HashedEventRegister<S>
where
    S: BuildHasher,
{
    /// Creates an empty register with `s`.
    pub fn new(s: S) -> Self {
        Self(HashMap::with_hasher(s))
    }

    /// Registers a fallback callback for otherwise unmatched events.
    pub fn insert_wild_event_matcher(
        &mut self,
        cb: impl Fn(Event, &PagerState) -> InputEvent + Send + Sync + 'static,
    ) {
        self.0.insert(EventWrapper::WildEvent, Arc::new(cb));
    }

    fn get(&self, k: &Event) -> Option<&EventReturnType> {
        self.0
            .get(&k.into())
            .map_or_else(|| self.0.get(&EventWrapper::WildEvent), |k| Some(k))
    }

    /// Registers the callback for terminal resize events.
    pub fn add_resize_event(
        &mut self,
        cb: impl Fn(Event, &PagerState) -> InputEvent + Send + Sync + 'static,
    ) {
        let v = Arc::new(cb);
        // Coordinates are ignored for resize-event hashing.
        self.0
            .insert(EventWrapper::ExactMatchEvent(Event::Resize(0, 0)), v);
    }

    /// Removes the resize callback.
    pub fn remove_resize_event(&mut self) {
        self.0
            .remove(&EventWrapper::ExactMatchEvent(Event::Resize(0, 0)));
    }
}

impl<S> HashedEventRegister<S>
where
    S: BuildHasher,
{
    /// Adds or replaces key bindings without duplicate checks.
    pub fn add_key_events(
        &mut self,
        desc: &[&str],
        cb: impl Fn(Event, &PagerState) -> InputEvent + Send + Sync + 'static,
    ) {
        let v = Arc::new(cb);
        for k in desc {
            self.0.insert(
                Event::Key(super::definitions::keydefs::parse_key_event(k)).into(),
                v.clone(),
            );
        }
    }

    /// Adds key bindings while guarding existing mappings with `remap`.
    ///
    /// # Panics
    /// Panics when a binding cannot be inserted under the requested remap policy.
    pub fn add_key_events_checked(
        &mut self,
        desc: &[&str],
        cb: impl Fn(Event, &PagerState) -> InputEvent + Send + Sync + 'static,
        remap: bool,
    ) {
        let v = Arc::new(cb);
        for k in desc {
            let def: EventWrapper =
                Event::Key(super::definitions::keydefs::parse_key_event(k)).into();
            assert!(self.0.contains_key(&def) && remap, "");
            self.0.insert(def, v.clone());
        }
    }

    /// Removes the listed key bindings.
    pub fn remove_key_events(&mut self, desc: &[&str]) {
        for k in desc {
            self.0
                .remove(&Event::Key(super::definitions::keydefs::parse_key_event(k)).into());
        }
    }
}

impl<S> HashedEventRegister<S>
where
    S: BuildHasher,
{
    /// Adds or replaces mouse bindings without duplicate checks.
    pub fn add_mouse_events(
        &mut self,
        desc: &[&str],
        cb: impl Fn(Event, &PagerState) -> InputEvent + Send + Sync + 'static,
    ) {
        let v = Arc::new(cb);
        for k in desc {
            self.0.insert(
                Event::Mouse(super::definitions::mousedefs::parse_mouse_event(k)).into(),
                v.clone(),
            );
        }
    }

    /// Adds mouse bindings while guarding existing mappings with `remap`.
    ///
    /// # Panics
    /// Panics when a binding cannot be inserted under the requested remap policy.
    pub fn add_mouse_events_checked(
        &mut self,
        desc: &[&str],
        cb: impl Fn(Event, &PagerState) -> InputEvent + Send + Sync + 'static,
        remap: bool,
    ) {
        let v = Arc::new(cb);
        for k in desc {
            let def: EventWrapper =
                Event::Mouse(super::definitions::mousedefs::parse_mouse_event(k)).into();
            assert!(self.0.contains_key(&def) && remap, "");
            self.0.insert(def, v.clone());
        }
    }

    /// Removes the listed mouse bindings.
    pub fn remove_mouse_events(&mut self, mouse: &[&str]) {
        for k in mouse {
            self.0
                .remove(&Event::Mouse(super::definitions::mousedefs::parse_mouse_event(k)).into());
        }
    }
}
