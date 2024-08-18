use criterion::{black_box, criterion_group, criterion_main, Criterion};
use santorini_minimax::game_state::{GameState};
use rand::{Rng, SeedableRng};
use santorini_minimax::game_state::game_state_5x5_binary_128bit::GameState5x5Binary128bit;
use santorini_minimax::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use santorini_minimax::generic_game_state::GenericGameState;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);

    let random_states: Vec<GameState5x5Binary128bit> = (0..1000000).map(|_| GameState5x5Binary128bit::from_generic_game_state(&GenericSantoriniGameState::<5, 5, 2>::generate_random_state_rng(&mut rng))).collect();

    let mut group = c.benchmark_group("GameState 5x5 Benchmarks");

    // Here we specify the desired sample size
    group.sample_size(10);

    group.bench_function("generate 1,000,000 children states", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.get_children_states());
        }
    }));

    group.bench_function("generate 1,000,000 children states with vec reuse", |b| b.iter(|| {
        let mut vec = Vec::with_capacity(32);
        for state in &random_states {
            vec = black_box(state.get_children_states_reuse_vec(vec));
        }
    }));

    group.bench_function("flip 1,000,000 states", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.get_flipped_state());
        }
    }));

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);