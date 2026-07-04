use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use std::any::TypeId;

use crate::commands::{detach_incoming_edge, detach_outgoing_edge};
use crate::components::{IncomingRelationships, OutgoingRelationships};

#[derive(Resource, Default)]
pub(crate) struct RegisteredManyRelationshipTypes {
    type_ids: HashSet<TypeId>,
}

/// A plugin that enables many-relationship behavior.
///
/// Add this plugin to your [`App`] before using any many-relationship APIs.
pub struct ManyRelationshipsPlugin;

impl Plugin for ManyRelationshipsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RegisteredManyRelationshipTypes>();
    }
}

pub(crate) fn ensure_many_relationship_registered<R: Send + Sync + 'static>(world: &mut World) {
    let type_id = TypeId::of::<R>();

    match world.get_resource::<RegisteredManyRelationshipTypes>() {
        Some(registered) if registered.type_ids.contains(&type_id) => return,
        None => {
            panic!(
                "ManyRelationshipsPlugin must be added to the App before using many-relationship APIs"
            );
        }
        Some(_) => {}
    }

    let mut registered = world.resource_mut::<RegisteredManyRelationshipTypes>();
    if !registered.type_ids.insert(type_id) {
        return;
    }

    // When a source entity with outgoing relationships is despawned,
    // clean up all targets' incoming sets.
    world.add_observer(cleanup_outgoing_on_despawn::<R>);

    // When a target entity with incoming relationships is despawned,
    // clean up all sources' outgoing sets.
    world.add_observer(cleanup_incoming_on_despawn::<R>);
}

fn cleanup_outgoing_on_despawn<R: Send + Sync + 'static>(
    trigger: On<Remove, OutgoingRelationships<R>>,
    query: Query<&OutgoingRelationships<R>>,
    mut commands: Commands,
) {
    let source = trigger.entity;
    let Ok(outgoing) = query.get(source) else {
        return;
    };

    let targets: Box<[Entity]> = outgoing.targets().collect();
    commands.queue(move |world: &mut World| {
        for target in targets {
            detach_incoming_edge::<R>(world, source, target);
        }
    });
}

fn cleanup_incoming_on_despawn<R: Send + Sync + 'static>(
    trigger: On<Remove, IncomingRelationships<R>>,
    query: Query<&IncomingRelationships<R>>,
    mut commands: Commands,
) {
    let target = trigger.entity;
    let Ok(incoming) = query.get(target) else {
        return;
    };

    let sources: Box<[Entity]> = incoming.sources().collect();
    commands.queue(move |world: &mut World| {
        for source in sources {
            detach_outgoing_edge::<R>(world, source, target);
        }
    });
}
