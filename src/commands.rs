use bevy::prelude::*;

use crate::ManyRelationshipType;
use crate::components::{IncomingRelationships, OutgoingRelationships};
use crate::events::{OnManyRelationshipAdded, OnManyRelationshipRemoved};

/// Extension trait on `Commands` for adding and removing many-to-many relationships.
pub trait ManyRelationshipCommands {
    /// Adds a many-to-many relationship of type `R` from `source` to `target`.
    ///
    /// If the relationship already exists, this is a no-op (no duplicate, no observer fired).
    /// Creates `OutgoingRelationships<R>` on source and `IncomingRelationships<R>` on target
    /// if they don't already exist.
    fn add_many_relationship<R: ManyRelationshipType>(
        &mut self,
        source: Entity,
        target: Entity,
    );

    /// Removes a many-to-many relationship of type `R` from `source` to `target`.
    ///
    /// If the removal causes an empty set, the component is removed entirely.
    fn remove_many_relationship<R: ManyRelationshipType>(
        &mut self,
        source: Entity,
        target: Entity,
    );
}

impl ManyRelationshipCommands for Commands<'_, '_> {
    fn add_many_relationship<R: ManyRelationshipType>(
        &mut self,
        source: Entity,
        target: Entity,
    ) {
        self.queue(AddManyRelationshipCommand::<R> {
            source,
            target,
            _marker: std::marker::PhantomData,
        });
    }

    fn remove_many_relationship<R: ManyRelationshipType>(
        &mut self,
        source: Entity,
        target: Entity,
    ) {
        self.queue(RemoveManyRelationshipCommand::<R> {
            source,
            target,
            _marker: std::marker::PhantomData,
        });
    }
}

struct AddManyRelationshipCommand<R: ManyRelationshipType> {
    source: Entity,
    target: Entity,
    _marker: std::marker::PhantomData<R>,
}

impl<R: ManyRelationshipType> Command for AddManyRelationshipCommand<R> {
    fn apply(self, world: &mut World) {
        // Check if the relationship already exists to avoid duplicate observer fires
        let already_exists = world
            .get::<OutgoingRelationships<R>>(self.source)
            .is_some_and(|outgoing| outgoing.contains(self.target));

        if already_exists {
            return;
        }

        // Add to source's outgoing set
        if let Some(mut outgoing) = world.get_mut::<OutgoingRelationships<R>>(self.source) {
            outgoing.insert(self.target);
        } else {
            let mut outgoing = OutgoingRelationships::<R>::new();
            outgoing.insert(self.target);
            world.entity_mut(self.source).insert(outgoing);
        }

        // Add to target's incoming set
        if let Some(mut incoming) = world.get_mut::<IncomingRelationships<R>>(self.target) {
            incoming.insert(self.source);
        } else {
            let mut incoming = IncomingRelationships::<R>::new();
            incoming.insert(self.source);
            world.entity_mut(self.target).insert(incoming);
        }

        // Fire observer event
        world.trigger(OnManyRelationshipAdded::<R>::new(self.source, self.target));
    }
}

struct RemoveManyRelationshipCommand<R: ManyRelationshipType> {
    source: Entity,
    target: Entity,
    _marker: std::marker::PhantomData<R>,
}

impl<R: ManyRelationshipType> Command for RemoveManyRelationshipCommand<R> {
    fn apply(self, world: &mut World) {
        // Check existence first
        let exists = world
            .get::<OutgoingRelationships<R>>(self.source)
            .is_some_and(|outgoing| outgoing.contains(self.target));

        if !exists {
            return;
        }

        // Remove from source's outgoing set
        let source_empty = if let Some(mut outgoing) =
            world.get_mut::<OutgoingRelationships<R>>(self.source)
        {
            outgoing.remove(&self.target);
            outgoing.is_empty()
        } else {
            false
        };

        if source_empty {
            world
                .entity_mut(self.source)
                .remove::<OutgoingRelationships<R>>();
        }

        // Remove from target's incoming set
        let target_empty = if let Some(mut incoming) =
            world.get_mut::<IncomingRelationships<R>>(self.target)
        {
            incoming.remove(&self.source);
            incoming.is_empty()
        } else {
            false
        };

        if target_empty {
            world
                .entity_mut(self.target)
                .remove::<IncomingRelationships<R>>();
        }

        // Fire observer event
        world.trigger(OnManyRelationshipRemoved::<R>::new(self.source, self.target));
    }
}
