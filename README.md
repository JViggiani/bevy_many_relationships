# bevy_many_relationships

A Bevy plugin for many-to-many relationships between entities.

Bevy's built-in `Relationship` trait supports one-to-one relationships (each entity has at most one component of a given relationship type). Many game mechanics require many-to-many: a character knows many NPCs, an NPC is known by many characters. This library provides that.

## Usage

Define a relationship type as a plain unit struct:

```rust
struct KnownContact;
```

Register it in your plugin:

```rust
fn build(&self, app: &mut App) {
    app.add_plugins(ManyRelationshipsPlugin);
    register_many_relationship::<KnownContact>(app);
}
```

Add and remove relationships via commands:

```rust
fn my_system(mut commands: Commands) {
    let a = commands.spawn_empty().id();
    let b = commands.spawn_empty().id();

    commands.add_many_relationship::<KnownContact>(a, b);
    // Later...
    commands.remove_many_relationship::<KnownContact>(a, b);
}
```

Query relationships:

```rust
fn query_system(query: Query<&OutgoingRelationships<KnownContact>>) {
    for outgoing in &query {
        for target in outgoing.targets() {
            // ...
        }
    }
}
```

React to changes with observers:

```rust
app.add_observer(|trigger: On<OnManyRelationshipAdded<KnownContact>>| {
    let event = trigger.event();
    println!("Relationship added: {:?} -> {:?}", event.source, event.target);
});
```

## License

MIT
