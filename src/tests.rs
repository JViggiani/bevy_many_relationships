use bevy::prelude::*;

use crate::prelude::*;

#[derive(Default)]
struct KnownContact;

#[derive(Default)]
struct FactionAlly;

#[derive(Clone, Debug, PartialEq, Eq)]
struct KnownContactDetails {
    established_at_unix: Option<i64>,
    known_party_phone: Option<String>,
    known_party_email: Option<String>,
    known_party_notes: Option<String>,
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(ManyRelationshipsPlugin);
    app
}

/// GIVEN an app with ManyRelationshipsPlugin
/// WHEN entity(b).add_one_many_related::<KnownContact>(a) is called and app.update() runs
/// THEN entity a has OutgoingRelationships<KnownContact> containing b
/// AND entity b has IncomingRelationships<KnownContact> containing a
#[test]
fn add_relationship_creates_outgoing_and_incoming() {
    let mut app = test_app();

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    let outgoing = app.world().get::<OutgoingRelationships<KnownContact>>(a).unwrap();
    assert!(outgoing.contains(b));
    assert_eq!(outgoing.len(), 1);

    let incoming = app.world().get::<IncomingRelationships<KnownContact>>(b).unwrap();
    assert!(incoming.contains(a));
    assert_eq!(incoming.len(), 1);
}

/// GIVEN entity a has OutgoingRelationships<KnownContact> containing {b, c}
/// WHEN entity(b).remove_many_related::<KnownContact>(&[a]) is called and app.update() runs
/// THEN entity a has OutgoingRelationships<KnownContact> containing {c} only
/// AND entity b no longer has IncomingRelationships<KnownContact> component (empty → removed)
#[test]
fn remove_one_of_two_targets_keeps_remaining() {
    let mut app = test_app();

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let c = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(c)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    // Verify both exist
    let outgoing = app.world().get::<OutgoingRelationships<KnownContact>>(a).unwrap();
    assert_eq!(outgoing.len(), 2);

    // Remove only a→b
    app.world_mut()
        .commands()
        .entity(b)
        .remove_many_related::<KnownContact>(&[a]);
    app.world_mut().flush();

    // a still has outgoing to c
    let outgoing = app.world().get::<OutgoingRelationships<KnownContact>>(a).unwrap();
    assert!(!outgoing.contains(b));
    assert!(outgoing.contains(c));
    assert_eq!(outgoing.len(), 1);

    // b no longer has incoming component (was the only source)
    assert!(
        app.world()
            .get::<IncomingRelationships<KnownContact>>(b)
            .is_none()
    );

    // c still has incoming from a
    let incoming = app.world().get::<IncomingRelationships<KnownContact>>(c).unwrap();
    assert!(incoming.contains(a));
}

/// GIVEN entity a has OutgoingRelationships<KnownContact> containing {b}
/// WHEN entity(b).remove_many_related::<KnownContact>(&[a]) is called and app.update() runs
/// THEN entity a no longer has OutgoingRelationships<KnownContact> component
/// AND entity b no longer has IncomingRelationships<KnownContact> component
#[test]
fn remove_last_relationship_removes_components() {
    let mut app = test_app();

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(b)
        .remove_many_related::<KnownContact>(&[a]);
    app.world_mut().flush();

    assert!(
        app.world()
            .get::<OutgoingRelationships<KnownContact>>(a)
            .is_none()
    );
    assert!(
        app.world()
            .get::<IncomingRelationships<KnownContact>>(b)
            .is_none()
    );
}

/// GIVEN two relationship types KnownContact and FactionAlly are used
/// WHEN entity(b).add_one_many_related::<KnownContact>(a) and entity(c).add_one_many_related::<FactionAlly>(a)
///      are called
/// THEN entity a has OutgoingRelationships<KnownContact> containing {b}
/// AND entity a has OutgoingRelationships<FactionAlly> containing {c}
/// AND the two relationship types do not interfere with each other
#[test]
fn different_relationship_types_are_independent() {
    let mut app = test_app();

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let c = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(c)
        .add_one_many_related::<FactionAlly>(a);
    app.world_mut().flush();

    let known = app
        .world()
        .get::<OutgoingRelationships<KnownContact>>(a)
        .unwrap();
    assert!(known.contains(b));
    assert!(!known.contains(c));
    assert_eq!(known.len(), 1);

    let faction = app
        .world()
        .get::<OutgoingRelationships<FactionAlly>>(a)
        .unwrap();
    assert!(faction.contains(c));
    assert!(!faction.contains(b));
    assert_eq!(faction.len(), 1);

    // b only has incoming KnownContact, not FactionAlly
    assert!(
        app.world()
            .get::<IncomingRelationships<KnownContact>>(b)
            .is_some()
    );
    assert!(
        app.world()
            .get::<IncomingRelationships<FactionAlly>>(b)
            .is_none()
    );

    // c only has incoming FactionAlly, not KnownContact
    assert!(
        app.world()
            .get::<IncomingRelationships<FactionAlly>>(c)
            .is_some()
    );
    assert!(
        app.world()
            .get::<IncomingRelationships<KnownContact>>(c)
            .is_none()
    );
}

