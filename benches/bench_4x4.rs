use criterion::{black_box, criterion_group, criterion_main, Criterion};
use santorini_minimax::game_state::{ContinuousBlockId, ContinuousId, GameState, SimplifiedState, StaticEvaluation};
use rand::{Rng, SeedableRng};
use santorini_minimax::game_state::game_state_4x4_binary_3bit::GameState4x4Binary3Bit;
use santorini_minimax::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use santorini_minimax::generic_game_state::GenericGameState;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);

    let random_states: Vec<GameState4x4Binary3Bit> = (0..1000000).map(|_| GameState4x4Binary3Bit::from_generic_game_state(&GenericSantoriniGameState::<4, 4, 1>::generate_random_state_rng(&mut rng))).collect();
    let random_simplified_states: Vec<GameState4x4Binary3Bit> = random_states.iter().map(|state| state.get_simplified_state()).collect();
    let random_continuous_ids: Vec<u64> = (0..1000000).map(|_| rng.gen_range(0..GameState4x4Binary3Bit::get_continuous_id_count())).collect();
    let random_continuous_block_ids: Vec<(u64, u64)> = random_simplified_states.iter().map(|state| (state.get_block_count(), state.get_continuous_block_id())).collect();

    let mut group = c.benchmark_group("GameState 4x4 Benchmarks");

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

    group.bench_function("get static valuation for 1,000,000 states", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.get_static_evaluation());
        }
    }));

    group.bench_function("get symmetric simplified state for 1,000,000 states", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.get_simplified_state());
        }
    }));


    group.bench_function("get continuous id for 1,000,000 simplified states", |b| b.iter(|| {
        for state in &random_simplified_states {
            black_box(state.get_continuous_id());
        }
    }));

    group.bench_function("generate 1,000,000 states from continuous id", |b| b.iter(|| {
        for continuous_id in &random_continuous_ids {
            black_box(GameState4x4Binary3Bit::from_continuous_id(continuous_id.clone()));
        }
    }));

    group.bench_function("get continuous block id for 1,000,000 simplified states", |b| b.iter(|| {
        for state in &random_simplified_states {
            black_box(state.get_continuous_block_id());
        }
    }));

    group.bench_function("generate 1,000,000 states from continuous block id", |b| b.iter(|| {
        for (block_count, continuous_block_id) in &random_continuous_block_ids {
            black_box(GameState4x4Binary3Bit::from_continuous_block_id(*block_count as usize, *continuous_block_id));
        }
    }));

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);