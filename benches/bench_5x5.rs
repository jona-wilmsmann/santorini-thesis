use criterion::{black_box, criterion_group, criterion_main, Criterion};
use santorini_minimax::game_state::{GameState, MinimaxReady};
use rand::{Rng, SeedableRng};
use santorini_minimax::game_state::game_state_4x4_binary_3bit::GameState4x4Binary3Bit;
use santorini_minimax::game_state::game_state_5x5_binary_128bit::GameState5x5Binary128bit;
use santorini_minimax::game_state::game_state_5x5_binary_composite::GameState5x5BinaryComposite;
use santorini_minimax::game_state::game_state_5x5_struct::GameState5x5Struct;
use santorini_minimax::generic_game_state::GenericGameState;
use santorini_minimax::minimax::minimax;
use santorini_minimax::minimax::minimax_cache::MinimaxCache;

fn benchmark_game_state<GS: GameState>(name: &str, c: &mut Criterion) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);

    let random_states: Vec<GS> = (0..1000000).map(|_| GS::from_generic_game_state(&GenericGameState::generate_random_state_rng(&mut rng))).collect();

    let mut group = c.benchmark_group(format!("{} - General Benchmark", name));

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

fn benchmark_minimax<GS: GameState + MinimaxReady>(name: &str, c: &mut Criterion) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);

    let random_states: Vec<GS> = (0..1000000).map(|_| GS::from_generic_game_state(&GenericGameState::generate_random_state_rng(&mut rng))).collect();

    let mut group = c.benchmark_group(format!("{} - Minimax Benchmark", name));

    group.sample_size(10);

    group.bench_function("get 1,000,000 static evaluations", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.get_static_evaluation());
        }
    }));

    group.bench_function("calculate minimax to depth 5 for 100 states", |b| b.iter(|| {
        for state in random_states.iter().take(100) {
            let mut minimax_cache = MinimaxCache::new();
            black_box(minimax(state, 5, f32::MIN, f32::MAX, &mut minimax_cache));
        }
    }));

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_game_state::<GameState5x5Binary128bit>("GameState5x5Binary128bit", c);
    benchmark_game_state::<GameState5x5Struct>("GameState5x5Struct", c);
    benchmark_game_state::<GameState5x5BinaryComposite>("GameState5x5Binary", c);


    benchmark_minimax::<GameState5x5Binary128bit>("GameState5x5Binary128bit", c);
    benchmark_minimax::<GameState5x5Struct>("GameState5x5Struct", c);
    benchmark_minimax::<GameState5x5BinaryComposite>("GameState5x5Binary", c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);