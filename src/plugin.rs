use bevy::prelude::*;

use crate::ManyRelationshipType;
use crate::components::{IncomingRelationships, OutgoingRelationships};

/// A no-op plugin that serves as the entry point for the many-relationships system.
///
/// Add this plugin to your app, then call `register_many_relationship::<R>(app)`
/// for each relationship type you want to use.
pub struct ManyRelationshipsPlugin;

impl Plugin for ManyRelationshipsPlugin {
    fn build(&self, _app: &mut App) {
        // No global state needed. Relationship types are registered individually
        // via register_many_relationship::<R>(app).
    }
}

/// Registers a many-to-many relationship type `R` with the app.
///
/// This sets up cleanup observers so that when an entity with relationships
/// is despawned, all related entities are updated accordingly.
///
/// Call this in your plugin's `build()` method for each relationship type.
pub fn register_many_relationship<R: ManyRelationshipType>(app: &mut App) {
    // When a source entity with outgoing relationships is despawned,
    // clean up all targets' incoming sets.
    app.add_observer(cleanup_outgoing_on_despawn::<R>);

    // When a target entity with incoming relationships is despawned,
    // clean up all sources' outgoing sets.
    app.add_observer(cleanup_incoming_on_despawn::<R>);
}

fn cleanup_outgoing_on_despawn<R: ManyRelationshipType>(
    trigger: On<Remove, OutgoingRelationships<R>>,
    query: Query<&OutgoingRelationships<R>>,
    mut commands: Commands,
) {
    let source = trigger.entity;
    let Ok(outgoing) = query.get(source) else {
        return;
    };

    let targets: Vec<Entity> = outgoing.targets().copied().collect();

    for target in targets {
        commands.queue(CleanupIncomingCommand::<R> {
            source,
            target,
            _marker: std::marker::PhantomData,
        });
    }
}

fn cleanup_incoming_on_despawn<R: ManyRelationshipType>(
    trigger: On<Remove, IncomingRelationships<R>>,
    query: Query<&IncomingRelationships<R>>,
    mut commands: Commands,
) {
    let target = trigger.entity;
    let Ok(incoming) = query.get(target) else {
        return;
    };

    let sources: Vec<Entity> = incoming.sources().copied().collect();

    for source in sources {
        commands.queue(CleanupOutgoingCommand::<R> {
            source,
            target,
            _marker: std::marker::PhantomData,
        });
    }
}

struct CleanupIncomingCommand<R: ManyRelationshipType> {
    source: Entity,
    target: Entity,
    _marker: std::marker::PhantomData<R>,
}

impl<R: ManyRelationshipType> Command for CleanupIncomingCommand<R> {
    fn apply(self, world: &mut World) {
        let Some(mut incoming) = world.get_mut::<IncomingRelationships<R>>(self.target) else {
            return;
        };
        incoming.remove(&self.source);
        if incoming.is_empty() {
            world
                .entity_mut(self.target)
                .remove::<IncomingRelationships<R>>();
        }
    }
}

struct CleanupOutgoingCommand<R: ManyRelationshipType> {
    source: Entity,
    target: Entity,
    _marker: std::marker::PhantomData<R>,
}

impl<R: ManyRelationshipType> Command for CleanupOutgoingCommand<R> {
    fn apply(self, world: &mut World) {
        let Some(mut outgoing) = world.get_mut::<OutgoingRelationships<R>>(self.source) else {
            return;
        };
        outgoing.remove(&self.target);
        if outgoing.is_empty() {
            world
                .entity_mut(self.source)
                .remove::<OutgoingRelationships<R>>();
        }
    }
}