/// GIVEN an observer registered for OnManyRelationshipAdded<KnownContact>
/// WHEN entity(b).add_one_many_related::<KnownContact>(a) is called and app.update() runs
/// THEN the observer fires with source=a and target=b
#[test]
fn observer_fires_on_add() {
    let mut app = test_app();

    #[derive(Resource, Default)]
    struct ObservedAdds {
        pairs: Vec<(Entity, Entity)>,
    }
    app.init_resource::<ObservedAdds>();

    app.add_observer(
        |trigger: On<OnManyRelationshipAdded<KnownContact>>,
         mut observed: ResMut<ObservedAdds>| {
            let event = trigger.event();
            observed.pairs.push((event.source, event.target));
        },
    );

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    let observed = app.world().resource::<ObservedAdds>();
    assert_eq!(observed.pairs.len(), 1);
    assert_eq!(observed.pairs[0], (a, b));
}

/// GIVEN entity a has OutgoingRelationships<KnownContact> containing {b}
/// AND an observer registered for OnManyRelationshipRemoved<KnownContact>
/// WHEN entity(b).remove_many_related::<KnownContact>(&[a]) is called and app.update() runs
/// THEN the observer fires with source=a and target=b
#[test]
fn observer_fires_on_remove() {
    let mut app = test_app();

    #[derive(Resource, Default)]
    struct ObservedRemoves {
        pairs: Vec<(Entity, Entity)>,
    }
    app.init_resource::<ObservedRemoves>();

    app.add_observer(
        |trigger: On<OnManyRelationshipRemoved<KnownContact>>,
         mut observed: ResMut<ObservedRemoves>| {
            let event = trigger.event();
            observed.pairs.push((event.source, event.target));
        },
    );

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(b)
        .remove_many_related::<KnownContact>(&[a]);
    app.world_mut().flush();

    let observed = app.world().resource::<ObservedRemoves>();
    assert_eq!(observed.pairs.len(), 1);
    assert_eq!(observed.pairs[0], (a, b));
}

/// GIVEN entity(b).add_one_many_related::<KnownContact>(a) is called twice
/// WHEN app.update() runs
/// THEN entity a has OutgoingRelationships<KnownContact> containing {b} (no duplicates)
#[test]
fn duplicate_add_is_idempotent() {
    let mut app = test_app();

    #[derive(Resource, Default)]
    struct ObservedAdds {
        count: u32,
    }
    app.init_resource::<ObservedAdds>();

    app.add_observer(
        |_trigger: On<OnManyRelationshipAdded<KnownContact>>,
         mut observed: ResMut<ObservedAdds>| {
            observed.count += 1;
        },
    );

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    let outgoing = app.world().get::<OutgoingRelationships<KnownContact>>(a).unwrap();
    assert_eq!(outgoing.len(), 1);
    assert!(outgoing.contains(b));

    let incoming = app.world().get::<IncomingRelationships<KnownContact>>(b).unwrap();
    assert_eq!(incoming.len(), 1);
    assert!(incoming.contains(a));

    // Observer fired only once (the second add was a no-op)
    let observed = app.world().resource::<ObservedAdds>();
    assert_eq!(observed.count, 1);
}

/// GIVEN entity a has relationships to b and c
/// WHEN entity b is despawned
/// THEN entity a's OutgoingRelationships<KnownContact> no longer contains b
/// AND entity a's OutgoingRelationships<KnownContact> still contains c
#[test]
fn despawning_target_cleans_up_source_outgoing() {
    let mut app = test_app();

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let c = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(c)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    // Despawn b
    app.world_mut().despawn(b);

    // a should still have outgoing to c but not b
    let outgoing = app.world().get::<OutgoingRelationships<KnownContact>>(a).unwrap();
    assert!(!outgoing.contains(b));
    assert!(outgoing.contains(c));
    assert_eq!(outgoing.len(), 1);
}

