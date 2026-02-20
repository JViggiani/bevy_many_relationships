use bevy::prelude::*;

use crate::prelude::*;

#[derive(ManyRelationship)]
struct KnownContact;

#[derive(ManyRelationship)]
struct FactionAlly;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(ManyRelationshipsPlugin);
    register_many_relationship::<KnownContact>(&mut app);
    register_many_relationship::<FactionAlly>(&mut app);
    app
}

/// GIVEN an app with ManyRelationshipsPlugin and KnownContact registered
/// WHEN add_many_relationship::<KnownContact>(a, b) is called and app.update() runs
/// THEN entity a has OutgoingRelationships<KnownContact> containing b
/// AND entity b has IncomingRelationships<KnownContact> containing a
#[test]
fn add_relationship_creates_outgoing_and_incoming() {
    let mut app = test_app();

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .add_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    let outgoing = app.world().get::<OutgoingRelationships<KnownContact>>(a).unwrap();
    assert!(outgoing.contains(b));
    assert_eq!(outgoing.len(), 1);

    let incoming = app.world().get::<IncomingRelationships<KnownContact>>(b).unwrap();
    assert!(incoming.contains(a));
    assert_eq!(incoming.len(), 1);
}

/// GIVEN entity a has OutgoingRelationships<KnownContact> containing {b, c}
/// WHEN remove_many_relationship::<KnownContact>(a, b) is called and app.update() runs
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
        .add_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .add_many_relationship::<KnownContact>(a, c);
    app.world_mut().flush();

    // Verify both exist
    let outgoing = app.world().get::<OutgoingRelationships<KnownContact>>(a).unwrap();
    assert_eq!(outgoing.len(), 2);

    // Remove only a→b
    app.world_mut()
        .commands()
        .remove_many_relationship::<KnownContact>(a, b);
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
/// WHEN remove_many_relationship::<KnownContact>(a, b) is called and app.update() runs
/// THEN entity a no longer has OutgoingRelationships<KnownContact> component
/// AND entity b no longer has IncomingRelationships<KnownContact> component
#[test]
fn remove_last_relationship_removes_components() {
    let mut app = test_app();

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .add_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .remove_many_relationship::<KnownContact>(a, b);
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

/// GIVEN two relationship types KnownContact and FactionAlly are registered
/// WHEN add_many_relationship::<KnownContact>(a, b) and add_many_relationship::<FactionAlly>(a, c)
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
        .add_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .add_many_relationship::<FactionAlly>(a, c);
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
/// WHEN add_many_relationship::<KnownContact>(a, b) is called and app.update() runs
/// THEN the observer fires with source=a and target=b
#[test]
fn observer_fires_on_add() {
    let mut app = test_app();

    #[derive(Resource, Default)]
    struct ObservedAdds(Vec<(Entity, Entity)>);
    app.init_resource::<ObservedAdds>();

    app.add_observer(
        |trigger: On<OnManyRelationshipAdded<KnownContact>>,
         mut observed: ResMut<ObservedAdds>| {
            let event = trigger.event();
            observed.0.push((event.source, event.target));
        },
    );

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .add_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    let observed = app.world().resource::<ObservedAdds>();
    assert_eq!(observed.0.len(), 1);
    assert_eq!(observed.0[0], (a, b));
}

/// GIVEN entity a has OutgoingRelationships<KnownContact> containing {b}
/// AND an observer registered for OnManyRelationshipRemoved<KnownContact>
/// WHEN remove_many_relationship::<KnownContact>(a, b) is called and app.update() runs
/// THEN the observer fires with source=a and target=b
#[test]
fn observer_fires_on_remove() {
    let mut app = test_app();

    #[derive(Resource, Default)]
    struct ObservedRemoves(Vec<(Entity, Entity)>);
    app.init_resource::<ObservedRemoves>();

    app.add_observer(
        |trigger: On<OnManyRelationshipRemoved<KnownContact>>,
         mut observed: ResMut<ObservedRemoves>| {
            let event = trigger.event();
            observed.0.push((event.source, event.target));
        },
    );

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .add_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .remove_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    let observed = app.world().resource::<ObservedRemoves>();
    assert_eq!(observed.0.len(), 1);
    assert_eq!(observed.0[0], (a, b));
}

/// GIVEN add_many_relationship::<KnownContact>(a, b) is called twice
/// WHEN app.update() runs
/// THEN entity a has OutgoingRelationships<KnownContact> containing {b} (no duplicates)
#[test]
fn duplicate_add_is_idempotent() {
    let mut app = test_app();

    #[derive(Resource, Default)]
    struct ObservedAdds(u32);
    app.init_resource::<ObservedAdds>();

    app.add_observer(
        |_trigger: On<OnManyRelationshipAdded<KnownContact>>,
         mut observed: ResMut<ObservedAdds>| {
            observed.0 += 1;
        },
    );

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .add_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .add_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    let outgoing = app.world().get::<OutgoingRelationships<KnownContact>>(a).unwrap();
    assert_eq!(outgoing.len(), 1);
    assert!(outgoing.contains(b));

    let incoming = app.world().get::<IncomingRelationships<KnownContact>>(b).unwrap();
    assert_eq!(incoming.len(), 1);
    assert!(incoming.contains(a));

    // Observer fired only once (the second add was a no-op)
    let observed = app.world().resource::<ObservedAdds>();
    assert_eq!(observed.0, 1);
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
        .add_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .add_many_relationship::<KnownContact>(a, c);
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
        .add_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .add_many_relationship::<KnownContact>(c, b);
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
        .add_many_relationship::<KnownContact>(a, b);
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
    struct ObservedRemoves(u32);
    app.init_resource::<ObservedRemoves>();

    app.add_observer(
        |_trigger: On<OnManyRelationshipRemoved<KnownContact>>,
         mut observed: ResMut<ObservedRemoves>| {
            observed.0 += 1;
        },
    );

    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();

    app.world_mut()
        .commands()
        .remove_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    let observed = app.world().resource::<ObservedRemoves>();
    assert_eq!(observed.0, 0);
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
        .add_many_relationship::<KnownContact>(a, b);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .add_many_relationship::<KnownContact>(a, c);
    app.world_mut().flush();

    app.world_mut()
        .commands()
        .add_many_relationship::<KnownContact>(b, c);
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
