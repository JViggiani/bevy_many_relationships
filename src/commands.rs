use bevy::prelude::*;

use crate::components::{IncomingRelationships, OutgoingRelationships};
use crate::events::{OnManyRelationshipAdded, OnManyRelationshipRemoved};
use crate::plugin::ensure_many_relationship_registered;

/// Extension trait on `EntityCommands` for many-relationship ergonomics.
pub trait ManyRelatedEntityCommands {
    /// Adds an incoming relationship from `source` to this entity (source -> this).
    ///
    /// Returns no result because command application is deferred.
    /// If you need an immediate boolean result, use world-level mutation helpers directly.
    fn add_incoming_from<R: Send + Sync + 'static>(
        &mut self,
        source: Entity,
        relationship: R,
    ) -> &mut Self;

    /// Sets (upserts) an incoming relationship from `source` to this entity (source -> this).
    fn set_incoming_from<R: Send + Sync + 'static>(
        &mut self,
        source: Entity,
        relationship: R,
    ) -> &mut Self;

    /// Removes an incoming relationship from `source` to this entity (source -/-> this).
    fn remove_incoming_from<R: Send + Sync + 'static>(&mut self, source: Entity) -> &mut Self;

    /// Adds an outgoing relationship from this entity to `target` (this -> target).
    fn add_outgoing_to<R: Send + Sync + 'static>(
        &mut self,
        target: Entity,
        relationship: R,
    ) -> &mut Self;

    /// Sets (upserts) an outgoing relationship from this entity to `target` (this -> target).
    fn set_outgoing_to<R: Send + Sync + 'static>(
        &mut self,
        target: Entity,
        relationship: R,
    ) -> &mut Self;

    /// Removes an outgoing relationship from this entity to `target` (this -/-> target).
    fn remove_outgoing_to<R: Send + Sync + 'static>(&mut self, target: Entity) -> &mut Self;

    /// Legacy helper for marker/default payload relationships.
    fn add_one_many_related<R: Send + Sync + Default + 'static>(&mut self, related: Entity) -> &mut Self;

    /// Legacy helper for marker/default payload relationships.
    fn add_many_related<R: Send + Sync + Default + 'static>(&mut self, related: &[Entity]) -> &mut Self;

    /// Removes many-relationships of type `R` from the given related entities to this entity.
    fn remove_many_related<R: Send + Sync + 'static>(&mut self, related: &[Entity]) -> &mut Self;

    /// Removes all many-relationships of type `R` targeting this entity.
    fn detach_all_many_related<R: Send + Sync + 'static>(&mut self) -> &mut Self;

    /// Spawns one related entity and links it to this entity via relationship type `R`.
    fn with_many_related<R: Send + Sync + Default + 'static>(&mut self, bundle: impl Bundle) -> &mut Self;

    /// Spawns related entities and links each to this entity via relationship type `R`.
    fn with_many_related_entities<R: Send + Sync + Default + 'static>(
        &mut self,
        func: impl FnOnce(&mut ManyRelatedSpawnerCommands<'_, R>),
    ) -> &mut Self;
}

