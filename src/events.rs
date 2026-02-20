use bevy::prelude::*;

use crate::ManyRelationshipType;

/// Event fired when a many-to-many relationship of type `R` is added.
#[derive(Event)]
pub struct OnManyRelationshipAdded<R: ManyRelationshipType> {
    /// The source entity (has the outgoing relationship).
    pub source: Entity,
    /// The target entity (has the incoming relationship).
    pub target: Entity,
    _marker: std::marker::PhantomData<R>,
}

impl<R: ManyRelationshipType> OnManyRelationshipAdded<R> {
    pub(crate) fn new(source: Entity, target: Entity) -> Self {
        Self {
            source,
            target,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Event fired when a many-to-many relationship of type `R` is removed.
#[derive(Event)]
pub struct OnManyRelationshipRemoved<R: ManyRelationshipType> {
    /// The source entity (had the outgoing relationship).
    pub source: Entity,
    /// The target entity (had the incoming relationship).
    pub target: Entity,
    _marker: std::marker::PhantomData<R>,
}

impl<R: ManyRelationshipType> OnManyRelationshipRemoved<R> {
    pub(crate) fn new(source: Entity, target: Entity) -> Self {
        Self {
            source,
            target,
            _marker: std::marker::PhantomData,
        }
    }
}
