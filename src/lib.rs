mod commands;
mod components;
mod events;
mod plugin;

pub use commands::ManyRelationshipCommands;
pub use components::{IncomingRelationships, OutgoingRelationships};
pub use events::{OnManyRelationshipAdded, OnManyRelationshipRemoved};
pub use plugin::{ManyRelationshipsPlugin, register_many_relationship};

pub mod prelude {
    pub use crate::commands::ManyRelationshipCommands;
    pub use crate::components::{IncomingRelationships, OutgoingRelationships};
    pub use crate::events::{OnManyRelationshipAdded, OnManyRelationshipRemoved};
    pub use crate::plugin::{ManyRelationshipsPlugin, register_many_relationship};
}

#[cfg(test)]
mod tests;
