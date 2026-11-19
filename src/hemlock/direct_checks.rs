use hemlock_ecs::entity::Entity;
use hemlock_ecs::later::{Edits, change, despawn as despawn_later, remove as remove_later, within};
use hemlock_ecs::later::{set as set_later, spawn as spawn_later};
use hemlock_ecs::order::batch;
use hemlock_ecs::resources::{Resources, get_mut as resource_mut, insert};
use hemlock_ecs::schedule::run;
use hemlock_ecs::storage::{
    Storage, changed_since, despawn, each_mut, get, has, is_alive, living, query, remove, set,
    spawn_batch, spawn_with, tick,
};
use hemlock_ecs::system::{IntoSystem, Mut, System, View, each, gated};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Count(u64);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Step(u64);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Wide([u64; 4]);

#[derive(Clone, Copy)]
struct Marker;

struct Heavy(String);

struct Open(bool);

fn sum(storage: &Storage) -> u64 {
    query::<&Count>(storage)
        .map(|(_entity, count)| count.0)
        .sum()
}

fn bump_all(storage: &mut Storage) {
    each_mut::<(Count,), ()>(storage, |_entity, (mut count,), ()| {
        count.0 += 1;
    });
}

fn run_batches(batches: &mut [Vec<System>], storage: &mut Storage, resources: &mut Resources) {
    run(batches, storage, resources, &mut None, cfg!(miri));
}

#[test]
fn a_direct_loop_visits_each_row_of_each_archetype() {
    let mut storage = Storage::default();
    for index in 0..5 {
        spawn_with(&mut storage, (Count(index),));
        spawn_with(&mut storage, (Count(index), Step(1)));
        spawn_with(&mut storage, (Count(index), Marker));
        spawn_with(&mut storage, (Step(9),));
    }
    assert_eq!(sum(&storage), 30);

    bump_all(&mut storage);
    assert_eq!(sum(&storage), 45);

    let mut seen = Vec::new();
    each_mut::<(Count,), (&Step,)>(&mut storage, |entity, (mut count,), (step,)| {
        count.0 += step.0 * 10;
        seen.push(entity);
    });
    assert_eq!(seen.len(), 5);
    assert_eq!(sum(&storage), 95);
}

#[test]
fn a_direct_loop_follows_each_change_of_structure() {
    let mut storage = Storage::default();
    let first: Vec<Entity> = (0..40)
        .map(|_| spawn_with(&mut storage, (Count(0),)))
        .collect();
    bump_all(&mut storage);
    assert_eq!(sum(&storage), 40);

    spawn_batch(&mut storage, (0..5_000).map(|_| (Count(0),)));
    bump_all(&mut storage);
    assert_eq!(sum(&storage), 40 * 2 + 5_000);

    for entity in first.iter().take(10) {
        despawn(&mut storage, *entity);
    }
    bump_all(&mut storage);
    assert_eq!(sum(&storage), 30 * 3 + 5_000 * 2);

    for entity in first.iter().skip(10).take(10) {
        set(&mut storage, *entity, Step(1));
    }
    bump_all(&mut storage);
    assert_eq!(sum(&storage), 30 * 4 + 5_000 * 3);

    for entity in first.iter().skip(10).take(10) {
        remove::<Step>(&mut storage, *entity);
    }
    for entity in first.iter().skip(20).take(10) {
        remove::<Count>(&mut storage, *entity);
    }
    bump_all(&mut storage);
    assert_eq!(sum(&storage), 20 * 5 + 5_000 * 4);
    for entity in first.iter().skip(10).take(10) {
        assert_eq!(get::<Count>(&storage, *entity), Some(&Count(5)));
    }
}

#[test]
fn a_direct_loop_reads_optional_columns() {
    let mut storage = Storage::default();
    for _ in 0..7 {
        spawn_with(&mut storage, (Count(0),));
        spawn_with(&mut storage, (Count(0), Step(3)));
    }
    each_mut::<(Count,), (Option<&Step>,)>(&mut storage, |_entity, (mut count,), (step,)| {
        count.0 += step.map_or(100, |step| step.0);
    });
    assert_eq!(sum(&storage), 7 * 100 + 7 * 3);
}

