# bevy_many_relationships

A Bevy plugin for many-to-many relationships between entities.

Bevy's built-in `Relationship` trait supports one-to-one relationships (each entity has at most one component of a given relationship type). Many game mechanics require many-to-many: a character knows many NPCs, an NPC is known by many characters. This library provides that.

## Bevy Version Support

| Bevy | bevy_many_relationships |
| --- | --- |
| 0.11 | 0.1.x |
| 0.18 | 0.3.x |
| 0.19 | 0.2.x (yanked) |

## Concepts

- `OutgoingRelationships<R>` stores `source -> target` edges for relationship payload type `R`.
- `IncomingRelationships<R>` stores the reverse index of sources for each target.
- `add_*` methods are insert-only (existing payload is preserved).
- `set_*` methods are upsert/overwrite.
- Entity command methods are deferred, so usage is: queue commands, then `flush`/`apply_deferred`, then query or react via events.

## Usage

Define a relationship type as a plain unit struct:

```rust
struct KnownContact;
```

Enable the plugin:

```rust
fn build(&self, app: &mut App) {
    app.add_plugins(ManyRelationshipsPlugin);
}
```

Add and remove relationships via entity commands:

```rust
fn my_system(mut commands: Commands) {
    let a = commands.spawn_empty().id();
    let b = commands.spawn_empty().id();

    commands
        .entity(b)
        .set_incoming_from::<KnownContact>(a, KnownContact);
    // Later...
    commands
        .entity(b)
        .remove_many_related::<KnownContact>(&[a]);
}
```

### Marker-style relationships (unit structs)

Use a unit struct when you only care that an edge exists.

```rust
#[derive(Default)]
struct KnownContact;

commands
    .entity(target)
    .add_incoming_from::<KnownContact>(source, KnownContact);
```

### Payload relationships (metadata per edge)

Use any payload struct to attach data directly to each `source -> target` edge.

```rust
#[derive(Clone)]
struct KnownContactDetails {
    known_party_email: Option<String>,
    known_party_phone: Option<String>,
}

commands
    .entity(source)
    .set_outgoing_to::<KnownContactDetails>(
        target,
        KnownContactDetails {
            known_party_email: Some("bob@example.com".to_string()),
            known_party_phone: Some("+1-555-0100".to_string()),
        },
    );
```

Directional command aliases for clearer intent:

```rust
// source -> target
commands.entity(source).add_outgoing_to::<KnownContact>(target, KnownContact);
commands.entity(source).remove_outgoing_to::<KnownContact>(target);

// equivalent target-side wording
commands.entity(target).add_incoming_from::<KnownContact>(source, KnownContact);
commands.entity(target).remove_incoming_from::<KnownContact>(source);
```

### `add` vs `set` semantics

```rust
// insert-only: does not overwrite existing payload
commands.entity(source).add_outgoing_to::<KnownContactDetails>(target, first_payload);
commands.entity(source).add_outgoing_to::<KnownContactDetails>(target, second_payload);

// upsert: overwrites existing payload
commands.entity(source).set_outgoing_to::<KnownContactDetails>(target, replacement_payload);
```

Bevy-style construction on spawn/insert:

```rust
let alice = commands.spawn(Name::new("Alice")).id();
commands.spawn((
    Name::new("Bob"),
    AddOutgoingRelationships::<KnownContact>::one(alice, KnownContact),
));

// multiple targets at once
commands.spawn((
    Name::new("Charlie"),
    AddOutgoingRelationships::<KnownContact>::new([
        (alice, KnownContact),
        (bob, KnownContact),
    ]),
));
```

### Spawner helpers for creating related source entities

```rust
commands
    .entity(target)
    .with_many_related_entities::<KnownContact>(|spawner| {
        spawner.spawn(Name::new("source-a"));
        spawner.spawn(Name::new("source-b"));
    });
```

### Detaching all incoming links to a target

```rust
commands
    .entity(target)
    .detach_all_many_related::<KnownContact>();
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

Query payload data from outgoing edges:

```rust
fn payload_query_system(query: Query<&OutgoingRelationships<KnownContactDetails>>) {
    for outgoing in &query {
        for (target, details) in outgoing.iter() {
            let _ = (target, &details.known_party_email);
        }
    }
}
```

## Deferred command behavior

`EntityCommands` are deferred in Bevy. Relationship mutations become visible after command application (`flush` in tests, schedule/apply deferred in app runtime).

When you need to know whether a mutation happened, validate via resulting world state or observers:

```rust
app.add_observer(|trigger: On<OnManyRelationshipAdded<KnownContact>>| {
    let event = trigger.event();
    let _ = (event.source, event.target);
});
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