impl<'a> ManyRelatedEntityCommands for EntityCommands<'a> {
    fn add_incoming_from<R: Send + Sync + 'static>(
        &mut self,
        source: Entity,
        relationship: R,
    ) -> &mut Self {
        let target = self.id();
        self.queue(move |mut _entity: EntityWorldMut| {
            _entity.world_scope(move |world| {
                add_many_relationship::<R>(world, source, target, relationship);
            });
        })
    }

    fn set_incoming_from<R: Send + Sync + 'static>(
        &mut self,
        source: Entity,
        relationship: R,
    ) -> &mut Self {
        let target = self.id();
        self.queue(move |mut _entity: EntityWorldMut| {
            _entity.world_scope(move |world| {
                set_many_relationship::<R>(world, source, target, relationship);
            });
        })
    }

    fn remove_incoming_from<R: Send + Sync + 'static>(&mut self, source: Entity) -> &mut Self {
        self.remove_many_related::<R>(&[source])
    }

    fn add_outgoing_to<R: Send + Sync + 'static>(
        &mut self,
        target: Entity,
        relationship: R,
    ) -> &mut Self {
        let source = self.id();
        self.queue(move |mut _entity: EntityWorldMut| {
            _entity.world_scope(move |world| {
                add_many_relationship::<R>(world, source, target, relationship);
            });
        })
    }

    fn set_outgoing_to<R: Send + Sync + 'static>(
        &mut self,
        target: Entity,
        relationship: R,
    ) -> &mut Self {
        let source = self.id();
        self.queue(move |mut _entity: EntityWorldMut| {
            _entity.world_scope(move |world| {
                set_many_relationship::<R>(world, source, target, relationship);
            });
        })
    }

    fn remove_outgoing_to<R: Send + Sync + 'static>(&mut self, target: Entity) -> &mut Self {
        let source = self.id();
        self.queue(move |mut _entity: EntityWorldMut| {
            _entity.world_scope(move |world| {
                remove_many_relationship::<R>(world, source, target);
            });
        })
    }

    fn add_one_many_related<R: Send + Sync + Default + 'static>(&mut self, related: Entity) -> &mut Self {
        self.add_incoming_from::<R>(related, R::default())
    }

    fn add_many_related<R: Send + Sync + Default + 'static>(&mut self, related: &[Entity]) -> &mut Self {
        let target = self.id();
        let related: Box<[Entity]> = related.into();
        self.queue(move |mut _entity: EntityWorldMut| {
            _entity.world_scope(move |world| {
                for source in related.iter().copied() {
                    add_many_relationship::<R>(world, source, target, R::default());
                }
            });
        })
    }

    fn remove_many_related<R: Send + Sync + 'static>(&mut self, related: &[Entity]) -> &mut Self {
        let target = self.id();
        let related: Box<[Entity]> = related.into();
        self.queue(move |mut _entity: EntityWorldMut| {
            _entity.world_scope(move |world| {
                for source in related.iter().copied() {
                    remove_many_relationship::<R>(world, source, target);
                }
            });
        })
    }

    fn detach_all_many_related<R: Send + Sync + 'static>(&mut self) -> &mut Self {
        let target = self.id();
        self.queue(move |mut _entity: EntityWorldMut| {
            _entity.world_scope(move |world| {
                let Some(incoming) = world.get::<IncomingRelationships<R>>(target) else {
                    return;
                };

                let sources: Vec<Entity> = incoming.sources().copied().collect();
                for source in sources {
                    remove_many_relationship::<R>(world, source, target);
                }
            });
        })
    }

    fn with_many_related<R: Send + Sync + Default + 'static>(&mut self, bundle: impl Bundle) -> &mut Self {
        let target = self.id();
        let source = self.commands().spawn(bundle).id();
        queue_add_many_relationship::<R>(&mut self.commands(), source, target, R::default());
        self
    }

    fn with_many_related_entities<R: Send + Sync + Default + 'static>(
        &mut self,
        func: impl FnOnce(&mut ManyRelatedSpawnerCommands<'_, R>),
    ) -> &mut Self {
        let target = self.id();
        let mut spawner = ManyRelatedSpawnerCommands::new(self.commands(), target);
        func(&mut spawner);
        self
    }
}

/// Spawns many-related source entities targeting a specific entity.
pub struct ManyRelatedSpawnerCommands<'w, R: Send + Sync + 'static> {
    target: Entity,
    commands: Commands<'w, 'w>,
    _marker: std::marker::PhantomData<R>,
}

impl<'w, R: Send + Sync + Default + 'static> ManyRelatedSpawnerCommands<'w, R> {
    pub fn new(commands: Commands<'w, 'w>, target: Entity) -> Self {
        Self {
            target,
            commands,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn spawn(&mut self, bundle: impl Bundle) -> EntityCommands<'_> {
        let source = self.commands.spawn(bundle).id();
        queue_add_many_relationship::<R>(&mut self.commands, source, self.target, R::default());
        self.commands.entity(source)
    }

    pub fn spawn_empty(&mut self) -> EntityCommands<'_> {
        let source = self.commands.spawn_empty().id();
        queue_add_many_relationship::<R>(&mut self.commands, source, self.target, R::default());
        self.commands.entity(source)
    }

    pub fn target_entity(&self) -> Entity {
        self.target
    }

    pub fn commands(&mut self) -> Commands<'_, '_> {
        self.commands.reborrow()
    }

    pub fn commands_mut(&mut self) -> &mut Commands<'w, 'w> {
        &mut self.commands
    }
}