/// GIVEN entity b is a target of relationships from a and c
/// WHEN entity a is despawned
/// THEN entity b's IncomingRelationships<KnownContact> no longer contains a
/// AND entity b's IncomingRelationships<KnownContact> still contains c
#[test]
fn despawning_source_cleans_up_target_incoming() {
    let mut app = test_app();

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let c = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(c);
    app.world_mut().flush();

    // Despawn a
    app.world_mut().despawn(a);

    // b should still have incoming from c but not a
    let incoming = app.world().get::<IncomingRelationships<KnownContact>>(b).unwrap();
    assert!(!incoming.contains(a));
    assert!(incoming.contains(c));
    assert_eq!(incoming.len(), 1);
}

/// GIVEN entity a has a single outgoing relationship to b
/// WHEN entity b is despawned
/// THEN entity a no longer has OutgoingRelationships<KnownContact> component (empty → removed)
#[test]
fn despawning_only_target_removes_outgoing_component() {
    let mut app = test_app();

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    app.world_mut().despawn(b);

    assert!(
        app.world()
            .get::<OutgoingRelationships<KnownContact>>(a)
            .is_none()
    );
}

/// GIVEN a remove is called for a relationship that doesn't exist
/// WHEN app.update() runs
/// THEN no panic occurs and no observer fires
#[test]
fn removing_nonexistent_relationship_is_noop() {
    let mut app = test_app();

    #[derive(Resource, Default)]
    struct ObservedRemoves {
        count: u32,
    }
    app.init_resource::<ObservedRemoves>();

    app.add_observer(
        |_trigger: On<OnManyRelationshipRemoved<KnownContact>>,
         mut observed: ResMut<ObservedRemoves>| {
            observed.count += 1;
        },
    );

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .remove_many_related::<KnownContact>(&[a]);
    app.world_mut().flush();

    let observed = app.world().resource::<ObservedRemoves>();
    assert_eq!(observed.count, 0);
}

/// GIVEN multiple relationships exist between different entities
/// WHEN all are queried via standard Bevy Query
/// THEN all relationships are correctly visible
#[test]
fn queryable_via_standard_bevy_query() {
    let mut app = test_app();

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let c = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(b)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(c)
        .add_one_many_related::<KnownContact>(a);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(c)
        .add_one_many_related::<KnownContact>(b);
    app.world_mut().flush();

    // Query all entities with outgoing KnownContact relationships
    let mut query = app
        .world_mut()
        .query::<(Entity, &OutgoingRelationships<KnownContact>)>();
    let results: Vec<_> = query.iter(app.world()).collect();

    assert_eq!(results.len(), 2); // a and b have outgoing

    for (entity, outgoing) in &results {
        if *entity == a {
            assert_eq!(outgoing.len(), 2);
            assert!(outgoing.contains(b));
            assert!(outgoing.contains(c));
        } else if *entity == b {
            assert_eq!(outgoing.len(), 1);
            assert!(outgoing.contains(c));
        }
    }
}

/// GIVEN an entity target and another source entity
/// WHEN source is linked using EntityCommands::add_one_many_related
/// THEN outgoing/incoming components are updated as expected
#[test]
fn entity_commands_add_one_many_related_works() {
    let mut app = test_app();

    let target = app.world_mut().spawn_empty().id();
    let source = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(target)
        .add_one_many_related::<KnownContact>(source);
    app.world_mut().flush();

    let outgoing = app
        .world()
        .get::<OutgoingRelationships<KnownContact>>(source)
        .unwrap();
    assert!(outgoing.contains(target));

    let incoming = app
        .world()
        .get::<IncomingRelationships<KnownContact>>(target)
        .unwrap();
    assert!(incoming.contains(source));
}

/// GIVEN an entity target
/// WHEN with_many_related is used
/// THEN a new source entity is spawned and linked to the target
#[test]
fn with_many_related_spawns_and_links() {
    let mut app = test_app();

    let target = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(target)
        .with_many_related::<KnownContact>(Name::new("spawned-source"));
    app.world_mut().flush();

    let incoming = app
        .world()
        .get::<IncomingRelationships<KnownContact>>(target)
        .unwrap();
    assert_eq!(incoming.len(), 1);
}

