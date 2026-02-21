use bevy::prelude::*;
use std::any::TypeId;
use std::collections::HashSet;

use crate::components::{IncomingRelationships, OutgoingRelationships};

#[derive(Resource, Default)]
struct RegisteredManyRelationshipTypes {
    type_ids: HashSet<TypeId>,
}

/// A plugin that enables many-relationship behavior.
pub struct ManyRelationshipsPlugin;

impl Plugin for ManyRelationshipsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RegisteredManyRelationshipTypes>();
    }
}

pub(crate) fn ensure_many_relationship_registered<R: Send + Sync + 'static>(world: &mut World) {
    world.init_resource::<RegisteredManyRelationshipTypes>();
    let type_id = TypeId::of::<R>();

    {
        let mut registered = world.resource_mut::<RegisteredManyRelationshipTypes>();
        if !registered.type_ids.insert(type_id) {
            return;
        }
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

    let targets: Vec<Entity> = outgoing.targets().copied().collect();

    commands.queue(CleanupIncomingBatchCommand::<R> {
        source,
        targets,
        _marker: std::marker::PhantomData,
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

    let sources: Vec<Entity> = incoming.sources().copied().collect();

    commands.queue(CleanupOutgoingBatchCommand::<R> {
        target,
        sources,
        _marker: std::marker::PhantomData,
    });
}

struct CleanupIncomingBatchCommand<R: Send + Sync + 'static> {
    source: Entity,
    targets: Vec<Entity>,
    _marker: std::marker::PhantomData<R>,
}

impl<R: Send + Sync + 'static> Command for CleanupIncomingBatchCommand<R> {
    fn apply(self, world: &mut World) {
        for target in self.targets {
            let remove_component = {
                let Some(mut incoming) = world.get_mut::<IncomingRelationships<R>>(target) else {
                    continue;
                };
                incoming.remove(&self.source);
                incoming.is_empty()
            };

            if remove_component {
                if let Ok(mut entity) = world.get_entity_mut(target) {
                    entity.remove::<IncomingRelationships<R>>();
                }
            }
        }
    }
}

struct CleanupOutgoingBatchCommand<R: Send + Sync + 'static> {
    target: Entity,
    sources: Vec<Entity>,
    _marker: std::marker::PhantomData<R>,
}

impl<R: Send + Sync + 'static> Command for CleanupOutgoingBatchCommand<R> {
    fn apply(self, world: &mut World) {
        for source in self.sources {
            let remove_component = {
                let Some(mut outgoing) = world.get_mut::<OutgoingRelationships<R>>(source) else {
                    continue;
                };
                outgoing.remove(&self.target);
                outgoing.is_empty()
            };

            if remove_component {
                if let Ok(mut entity) = world.get_entity_mut(source) {
                    entity.remove::<OutgoingRelationships<R>>();
                }
            }
        }
    }
}
