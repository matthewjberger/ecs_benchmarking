use freecs::dynamic::DynWorld;

macro_rules! components {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Clone, Copy, Default)]
            struct $name(f32);
        )*
    };
}

components!(C0, C1, C2, C3, C4, C5, C6, C7);

freecs::impl_component!(C0, C1, C2, C3, C4, C5, C6, C7);

pub const ENTITY_COUNT: usize = 8_000_000;

pub struct Benchmark(DynWorld);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = DynWorld::new();
        world.spawn_bundles(
            (
                C0(0.0),
                C1(1.0),
                C2(1.0),
                C3(1.0),
                C4(1.0),
                C5(1.0),
                C6(1.0),
                C7(1.0),
            ),
            ENTITY_COUNT,
        );

        Self(world)
    }

    pub fn run(&mut self) {
        self.0
            .query::<(&mut C0, &C1, &C2, &C3, &C4, &C5, &C6, &C7)>()
            .for_each(|_entity, (c0, c1, c2, c3, c4, c5, c6, c7)| {
                c0.0 += c1.0 + c2.0 + c3.0 + c4.0 + c5.0 + c6.0 + c7.0;
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_every_column_into_the_first() {
        let mut world = DynWorld::new();
        world.spawn_bundles(
            (
                C0(0.0),
                C1(1.0),
                C2(1.0),
                C3(1.0),
                C4(1.0),
                C5(1.0),
                C6(1.0),
                C7(1.0),
            ),
            64,
        );
        let mut bench = Benchmark(world);

        bench.run();

        let mut count = 0;
        let mut last = 0.0;
        bench.0.query::<&C0>().for_each(|_entity, c0| {
            count += 1;
            last = c0.0;
        });
        assert_eq!(count, 64);
        assert!((last - 7.0).abs() < 1e-3);
    }
}