fn queue_add_many_relationship<R: Send + Sync + 'static>(
    commands: &mut Commands<'_, '_>,
    source: Entity,
    target: Entity,
    relationship: R,
) {
    commands.queue(AddManyRelationshipCommand::<R> {
        source,
        target,
        relationship,
        _marker: std::marker::PhantomData,
    });
}

pub fn add_many_relationship<R: Send + Sync + 'static>(
    world: &mut World,
    source: Entity,
    target: Entity,
    relationship: R,
) -> bool {
    ensure_many_relationship_registered::<R>(world);

    let added_new = if let Some(mut outgoing) = world.get_mut::<OutgoingRelationships<R>>(source) {
        outgoing.add(target, relationship)
    } else {
        let mut outgoing = OutgoingRelationships::<R>::new();
        let _ = outgoing.add(target, relationship);
        world.entity_mut(source).insert(outgoing);
        true
    };

    if !added_new {
        return false;
    }

    if let Some(mut incoming) = world.get_mut::<IncomingRelationships<R>>(target) {
        incoming.insert(source);
    } else {
        let mut incoming = IncomingRelationships::<R>::new();
        incoming.insert(source);
        world.entity_mut(target).insert(incoming);
    }

    world.trigger(OnManyRelationshipAdded::<R>::new(source, target));

    true
}

pub fn set_many_relationship<R: Send + Sync + 'static>(
    world: &mut World,
    source: Entity,
    target: Entity,
    relationship: R,
) -> bool {
    ensure_many_relationship_registered::<R>(world);

    let added_new = if let Some(mut outgoing) = world.get_mut::<OutgoingRelationships<R>>(source) {
        outgoing.set(target, relationship).is_none()
    } else {
        let mut outgoing = OutgoingRelationships::<R>::new();
        let _ = outgoing.add(target, relationship);
        world.entity_mut(source).insert(outgoing);
        true
    };

    if let Some(mut incoming) = world.get_mut::<IncomingRelationships<R>>(target) {
        incoming.insert(source);
    } else {
        let mut incoming = IncomingRelationships::<R>::new();
        incoming.insert(source);
        world.entity_mut(target).insert(incoming);
    }

    if added_new {
        world.trigger(OnManyRelationshipAdded::<R>::new(source, target));
    }

    added_new
}

pub fn remove_many_relationship<R: Send + Sync + 'static>(
    world: &mut World,
    source: Entity,
    target: Entity,
) -> bool {
    ensure_many_relationship_registered::<R>(world);

    let (removed, source_empty) = if let Some(mut outgoing) = world.get_mut::<OutgoingRelationships<R>>(source) {
        let removed = outgoing.remove(&target).is_some();
        (removed, outgoing.is_empty())
    } else {
        (false, false)
    };

    if !removed {
        return false;
    }

    if source_empty {
        world.entity_mut(source).remove::<OutgoingRelationships<R>>();
    }

    // Remove from target's incoming set
    let target_empty = if let Some(mut incoming) = world.get_mut::<IncomingRelationships<R>>(target) {
        incoming.remove(&source);
        incoming.is_empty()
    } else {
        false
    };

    if target_empty {
        world.entity_mut(target).remove::<IncomingRelationships<R>>();
    }

    world.trigger(OnManyRelationshipRemoved::<R>::new(source, target));

    true
}

struct AddManyRelationshipCommand<R: Send + Sync + 'static> {
    source: Entity,
    target: Entity,
    relationship: R,
    _marker: std::marker::PhantomData<R>,
}

impl<R: Send + Sync + 'static> Command for AddManyRelationshipCommand<R> {
    fn apply(self, world: &mut World) {
        let _ = add_many_relationship::<R>(world, self.source, self.target, self.relationship);
    }
}