/// GIVEN an entity target
/// WHEN with_many_related_entities is used to spawn two entities
/// THEN both entities are linked to the target
#[test]
fn with_many_related_entities_spawns_and_links_multiple() {
    let mut app = test_app();

    let target = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(target)
        .with_many_related_entities::<KnownContact>(|spawner| {
            spawner.spawn(Name::new("source-a"));
            spawner.spawn(Name::new("source-b"));
        });
    app.world_mut().flush();

    let incoming = app
        .world()
        .get::<IncomingRelationships<KnownContact>>(target)
        .unwrap();
    assert_eq!(incoming.len(), 2);
}

#[test]
fn add_many_related_links_multiple_sources() {
    let mut app = test_app();

    let target = app.world_mut().spawn_empty().id();
    let source_a = app.world_mut().spawn_empty().id();
    let source_b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(target)
        .add_many_related::<KnownContact>(&[source_a, source_b]);
    app.world_mut().flush();

    let incoming = app
        .world()
        .get::<IncomingRelationships<KnownContact>>(target)
        .unwrap();
    assert_eq!(incoming.len(), 2);
    assert!(incoming.contains(source_a));
    assert!(incoming.contains(source_b));

    let outgoing_a = app
        .world()
        .get::<OutgoingRelationships<KnownContact>>(source_a)
        .unwrap();
    assert!(outgoing_a.contains(target));

    let outgoing_b = app
        .world()
        .get::<OutgoingRelationships<KnownContact>>(source_b)
        .unwrap();
    assert!(outgoing_b.contains(target));
}

#[test]
fn detach_all_many_related_removes_all_links_to_target() {
    let mut app = test_app();

    let target = app.world_mut().spawn_empty().id();
    let source_a = app.world_mut().spawn_empty().id();
    let source_b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(target)
        .add_many_related::<KnownContact>(&[source_a, source_b]);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(target)
        .detach_all_many_related::<KnownContact>();
    app.world_mut().flush();

    assert!(
        app.world()
            .get::<IncomingRelationships<KnownContact>>(target)
            .is_none()
    );
    assert!(
        app.world()
            .get::<OutgoingRelationships<KnownContact>>(source_a)
            .is_none()
    );
    assert!(
        app.world()
            .get::<OutgoingRelationships<KnownContact>>(source_b)
            .is_none()
    );
}

#[test]
fn add_incoming_from_creates_source_to_target_edge() {
    let mut app = test_app();

    let target = app.world_mut().spawn_empty().id();
    let source = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(target)
        .add_incoming_from::<KnownContact>(source, KnownContact);
    app.world_mut().flush();

    let outgoing = app
        .world()
        .get::<OutgoingRelationships<KnownContact>>(source)
        .unwrap();
    assert!(outgoing.contains(target));

    let incoming = app
        .world()
        .get::<IncomingRelationships<KnownContact>>(target)
        .unwrap();
    assert!(incoming.contains(source));
}

#[test]
fn add_outgoing_to_creates_source_to_target_edge() {
    let mut app = test_app();

    let source = app.world_mut().spawn_empty().id();
    let target = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(source)
        .add_outgoing_to::<KnownContact>(target, KnownContact);
    app.world_mut().flush();

    let outgoing = app
        .world()
        .get::<OutgoingRelationships<KnownContact>>(source)
        .unwrap();
    assert!(outgoing.contains(target));

    let incoming = app
        .world()
        .get::<IncomingRelationships<KnownContact>>(target)
        .unwrap();
    assert!(incoming.contains(source));
}

#[test]
fn remove_incoming_from_removes_source_to_target_edge() {
    let mut app = test_app();

    let source = app.world_mut().spawn_empty().id();
    let target = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(source)
        .add_outgoing_to::<KnownContact>(target, KnownContact);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(target)
        .remove_incoming_from::<KnownContact>(source);
    app.world_mut().flush();

    assert!(
        app.world()
            .get::<OutgoingRelationships<KnownContact>>(source)
            .is_none()
    );
    assert!(
        app.world()
            .get::<IncomingRelationships<KnownContact>>(target)
            .is_none()
    );
}

#[test]
fn remove_outgoing_to_removes_source_to_target_edge() {
    let mut app = test_app();

    let source = app.world_mut().spawn_empty().id();
    let target = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(source)
        .add_outgoing_to::<KnownContact>(target, KnownContact);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(source)
        .remove_outgoing_to::<KnownContact>(target);
    app.world_mut().flush();

    assert!(
        app.world()
            .get::<OutgoingRelationships<KnownContact>>(source)
            .is_none()
    );
    assert!(
        app.world()
            .get::<IncomingRelationships<KnownContact>>(target)
            .is_none()
    );
}

