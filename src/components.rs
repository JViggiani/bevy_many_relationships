use std::collections::HashSet;
use std::marker::PhantomData;

use bevy::prelude::*;

/// Tracks all outgoing relationships of type `R` from this entity.
///
/// Each entity in `targets` is a target of the relationship.
/// Automatically managed by `add_many_relationship` / `remove_many_relationship` commands.
#[derive(Component)]
pub struct OutgoingRelationships<R: Send + Sync + 'static> {
    targets: HashSet<Entity>,
    _marker: PhantomData<R>,
}

impl<R: Send + Sync + 'static> OutgoingRelationships<R> {
    pub(crate) fn new() -> Self {
        Self {
            targets: HashSet::new(),
            _marker: PhantomData,
        }
    }

    pub(crate) fn insert(&mut self, entity: Entity) -> bool {
        self.targets.insert(entity)
    }

    pub(crate) fn remove(&mut self, entity: &Entity) -> bool {
        self.targets.remove(entity)
    }

    /// Returns an iterator over all target entities.
    pub fn targets(&self) -> impl Iterator<Item = &Entity> {
        self.targets.iter()
    }

    /// Returns true if the given entity is a target of this relationship.
    pub fn contains(&self, entity: Entity) -> bool {
        self.targets.contains(&entity)
    }

    /// Returns the number of targets.
    pub fn len(&self) -> usize {
        self.targets.len()
    }

    /// Returns true if there are no targets.
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }
}

/// Tracks all incoming relationships of type `R` to this entity.
///
/// Each entity in `sources` has an outgoing relationship to this entity.
/// Automatically managed by `add_many_relationship` / `remove_many_relationship` commands.
#[derive(Component)]
pub struct IncomingRelationships<R: Send + Sync + 'static> {
    sources: HashSet<Entity>,
    _marker: PhantomData<R>,
}

impl<R: Send + Sync + 'static> IncomingRelationships<R> {
    pub(crate) fn new() -> Self {
        Self {
            sources: HashSet::new(),
            _marker: PhantomData,
        }
    }

    pub(crate) fn insert(&mut self, entity: Entity) -> bool {
        self.sources.insert(entity)
    }

    pub(crate) fn remove(&mut self, entity: &Entity) -> bool {
        self.sources.remove(entity)
    }

    /// Returns an iterator over all source entities.
    pub fn sources(&self) -> impl Iterator<Item = &Entity> {
        self.sources.iter()
    }

    /// Returns true if the given entity is a source of this relationship.
    pub fn contains(&self, entity: Entity) -> bool {
        self.sources.contains(&entity)
    }

    /// Returns the number of sources.
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Returns true if there are no sources.
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}