#[test]
fn a_direct_loop_marks_only_the_rows_it_writes() {
    let mut storage = Storage::default();
    let entities: Vec<Entity> = (0..20)
        .map(|index| spawn_with(&mut storage, (Count(index),)))
        .collect();
    let before = tick(&storage) - 1;
    assert_eq!(changed_since::<Count>(&storage, before).len(), 20);

    let mut batches = vec![vec![idle.into_system()]];
    let mut resources = Resources::default();
    run_batches(&mut batches, &mut storage, &mut resources);
    let since = tick(&storage) - 1;
    run_batches(&mut batches, &mut storage, &mut resources);

    each_mut::<(Count,), ()>(&mut storage, |_entity, (mut count,), ()| {
        if count.0 % 4 == 0 {
            count.0 += 100;
        }
    });
    let mut changed = changed_since::<Count>(&storage, since);
    changed.sort_unstable();
    let mut wanted: Vec<Entity> = entities.iter().copied().step_by(4).collect();
    wanted.sort_unstable();
    assert_eq!(changed, wanted);

    each_mut::<(Count,), ()>(&mut storage, |_entity, (count,), ()| {
        assert!(count.0 < 1_000);
    });
    assert_eq!(changed_since::<Count>(&storage, since).len(), 5);
}

fn idle(_counts: View<&Marker>) {}

#[test]
#[cfg_attr(miri, ignore)]
fn a_direct_loop_visits_a_large_archetype_in_blocks() {
    let rows = 300_007;
    let mut storage = Storage::default();
    spawn_batch(
        &mut storage,
        (0..rows).map(|index| (Count(index), Step(2), Wide([index; 4]))),
    );
    let mut visited = 0;
    each_mut::<(Count, Wide), (&Step,)>(&mut storage, |_entity, (mut count, mut wide), (step,)| {
        count.0 += step.0;
        wide.0[3] += 1;
        visited += 1;
    });
    assert_eq!(visited, rows);
    let total: u64 = (0..rows).sum::<u64>() + rows * 2;
    assert_eq!(sum(&storage), total);
    let widest = query::<(&Count, &Wide)>(&storage)
        .filter(|(_entity, (count, wide))| wide.0[3] + 1 == count.0 && wide.0[0] + 2 == count.0)
        .count();
    assert_eq!(widest as u64, rows);
}

#[test]
fn a_direct_loop_writes_values_that_own_memory() {
    let mut storage = Storage::default();
    for index in 0..50 {
        spawn_with(&mut storage, (Heavy(format!("row {index}")), Marker));
    }
    each_mut::<(Heavy,), (&Marker,)>(&mut storage, |_entity, (mut heavy,), (_marker,)| {
        heavy.0.push_str(" seen");
    });
    assert!(query::<&Heavy>(&storage).all(|(_entity, heavy)| heavy.0.ends_with(" seen")));
    for entity in living(&storage).into_iter().step_by(2) {
        despawn(&mut storage, entity);
    }
    each_mut::<(Heavy,), ()>(&mut storage, |_entity, (mut heavy,), ()| {
        heavy.0.push_str(" twice");
    });
    assert_eq!(query::<&Heavy>(&storage).count(), 25);
    assert!(query::<&Heavy>(&storage).all(|(_entity, heavy)| heavy.0.ends_with(" seen twice")));
}

#[test]
fn a_batch_spawn_takes_an_iterator_of_unknown_length() {
    let mut storage = Storage::default();
    spawn_batch(
        &mut storage,
        (0..9_000u64)
            .filter(|index| index % 3 != 0)
            .map(|index| (Count(index), Heavy(index.to_string()), Marker)),
    );
    assert_eq!(living(&storage).len(), 6_000);
    assert!(
        query::<(&Count, &Heavy)>(&storage)
            .all(|(_entity, (count, heavy))| heavy.0 == count.0.to_string())
    );
    spawn_batch(&mut storage, std::iter::empty::<(Count, Heavy, Marker)>());
    spawn_batch(
        &mut storage,
        std::iter::once((Count(1), Heavy(String::from("1")), Marker)),
    );
    assert_eq!(living(&storage).len(), 6_001);
}