#[test]
fn bevy_style_spawn_with_add_outgoing_relationships_component_works() {
    let mut app = test_app();

    let alice = app.world_mut().spawn(Name::new("Alice")).id();
    let bob = app
        .world_mut()
        .spawn((
            Name::new("Bob"),
            AddOutgoingRelationships::<KnownContact>::one(alice, KnownContact),
        ))
        .id();

    app.world_mut().flush();

    let outgoing = app
        .world()
        .get::<OutgoingRelationships<KnownContact>>(bob)
        .unwrap();
    assert!(outgoing.contains(alice));

    let incoming = app
        .world()
        .get::<IncomingRelationships<KnownContact>>(alice)
        .unwrap();
    assert!(incoming.contains(bob));

    assert!(
        app.world()
            .get::<AddOutgoingRelationships<KnownContact>>(bob)
            .is_none()
    );
}

#[test]
fn bevy_style_insert_add_outgoing_relationships_component_works() {
    let mut app = test_app();

    let alice = app.world_mut().spawn_empty().id();
    let bob = app.world_mut().spawn_empty().id();

    app.world_mut().entity_mut(bob).insert(
        AddOutgoingRelationships::<KnownContact>::new([(alice, KnownContact)]),
    );
    app.world_mut().flush();

    let outgoing = app
        .world()
        .get::<OutgoingRelationships<KnownContact>>(bob)
        .unwrap();
    assert!(outgoing.contains(alice));

    let incoming = app
        .world()
        .get::<IncomingRelationships<KnownContact>>(alice)
        .unwrap();
    assert!(incoming.contains(bob));
}

#[test]
fn bevy_style_add_outgoing_relationships_supports_multiple_targets() {
    let mut app = test_app();

    let alice = app.world_mut().spawn_empty().id();
    let charlie = app.world_mut().spawn_empty().id();
    let bob = app.world_mut().spawn_empty().id();

    app.world_mut().entity_mut(bob).insert(
        AddOutgoingRelationships::<KnownContact>::new([
            (alice, KnownContact),
            (charlie, KnownContact),
        ]),
    );
    app.world_mut().flush();

    let outgoing = app
        .world()
        .get::<OutgoingRelationships<KnownContact>>(bob)
        .unwrap();
    assert_eq!(outgoing.len(), 2);
    assert!(outgoing.contains(alice));
    assert!(outgoing.contains(charlie));
}

#[test]
fn with_many_related_entities_supports_spawn_empty_and_target_accessor() {
    let mut app = test_app();

    let target = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(target)
        .with_many_related_entities::<KnownContact>(|spawner| {
            assert_eq!(spawner.target_entity(), target);
            let source = spawner.spawn_empty().id();
            spawner
                .commands_mut()
                .entity(source)
                .insert(Name::new("spawn-empty-source"));
        });
    app.world_mut().flush();

    let incoming = app
        .world()
        .get::<IncomingRelationships<KnownContact>>(target)
        .unwrap();
    assert_eq!(incoming.len(), 1);
}

#[test]
fn relationship_payload_get_returns_known_party_details() {
    let mut app = test_app();

    let alice = app.world_mut().spawn_empty().id();
    let bob = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(bob)
        .set_incoming_from::<KnownContactDetails>(
            alice,
            KnownContactDetails {
                established_at_unix: Some(1_708_934_400),
                known_party_phone: Some("+1-555-0100".to_string()),
                known_party_email: Some("bob@example.com".to_string()),
                known_party_notes: Some("Met at conference".to_string()),
            },
        );
    app.world_mut().flush();

    let outgoing = app
        .world()
        .get::<OutgoingRelationships<KnownContactDetails>>(alice)
        .unwrap();
    assert!(outgoing.contains(bob));

    let relationship = outgoing.get(bob).unwrap();
    assert_eq!(relationship.known_party_email.as_deref(), Some("bob@example.com"));
    assert_eq!(relationship.known_party_phone.as_deref(), Some("+1-555-0100"));
}

