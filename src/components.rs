use std::marker::PhantomData;
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

use crate::commands::set_many_relationship;

/// Tracks all outgoing relationships of type `R` from this entity.
///
/// Each target entity maps to one relationship payload of type `R`.
/// Automatically managed by entity-level many-relationship commands.
#[derive(Component)]
pub struct OutgoingRelationships<R: Send + Sync + 'static> {
    targets: EntityHashMap<R>,
}

impl<R: Send + Sync + 'static> OutgoingRelationships<R> {
    pub(crate) fn new() -> Self {
        Self {
            targets: EntityHashMap::default(),
        }
    }

    /// Adds a relationship if absent.
    /// Returns `true` if inserted, `false` if an entry already existed.
    pub(crate) fn add(&mut self, entity: Entity, relationship: R) -> bool {
        if self.targets.contains_key(&entity) {
            false
        } else {
            self.targets.insert(entity, relationship);
            true
        }
    }

    /// Sets (upserts) a relationship, replacing an existing value if present.
    /// Returns the previous value when replaced.
    pub(crate) fn set(&mut self, entity: Entity, relationship: R) -> Option<R> {
        self.targets.insert(entity, relationship)
    }

    pub(crate) fn remove(&mut self, entity: &Entity) -> Option<R> {
        self.targets.remove(entity)
    }

    /// Returns an iterator over all target entities.
    pub fn targets(&self) -> impl Iterator<Item = Entity> {
        self.targets.keys().copied()
    }

    /// Returns an iterator over all outgoing edges and their relationship payloads.
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &R)> {
        self.targets
            .iter()
            .map(|(entity, relationship)| (*entity, relationship))
    }

    /// Returns a relationship payload for the given target entity.
    pub fn get(&self, entity: Entity) -> Option<&R> {
        self.targets.get(&entity)
    }

    /// Returns a mutable relationship payload for the given target entity.
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut R> {
        self.targets.get_mut(&entity)
    }

    /// Returns true if the given entity is a target of this relationship.
    pub fn contains(&self, entity: Entity) -> bool {
        self.targets.contains_key(&entity)
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
/// Automatically managed by entity-level many-relationship commands.
///
/// Payloads are stored only on the source's [`OutgoingRelationships`]. To read a
/// payload for an incoming edge, fetch the source entity's outgoing map via
/// [`get_relationship_payload`](crate::commands::get_relationship_payload) or
/// query `OutgoingRelationships<R>` on the source.
#[derive(Component)]
pub struct IncomingRelationships<R: Send + Sync + 'static> {
    sources: EntityHashSet,
    _marker: PhantomData<R>, 
}

impl<R: Send + Sync + 'static> IncomingRelationships<R> {
    pub(crate) fn new() -> Self {
        Self {
            sources: EntityHashSet::default(),
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
    pub fn sources(&self) -> impl Iterator<Item = Entity> {
        self.sources.iter().copied()
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

/// Bevy-style construction component for adding outgoing relationships on spawn/insert.
///
/// Insert this on a source entity to create relationships `source -> target` for each target and payload.
/// The component is consumed immediately after insertion.
#[derive(Component)]
#[component(on_insert = Self::on_insert)]
pub struct AddOutgoingRelationships<R: Send + Sync + 'static> {
    relationships: Vec<(Entity, R)>,
}

impl<R: Send + Sync + 'static> AddOutgoingRelationships<R> {
    pub fn one(target: Entity, relationship: R) -> Self {
        Self::new([(target, relationship)])
    }

    pub fn new(relationships: impl IntoIterator<Item = (Entity, R)>) -> Self {
        Self {
            relationships: relationships.into_iter().collect(),
        }
    }

    pub fn targets(&self) -> impl Iterator<Item = Entity> + '_ {
        self.relationships.iter().map(|(target, _)| *target)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &R)> {
        self.relationships
            .iter()
            .map(|(target, relationship)| (*target, relationship))
    }

    pub fn len(&self) -> usize {
        self.relationships.len()
    }

    pub fn is_empty(&self) -> bool {
        self.relationships.is_empty()
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let source = context.entity;
        let Some(mut component) = world.get_mut::<Self>(source) else {
            return;
        };
        let relationships = std::mem::take(&mut component.relationships);
        drop(component);

        world.commands().queue(move |world: &mut World| {
            for (target, relationship) in relationships {
                set_many_relationship::<R>(world, source, target, relationship);
            }

            if let Ok(mut source_entity) = world.get_entity_mut(source) {
                source_entity.remove::<Self>();
            }
        });
    }
}