#[test]
fn a_column_keeps_its_values_each_time_it_grows() {
    let mut storage = Storage::default();
    let entities: Vec<Entity> = (0..20_000u64)
        .map(|index| {
            spawn_with(
                &mut storage,
                (Wide([index, index + 1, index + 2, index + 3]),),
            )
        })
        .collect();
    for (index, entity) in entities.iter().enumerate() {
        let index = index as u64;
        assert_eq!(
            get::<Wide>(&storage, *entity),
            Some(&Wide([index, index + 1, index + 2, index + 3]))
        );
    }
    for entity in entities.iter().step_by(3) {
        set(&mut storage, *entity, Count(7));
    }
    for (index, entity) in entities.iter().enumerate() {
        let index = index as u64;
        assert_eq!(
            get::<Wide>(&storage, *entity).map(|wide| wide.0[0]),
            Some(index)
        );
        assert_eq!(has::<Count>(&storage, *entity), index.is_multiple_of(3));
    }
}

#[test]
fn deferred_edits_land_in_the_order_they_were_queued() {
    let mut storage = Storage::default();
    let kept = spawn_with(&mut storage, (Count(1),));
    let gone = spawn_with(&mut storage, (Count(2),));
    let made = within(&mut storage, |later| {
        set_later(later, kept, Count(5));
        remove_later::<Count>(later, kept);
        set_later(later, kept, Step(1));
        set_later(later, kept, Count(7));
        change(later, move |storage| {
            let held = get::<Count>(storage, kept).map_or(0, |count| count.0);
            set(storage, kept, Count(held * 10));
        });
        despawn_later(later, gone);
        set_later(later, gone, Count(99));
        let made = spawn_later(later, (Count(3), Step(3)));
        set_later(later, made, Count(4));
        made
    });
    assert_eq!(get::<Count>(&storage, kept), Some(&Count(70)));
    assert_eq!(get::<Step>(&storage, kept), Some(&Step(1)));
    assert!(!is_alive(&storage, gone));
    assert_eq!(get::<Count>(&storage, made), Some(&Count(4)));
    assert_eq!(get::<Step>(&storage, made), Some(&Step(3)));
    assert_eq!(living(&storage).len(), 2);
}

fn retag(counts: View<&Count>, mut edits: Edits) {
    for (entity, count) in each(&counts) {
        match count.0 % 2 {
            0 => set_later(&mut edits, entity, Step(count.0)),
            _ => remove_later::<Step>(&mut edits, entity),
        }
        set_later(&mut edits, entity, Count(count.0 + 1));
    }
}

#[test]
fn a_system_queue_is_empty_after_each_batch() {
    let mut storage = Storage::default();
    let entities: Vec<Entity> = (0..9)
        .map(|index| spawn_with(&mut storage, (Count(index),)))
        .collect();
    let mut batches = vec![vec![retag.into_system()]];
    let mut resources = Resources::default();
    for round in 1..=4 {
        run_batches(&mut batches, &mut storage, &mut resources);
        for (index, entity) in entities.iter().enumerate() {
            let before = index as u64 + round - 1;
            assert_eq!(get::<Count>(&storage, *entity), Some(&Count(before + 1)));
            assert_eq!(has::<Step>(&storage, *entity), before.is_multiple_of(2));
        }
    }
}

fn bump(mut rows: Mut<(Count,)>) {
    hemlock_ecs::system::each_mut(&mut rows, |_entity, (mut count,), ()| {
        count.0 += 1;
    });
}

fn bump_steps(mut rows: Mut<(Step,)>) {
    hemlock_ecs::system::each_mut(&mut rows, |_entity, (mut step,), ()| {
        step.0 += 1;
    });
}

#[test]
fn a_closed_gate_skips_only_its_system() {
    let mut storage = Storage::default();
    spawn_batch(&mut storage, (0..10).map(|_| (Count(0), Step(0))));
    let mut resources = Resources::default();
    insert(&mut resources, Open(false));
    let mut systems = vec![gated(bump, |open: &Open| open.0), bump_steps.into_system()];
    let mut batches = batch(&mut systems, &[]);
    let steps =
        |storage: &Storage| -> u64 { query::<&Step>(storage).map(|(_entity, step)| step.0).sum() };

    for _ in 0..3 {
        run_batches(&mut batches, &mut storage, &mut resources);
    }
    assert_eq!(sum(&storage), 0);
    assert_eq!(steps(&storage), 30);

    if let Some(open) = resource_mut::<Open>(&mut resources) {
        open.0 = true;
    }
    for _ in 0..9 {
        run_batches(&mut batches, &mut storage, &mut resources);
    }
    assert_eq!(sum(&storage), 90);
    assert_eq!(steps(&storage), 120);

    if let Some(open) = resource_mut::<Open>(&mut resources) {
        open.0 = false;
    }
    run_batches(&mut batches, &mut storage, &mut resources);
    assert_eq!(sum(&storage), 90);
    assert_eq!(steps(&storage), 130);
}

