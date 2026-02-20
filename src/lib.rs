// Required so the derive macro can resolve `::bevy_many_relationships::` paths
// when used within this crate's own tests.
extern crate self as bevy_many_relationships;

mod commands;
mod components;
mod events;
mod plugin;

pub use bevy_many_relationships_derive::ManyRelationship;
pub use commands::ManyRelationshipCommands;
pub use components::{IncomingRelationships, OutgoingRelationships};
pub use events::{OnManyRelationshipAdded, OnManyRelationshipRemoved};
pub use plugin::{ManyRelationshipsPlugin, register_many_relationship};

/// Marker trait for many-to-many relationship types.
///
/// Implement this via `#[derive(ManyRelationship)]` on a unit struct.
pub trait ManyRelationshipType: Send + Sync + 'static {
    /// Returns the name of this relationship type (used for debugging).
    fn relationship_name() -> &'static str;
}

pub mod prelude {
    pub use crate::ManyRelationship;
    pub use crate::ManyRelationshipType;
    pub use crate::commands::ManyRelationshipCommands;
    pub use crate::components::{IncomingRelationships, OutgoingRelationships};
    pub use crate::events::{OnManyRelationshipAdded, OnManyRelationshipRemoved};
    pub use crate::plugin::{ManyRelationshipsPlugin, register_many_relationship};
}

#[cfg(test)]
mod tests;