#[test]
fn command_add_without_bool_validates_via_state_after_flush() {
    let mut app = test_app();

    let source = app.world_mut().spawn_empty().id();
    let target = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(source)
        .add_outgoing_to::<KnownContactDetails>(
            target,
            KnownContactDetails {
                established_at_unix: Some(1),
                known_party_phone: Some("+1-111-0000".to_string()),
                known_party_email: Some("first@example.com".to_string()),
                known_party_notes: None,
            },
        );
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(source)
        .add_outgoing_to::<KnownContactDetails>(
            target,
            KnownContactDetails {
                established_at_unix: Some(2),
                known_party_phone: Some("+1-222-0000".to_string()),
                known_party_email: Some("second@example.com".to_string()),
                known_party_notes: Some("should-not-overwrite".to_string()),
            },
        );
    app.world_mut().flush();

    let outgoing = app
        .world()
        .get::<OutgoingRelationships<KnownContactDetails>>(source)
        .unwrap();
    let relationship = outgoing.get(target).unwrap();
    assert_eq!(relationship.known_party_email.as_deref(), Some("first@example.com"));
}

#[test]
fn command_set_without_bool_validates_via_state_after_flush() {
    let mut app = test_app();

    let source = app.world_mut().spawn_empty().id();
    let target = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .entity(source)
        .set_outgoing_to::<KnownContactDetails>(
            target,
            KnownContactDetails {
                established_at_unix: Some(1),
                known_party_phone: Some("+1-111-0000".to_string()),
                known_party_email: Some("old@example.com".to_string()),
                known_party_notes: None,
            },
        );
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .entity(source)
        .set_outgoing_to::<KnownContactDetails>(
            target,
            KnownContactDetails {
                established_at_unix: Some(2),
                known_party_phone: Some("+1-222-0000".to_string()),
                known_party_email: Some("new@example.com".to_string()),
                known_party_notes: Some("updated".to_string()),
            },
        );
    app.world_mut().flush();

    let outgoing = app
        .world()
        .get::<OutgoingRelationships<KnownContactDetails>>(source)
        .unwrap();
    let relationship = outgoing.get(target).unwrap();
    assert_eq!(relationship.known_party_email.as_deref(), Some("new@example.com"));
}

#[test]
fn set_overwrites_existing_relationship_payload() {
    let mut app = test_app();

    let source = app.world_mut().spawn_empty().id();
    let target = app.world_mut().spawn_empty().id();

    let first_was_new = crate::commands::set_many_relationship::<KnownContactDetails>(
        app.world_mut(),
        source,
        target,
        KnownContactDetails {
            established_at_unix: Some(1),
            known_party_phone: Some("+1-111-0000".to_string()),
            known_party_email: Some("old@example.com".to_string()),
            known_party_notes: None,
        },
    );
    assert!(first_was_new);

    let second_was_new = crate::commands::set_many_relationship::<KnownContactDetails>(
        app.world_mut(),
        source,
        target,
        KnownContactDetails {
            established_at_unix: Some(2),
            known_party_phone: Some("+1-222-0000".to_string()),
            known_party_email: Some("new@example.com".to_string()),
            known_party_notes: Some("updated".to_string()),
        },
    );
    assert!(!second_was_new);

    let outgoing = app
        .world()
        .get::<OutgoingRelationships<KnownContactDetails>>(source)
        .unwrap();
    let relationship = outgoing.get(target).unwrap();
    assert_eq!(relationship.known_party_email.as_deref(), Some("new@example.com"));
    assert_eq!(relationship.known_party_phone.as_deref(), Some("+1-222-0000"));
}

#[test]
fn add_does_not_overwrite_existing_relationship_payload() {
    let mut app = test_app();

    let source = app.world_mut().spawn_empty().id();
    let target = app.world_mut().spawn_empty().id();

    let inserted_first = crate::commands::add_many_relationship::<KnownContactDetails>(
        app.world_mut(),
        source,
        target,
        KnownContactDetails {
            established_at_unix: Some(1),
            known_party_phone: Some("+1-111-0000".to_string()),
            known_party_email: Some("first@example.com".to_string()),
            known_party_notes: None,
        },
    );
    assert!(inserted_first);

    let inserted_second = crate::commands::add_many_relationship::<KnownContactDetails>(
        app.world_mut(),
        source,
        target,
        KnownContactDetails {
            established_at_unix: Some(2),
            known_party_phone: Some("+1-222-0000".to_string()),
            known_party_email: Some("second@example.com".to_string()),
            known_party_notes: Some("should-not-overwrite".to_string()),
        },
    );
    assert!(!inserted_second);

    let outgoing = app
        .world()
        .get::<OutgoingRelationships<KnownContactDetails>>(source)
        .unwrap();
    let relationship = outgoing.get(target).unwrap();
    assert_eq!(relationship.known_party_email.as_deref(), Some("first@example.com"));
}