fn bump_some(mut rows: Mut<(Count, Step)>) {
    hemlock_ecs::system::each_mut(&mut rows, |_entity, (mut count, mut step), ()| {
        if count.0 % 2 == 0 {
            count.0 += 2;
        }
        if step.0 % 3 == 0 {
            step.0 += 3;
        }
    });
}

#[test]
fn a_loop_over_one_watched_and_one_plain_column_marks_each_correctly() {
    let mut storage = Storage::default();
    let entities: Vec<Entity> = (0..30)
        .map(|index| spawn_with(&mut storage, (Count(index), Step(index))))
        .collect();
    let mut batches = vec![vec![bump_some.into_system()]];
    let mut resources = Resources::default();
    assert_eq!(changed_since::<Count>(&storage, 0).len(), 30);
    run_batches(&mut batches, &mut storage, &mut resources);
    let since = tick(&storage);
    run_batches(&mut batches, &mut storage, &mut resources);

    let mut counts = changed_since::<Count>(&storage, since);
    counts.sort_unstable();
    let mut even: Vec<Entity> = entities.iter().copied().step_by(2).collect();
    even.sort_unstable();
    assert_eq!(counts, even);
    assert_eq!(changed_since::<Step>(&storage, since).len(), 30);

    run_batches(&mut batches, &mut storage, &mut resources);
    let later = tick(&storage);
    run_batches(&mut batches, &mut storage, &mut resources);
    assert_eq!(changed_since::<Step>(&storage, later).len(), 10);
    assert_eq!(changed_since::<Count>(&storage, later).len(), 15);
}

#[test]
fn a_system_follows_the_storage_that_it_runs_on() {
    let mut first = Storage::default();
    let mut second = Storage::default();
    spawn_batch(&mut first, (0..300).map(|_| (Count(0), Step(0))));
    spawn_batch(&mut second, (0..300).map(|_| (Count(1_000), Step(0))));
    let mut batches = vec![vec![bump.into_system()]];
    let mut resources = Resources::default();
    for round in 1..=3 {
        run_batches(&mut batches, &mut first, &mut resources);
        run_batches(&mut batches, &mut second, &mut resources);
        assert_eq!(sum(&first), 300 * round);
        assert_eq!(sum(&second), 300 * (1_000 + round));
    }
    drop(second);
    let mut third = Storage::default();
    spawn_batch(&mut third, (0..300).map(|_| (Count(5), Step(0))));
    run_batches(&mut batches, &mut third, &mut resources);
    run_batches(&mut batches, &mut first, &mut resources);
    assert_eq!(sum(&third), 300 * 6);
    assert_eq!(sum(&first), 300 * 4);
}

#[test]
fn two_writers_of_one_column_in_one_batch_do_not_share_it() {
    let mut storage = Storage::default();
    spawn_batch(&mut storage, (0..50).map(|_| (Count(0),)));
    let mut resources = Resources::default();
    let mut alone = vec![vec![bump.into_system()]];
    run_batches(&mut alone, &mut storage, &mut resources);
    assert_eq!(sum(&storage), 50);
    let mut batches = vec![vec![bump.into_system(), bump.into_system()]];
    let mut before = sum(&storage);
    for _ in 0..3 {
        run_batches(&mut batches, &mut storage, &mut resources);
        let gained = sum(&storage) - before;
        assert!(gained == 50 || gained == 100);
        before = sum(&storage);
    }
    let lowest = query::<&Count>(&storage)
        .map(|(_entity, count)| count.0)
        .min();
    let highest = query::<&Count>(&storage)
        .map(|(_entity, count)| count.0)
        .max();
    assert_eq!(lowest, highest);
    spawn_batch(&mut storage, (0..50).map(|_| (Count(0),)));
    run_batches(&mut batches, &mut storage, &mut resources);
    let gained = sum(&storage) - before;
    assert!(gained == 100 || gained == 200);
}
