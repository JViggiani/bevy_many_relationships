mod commands;
mod components;
mod events;
mod plugin;

pub use commands::{ManyRelatedEntityCommands, ManyRelatedSpawnerCommands};
pub use commands::{add_many_relationship, remove_many_relationship, set_many_relationship};
pub use components::{AddOutgoingRelationships, IncomingRelationships, OutgoingRelationships};
pub use events::{OnManyRelationshipAdded, OnManyRelationshipRemoved};
pub use plugin::ManyRelationshipsPlugin;

pub mod prelude {
    pub use crate::commands::{ManyRelatedEntityCommands, ManyRelatedSpawnerCommands};
    pub use crate::commands::{add_many_relationship, remove_many_relationship, set_many_relationship};
    pub use crate::components::{AddOutgoingRelationships, IncomingRelationships, OutgoingRelationships};
    pub use crate::events::{OnManyRelationshipAdded, OnManyRelationshipRemoved};
    pub use crate::plugin::ManyRelationshipsPlugin;
}

#[cfg(test)]
mod tests;
