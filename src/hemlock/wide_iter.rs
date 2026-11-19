use hemlock_ecs::storage::{Storage, each_mut, spawn_batch};

macro_rules! components {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Clone, Copy)]
            struct $name(f32);
        )*
    };
}

components!(C0, C1, C2, C3, C4, C5, C6, C7);

pub const ENTITY_COUNT: usize = 8_000_000;

type Addends = (
    &'static C1,
    &'static C2,
    &'static C3,
    &'static C4,
    &'static C5,
    &'static C6,
    &'static C7,
);

fn filled(count: usize) -> Benchmark {
    let mut storage = Storage::default();
    spawn_batch(
        &mut storage,
        (0..count).map(|_| {
            (
                C0(0.0),
                C1(1.0),
                C2(1.0),
                C3(1.0),
                C4(1.0),
                C5(1.0),
                C6(1.0),
                C7(1.0),
            )
        }),
    );

    Benchmark(storage)
}

pub struct Benchmark(Storage);

impl Benchmark {
    pub fn setup() -> Self {
        filled(ENTITY_COUNT)
    }

    pub fn run(&mut self) {
        each_mut::<(C0,), Addends>(
            &mut self.0,
            |_entity, (mut c0,), (c1, c2, c3, c4, c5, c6, c7)| {
                c0.0 += c1.0 + c2.0 + c3.0 + c4.0 + c5.0 + c6.0 + c7.0;
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemlock_ecs::storage::query;

    #[test]
    fn accumulates_every_column_into_the_first() {
        let mut bench = filled(64);

        bench.run();

        let mut count = 0;
        let mut last = 0.0;
        for (_entity, c0) in query::<&C0>(&bench.0) {
            count += 1;
            last = c0.0;
        }
        assert_eq!(count, 64);
        assert!((last - 7.0).abs() < 1e-3);
    }
}
