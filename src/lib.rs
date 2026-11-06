mod commands;
mod components;
mod events;

pub use commands::{
    ManyRelatedEntityCommands, ManyRelatedSpawnerCommands, add_many_relationship,
    get_relationship_payload, remove_many_relationship, set_many_relationship,
};
pub use components::{AddOutgoingRelationships, IncomingRelationships, OutgoingRelationships};
pub use events::{OnManyRelationshipAdded, OnManyRelationshipRemoved};

/// Convenience re-exports for typical `use bevy_many_relationships::prelude::*` imports.
pub mod prelude {
    pub use super::{
        AddOutgoingRelationships, IncomingRelationships, ManyRelatedEntityCommands,
        ManyRelatedSpawnerCommands, OnManyRelationshipAdded,
        OnManyRelationshipRemoved, OutgoingRelationships, add_many_relationship,
        get_relationship_payload, remove_many_relationship, set_many_relationship,
    };
}

#[cfg(test)]
mod tests;
