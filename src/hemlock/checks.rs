use hemlock_ecs::entity::Entity;
use hemlock_ecs::later::{Later, spawn};
use hemlock_ecs::order::batch;
use hemlock_ecs::resources::Resources;
use hemlock_ecs::schedule::{once, run};
use hemlock_ecs::storage::{
    Storage, attach, changed_at, changed_since, despawn, get, get_mut, has, is_alive, living,
    query, remove, removed_since, require, set, spawn_batch, spawn_with, tick,
};
use hemlock_ecs::system::{
    IntoSystem, Mut, System, View, each, each_mut, each_mut_parallel, each_mut_split, peek_mut,
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Count(u64);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Step(u64);

#[derive(Clone, Copy, Debug, PartialEq, Default)]
struct Needed(u64);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Owner(u64);

#[derive(Clone, Copy)]
struct Marker;

#[derive(Clone, Copy)]
struct Late(u64);

struct Heavy(String);

fn step(mut rows: Mut<(Count,), (&Step,)>) {
    each_mut(&mut rows, |_entity, (mut count,), (step,)| {
        count.0 += step.0;
    });
}

fn bump_the_odd(mut rows: Mut<(Count,)>) {
    each_mut(&mut rows, |_entity, (mut count,), ()| {
        if count.0 % 2 == 1 {
            count.0 += 2;
        }
    });
}

fn bump(mut rows: Mut<(Count,)>) {
    each_mut(&mut rows, |_entity, (mut count,), ()| {
        count.0 += 1;
    });
}

fn bump_late(mut rows: Mut<(Late,)>) {
    each_mut(&mut rows, |_entity, (mut late,), ()| {
        late.0 += 1;
    });
}

fn bump_split(mut rows: Mut<(Count,), (&Step,)>) {
    each_mut_split(&mut rows, 7, |_entity, (mut count,), (step,)| {
        count.0 += step.0;
    });
}

fn bump_parallel(mut rows: Mut<(Count,), (&Step,)>) {
    each_mut_parallel(&mut rows, |_entity, (mut count,), (step,)| {
        count.0 += step.0;
    });
}

fn bump_with_option(mut rows: Mut<(Count,), (Option<&Step>,)>) {
    each_mut(&mut rows, |_entity, (mut count,), (step,)| {
        count.0 += step.map_or(100, |step| step.0);
    });
}

fn sum(storage: &Storage) -> u64 {
    query::<&Count>(storage)
        .map(|(_entity, count)| count.0)
        .sum()
}

fn run_once(system: &mut System, storage: &mut Storage) {
    once(system, storage, &mut Resources::default());
}

fn run_batches(batches: &mut [Vec<System>], storage: &mut Storage) {
    run(
        batches,
        storage,
        &mut Resources::default(),
        &mut None,
        false,
    );
}

#[test]
fn despawn_keeps_the_other_rows() {
    let mut storage = Storage::default();
    let first = spawn_with(&mut storage, (Count(1), Step(10)));
    let second = spawn_with(&mut storage, (Count(2), Step(20)));
    let third = spawn_with(&mut storage, (Count(3), Step(30)));

    despawn(&mut storage, first);

    assert!(!is_alive(&storage, first));
    assert_eq!(get::<Count>(&storage, first), None);
    assert_eq!(get::<Count>(&storage, second), Some(&Count(2)));
    assert_eq!(get::<Step>(&storage, second), Some(&Step(20)));
    assert_eq!(get::<Count>(&storage, third), Some(&Count(3)));
    assert_eq!(get::<Step>(&storage, third), Some(&Step(30)));
    assert_eq!(living(&storage).len(), 2);

    let reused = spawn_with(&mut storage, (Count(4),));
    assert_eq!(reused.index, first.index);
    assert_ne!(reused, first);
    assert!(!is_alive(&storage, first));
    assert_eq!(get::<Count>(&storage, reused), Some(&Count(4)));
    assert!(!has::<Step>(&storage, reused));
}

#[test]
fn set_and_remove_move_the_row_and_keep_its_values() {
    let mut storage = Storage::default();
    let entities: Vec<Entity> = (0..5)
        .map(|value| spawn_with(&mut storage, (Count(value),)))
        .collect();

    for (value, entity) in entities.iter().enumerate() {
        set(&mut storage, *entity, Step(value as u64 * 10));
    }
    for (value, entity) in entities.iter().enumerate() {
        assert_eq!(get::<Count>(&storage, *entity), Some(&Count(value as u64)));
        assert_eq!(
            get::<Step>(&storage, *entity),
            Some(&Step(value as u64 * 10))
        );
    }

    set(&mut storage, entities[1], Step(99));
    assert_eq!(get::<Step>(&storage, entities[1]), Some(&Step(99)));

    assert!(removed_since::<Step>(&storage, 0).is_empty());
    remove::<Step>(&mut storage, entities[0]);
    remove::<Step>(&mut storage, entities[3]);
    remove::<Step>(&mut storage, entities[3]);

    assert!(!has::<Step>(&storage, entities[0]));
    assert!(!has::<Step>(&storage, entities[3]));
    assert_eq!(get::<Step>(&storage, entities[1]), Some(&Step(99)));
    assert_eq!(get::<Step>(&storage, entities[2]), Some(&Step(20)));
    assert_eq!(get::<Step>(&storage, entities[4]), Some(&Step(40)));
    for (value, entity) in entities.iter().enumerate() {
        assert_eq!(get::<Count>(&storage, *entity), Some(&Count(value as u64)));
    }
    let mut removed = removed_since::<Step>(&storage, 0);
    removed.sort_unstable();
    let mut expected = vec![entities[0], entities[3]];
    expected.sort_unstable();
    assert_eq!(removed, expected);
    assert_eq!(query::<(&Count, &Step)>(&storage).count(), 3);
    assert_eq!(query::<&Count>(&storage).count(), 5);
}

#[test]
fn values_that_own_memory_survive_moves_and_drops() {
    let mut storage = Storage::default();
    let entities: Vec<Entity> = (0..50)
        .map(|value| spawn_with(&mut storage, (Heavy(format!("value {value}")),)))
        .collect();
    for entity in entities.iter().step_by(2) {
        set(&mut storage, *entity, Count(1));
    }
    for entity in entities.iter().step_by(3) {
        despawn(&mut storage, *entity);
    }
    for (value, entity) in entities.iter().enumerate() {
        let held = get::<Heavy>(&storage, *entity).map(|heavy| heavy.0.clone());
        match value % 3 {
            0 => assert_eq!(held, None),
            _ => assert_eq!(held, Some(format!("value {value}"))),
        }
    }
}

#[test]
fn spawn_batch_matches_single_spawns() {
    let mut storage = Storage::default();
    let single = spawn_with(&mut storage, (Count(1000), Step(1)));
    despawn(&mut storage, single);

    spawn_batch(&mut storage, (0..100).map(|value| (Count(value), Step(2))));
    spawn_batch(&mut storage, (0..10).map(|value| (Count(value), Step(2))));

    let entities = living(&storage);
    assert_eq!(entities.len(), 110);
    assert_eq!(sum(&storage), 4950 + 45);
    assert_eq!(query::<(&Count, &Step)>(&storage).count(), 110);
    for entity in &entities {
        assert!(is_alive(&storage, *entity));
        assert_eq!(get::<Step>(&storage, *entity), Some(&Step(2)));
    }
    let mut indices: Vec<u32> = entities.iter().map(|entity| entity.index).collect();
    indices.sort_unstable();
    indices.dedup();
    assert_eq!(indices.len(), 110);

    despawn(&mut storage, entities[5]);
    assert_eq!(living(&storage).len(), 109);
    assert_eq!(query::<(&Count, &Step)>(&storage).count(), 109);
}

#[test]
fn spawn_batch_with_one_component_two_times_keeps_the_last() {
    let mut storage = Storage::default();
    spawn_batch(&mut storage, (0..4).map(|value| (Count(value), Count(7))));
    assert_eq!(living(&storage).len(), 4);
    assert_eq!(sum(&storage), 28);
}

#[test]
fn required_components_arrive_on_each_path() {
    let mut storage = Storage::default();
    let before = spawn_with(&mut storage, (Step(1),));
    set(&mut storage, before, Owner(0));
    assert!(!has::<Needed>(&storage, before));

    require::<Owner, Needed>(&mut storage);

    let spawned = spawn_with(&mut storage, (Owner(1), Count(1)));
    assert_eq!(get::<Needed>(&storage, spawned), Some(&Needed(0)));
    assert_eq!(get::<Owner>(&storage, spawned), Some(&Owner(1)));

    let kept = spawn_with(&mut storage, (Owner(2), Needed(5)));
    assert_eq!(get::<Needed>(&storage, kept), Some(&Needed(5)));

    let later = spawn_with(&mut storage, (Count(2),));
    set(&mut storage, later, Owner(3));
    assert_eq!(get::<Needed>(&storage, later), Some(&Needed(0)));
    assert_eq!(get::<Count>(&storage, later), Some(&Count(2)));

    let attached = spawn_with(&mut storage, (Count(3),));
    attach(&mut storage, attached, (Owner(4), Step(4)));
    assert_eq!(get::<Needed>(&storage, attached), Some(&Needed(0)));
    assert_eq!(get::<Step>(&storage, attached), Some(&Step(4)));
    assert_eq!(get::<Count>(&storage, attached), Some(&Count(3)));

    spawn_batch(
        &mut storage,
        (10..20).map(|value| (Owner(value), Count(value))),
    );
    let owners: Vec<(Entity, u64)> = query::<&Owner>(&storage)
        .map(|(entity, owner)| (entity, owner.0))
        .collect();
    assert_eq!(owners.len(), 15);
    for (entity, owner) in owners {
        if owner >= 10 {
            assert_eq!(get::<Needed>(&storage, entity), Some(&Needed(0)));
            assert_eq!(get::<Count>(&storage, entity), Some(&Count(owner)));
        }
    }
    assert_eq!(query::<(&Owner, &Needed)>(&storage).count(), 14);
}

#[test]
fn a_system_visits_each_row_of_each_archetype() {
    let mut storage = Storage::default();
    spawn_batch(&mut storage, (0..10).map(|_| (Count(0), Step(1))));
    spawn_batch(&mut storage, (0..10).map(|_| (Count(0), Step(2), Marker)));
    spawn_batch(&mut storage, (0..10).map(|_| (Count(0),)));
    let mut system = step.into_system();

    run_once(&mut system, &mut storage);
    assert_eq!(sum(&storage), 30);

    spawn_batch(&mut storage, (0..10).map(|_| (Count(0), Step(5), Owner(0))));
    run_once(&mut system, &mut storage);
    assert_eq!(sum(&storage), 30 + 30 + 50);
}

#[test]
fn a_system_made_before_its_component_exists_finds_it_later() {
    let mut storage = Storage::default();
    let mut system = bump_late.into_system();
    run_once(&mut system, &mut storage);
    spawn_with(&mut storage, (Count(1),));
    run_once(&mut system, &mut storage);

    let entity = spawn_with(&mut storage, (Late(1),));
    run_once(&mut system, &mut storage);
    run_once(&mut system, &mut storage);

    assert_eq!(get::<Late>(&storage, entity).map(|late| late.0), Some(3));
}

#[test]
fn optional_reads_give_none_where_the_component_is_absent() {
    let mut storage = Storage::default();
    spawn_batch(&mut storage, (0..3).map(|_| (Count(0), Step(1))));
    spawn_batch(&mut storage, (0..2).map(|_| (Count(0),)));
    let mut system = bump_with_option.into_system();

    run_once(&mut system, &mut storage);

    assert_eq!(sum(&storage), 3 + 200);
}

#[test]
fn split_and_parallel_loops_match_the_plain_loop() {
    for count in [0u64, 1, 6, 7, 8, 100, 5000] {
        let mut storage = Storage::default();
        spawn_batch(
            &mut storage,
            (0..count).map(|value| (Count(value), Step(3))),
        );
        let entities = living(&storage);

        run_once(&mut bump_split.into_system(), &mut storage);
        run_once(&mut bump_parallel.into_system(), &mut storage);
        run_once(&mut step.into_system(), &mut storage);

        for (value, entity) in entities.iter().enumerate() {
            assert_eq!(
                get::<Count>(&storage, *entity),
                Some(&Count(value as u64 + 9))
            );
        }
    }
}

#[test]
fn the_first_change_read_gives_a_superset_and_later_reads_are_exact() {
    let mut storage = Storage::default();
    let entities: Vec<Entity> = (0..6)
        .map(|value| spawn_with(&mut storage, (Count(value),)))
        .collect();
    let odd: Vec<Entity> = entities.iter().copied().skip(1).step_by(2).collect();
    let mut systems = vec![bump_the_odd.into_system()];
    let mut batches = batch(&mut systems, &[]);

    let start = tick(&storage);
    run_batches(&mut batches, &mut storage);
    let mut first = changed_since::<Count>(&storage, start);
    first.sort_unstable();
    for entity in &odd {
        assert!(first.contains(entity));
    }

    let middle = tick(&storage);
    assert!(changed_since::<Count>(&storage, middle).is_empty());

    run_batches(&mut batches, &mut storage);
    let mut second = changed_since::<Count>(&storage, middle);
    second.sort_unstable();
    let mut expected = odd.clone();
    expected.sort_unstable();
    assert_eq!(second, expected);

    let last = tick(&storage);
    spawn_with(&mut storage, (Count(100),));
    set(&mut storage, entities[0], Count(50));
    let mut third = changed_since::<Count>(&storage, last - 1);
    third.sort_unstable();
    assert_eq!(third.len(), 2);
    assert!(third.contains(&entities[0]));
}

#[test]
fn change_ticks_follow_a_row_through_moves_and_removals() {
    let mut storage = Storage::default();
    let entities: Vec<Entity> = (0..4)
        .map(|value| spawn_with(&mut storage, (Count(value),)))
        .collect();
    assert_eq!(changed_since::<Count>(&storage, 0).len(), 4);
    let mut systems = vec![bump.into_system()];
    let mut batches = batch(&mut systems, &[]);
    run_batches(&mut batches, &mut storage);
    let after_bump = tick(&storage) - 1;

    set(&mut storage, entities[1], Step(1));
    remove::<Step>(&mut storage, entities[1]);
    despawn(&mut storage, entities[0]);
    assert!(changed_since::<Count>(&storage, after_bump).is_empty());

    if let Some(mut count) = get_mut::<Count>(&mut storage, entities[2]) {
        count.0 = 77;
    }
    assert_eq!(
        changed_since::<Count>(&storage, after_bump),
        vec![entities[2]]
    );
    assert_eq!(
        changed_at::<Count>(&storage, entities[2]),
        Some(tick(&storage))
    );
    assert_eq!(get::<Count>(&storage, entities[2]), Some(&Count(77)));
    assert_eq!(get::<Count>(&storage, entities[3]), Some(&Count(4)));

    spawn_batch(&mut storage, (0..3).map(|value| (Count(value),)));
    assert_eq!(changed_since::<Count>(&storage, after_bump).len(), 4);
}

#[test]
fn reading_through_a_tracked_value_marks_no_change() {
    let mut storage = Storage::default();
    let entity = spawn_with(&mut storage, (Count(1),));
    assert_eq!(changed_since::<Count>(&storage, 0).len(), 1);
    let mut systems = vec![bump.into_system()];
    let mut batches = batch(&mut systems, &[]);
    run_batches(&mut batches, &mut storage);
    let after = tick(&storage) - 1;

    let read = get_mut::<Count>(&mut storage, entity).map(|count| count.0);

    assert_eq!(read, Some(2));
    assert!(changed_since::<Count>(&storage, after).is_empty());
}

fn peek_the_first(mut rows: Mut<(Count,)>, targets: View<&Owner>) {
    let wanted: Vec<Entity> = each(&targets).map(|(entity, _)| entity).collect();
    for entity in wanted {
        if let Some((mut count,)) = peek_mut(&mut rows, entity) {
            count.0 += 1000;
        }
    }
}

#[test]
fn peek_mut_reaches_one_row() {
    let mut storage = Storage::default();
    let plain = spawn_with(&mut storage, (Count(1),));
    let owned = spawn_with(&mut storage, (Count(2), Owner(0)));
    let without = spawn_with(&mut storage, (Owner(1),));

    run_once(&mut peek_the_first.into_system(), &mut storage);

    assert_eq!(get::<Count>(&storage, plain), Some(&Count(1)));
    assert_eq!(get::<Count>(&storage, owned), Some(&Count(1002)));
    assert!(!has::<Count>(&storage, without));
}

fn spawn_later(mut later: Later, counts: View<&Count>) {
    let held = each(&counts).count() as u64;
    spawn(&mut later, (Count(held), Step(held)));
}

#[test]
fn a_deferred_spawn_lands_after_the_system() {
    let mut storage = Storage::default();
    require::<Step, Needed>(&mut storage);
    let old = spawn_with(&mut storage, (Count(9),));
    despawn(&mut storage, old);
    let mut system = spawn_later.into_system();

    run_once(&mut system, &mut storage);
    run_once(&mut system, &mut storage);
    run_once(&mut system, &mut storage);

    let mut counts: Vec<u64> = query::<(&Count, &Step, &Needed)>(&storage)
        .map(|(_entity, (count, _, _))| count.0)
        .collect();
    counts.sort_unstable();
    assert_eq!(counts, vec![0, 1, 2]);
    assert_eq!(living(&storage).len(), 3);
}

fn grow_counts(mut rows: Mut<(Count,)>) {
    each_mut(&mut rows, |_entity, (mut count,), ()| {
        let mut value = count.0;
        for _ in 0..200 {
            value = value.wrapping_mul(6364136223846793005).wrapping_add(1);
        }
        count.0 = value;
    });
}

fn grow_steps(mut rows: Mut<(Step,)>) {
    each_mut(&mut rows, |_entity, (mut step,), ()| {
        let mut value = step.0;
        for _ in 0..200 {
            value = value.wrapping_mul(6364136223846793005).wrapping_add(1);
        }
        step.0 = value;
    });
}

#[test]
fn a_batch_that_runs_in_parallel_matches_single_runs() {
    let mut together = Storage::default();
    let mut alone = Storage::default();
    for storage in [&mut together, &mut alone] {
        spawn_batch(
            storage,
            (0..20_000).map(|value| (Count(value), Step(value))),
        );
    }
    let mut systems = vec![grow_counts.into_system(), grow_steps.into_system()];
    let mut batches = batch(&mut systems, &[]);
    assert_eq!(batches.iter().map(Vec::len).collect::<Vec<_>>(), vec![2]);
    let mut first = grow_counts.into_system();
    let mut second = grow_steps.into_system();

    for _ in 0..10 {
        run_batches(&mut batches, &mut together);
        run_once(&mut first, &mut alone);
        run_once(&mut second, &mut alone);
    }

    let left: Vec<(u64, u64)> = query::<(&Count, &Step)>(&together)
        .map(|(_entity, (count, step))| (count.0, step.0))
        .collect();
    let right: Vec<(u64, u64)> = query::<(&Count, &Step)>(&alone)
        .map(|(_entity, (count, step))| (count.0, step.0))
        .collect();
    assert_eq!(left.len(), 20_000);
    assert_eq!(left, right);
}

#[test]
fn removals_before_the_first_read_are_not_lost() {
    let mut storage = Storage::default();
    let entities: Vec<Entity> = (0..4)
        .map(|value| spawn_with(&mut storage, (Count(value), Step(value))))
        .collect();

    remove::<Step>(&mut storage, entities[0]);
    despawn(&mut storage, entities[1]);
    remove::<Step>(&mut storage, entities[3]);
    set(&mut storage, entities[3], Step(7));

    for _ in 0..2 {
        let removed = removed_since::<Step>(&storage, 0);
        assert!(removed.contains(&entities[0]));
        assert!(removed.contains(&entities[1]));
        assert!(removed.contains(&entities[3]));
    }

    remove::<Step>(&mut storage, entities[2]);
    let removed = removed_since::<Step>(&storage, 0);
    assert!(removed.contains(&entities[0]));
    assert!(removed.contains(&entities[2]));
    assert!(removed_since::<Step>(&storage, tick(&storage)).is_empty());
}

#[test]
fn removals_after_the_first_read_are_listed_exactly() {
    let mut storage = Storage::default();
    let entities: Vec<Entity> = (0..4)
        .map(|value| spawn_with(&mut storage, (Count(value), Late(value))))
        .collect();
    spawn_with(&mut storage, (Count(9),));

    assert!(removed_since::<Late>(&storage, 0).is_empty());

    remove::<Late>(&mut storage, entities[3]);
    set(&mut storage, entities[3], Late(7));
    despawn(&mut storage, entities[1]);

    let mut expected = vec![entities[1], entities[3]];
    expected.sort_unstable();
    assert_eq!(removed_since::<Late>(&storage, 0), expected);
}
