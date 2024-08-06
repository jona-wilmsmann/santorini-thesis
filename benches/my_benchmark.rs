use criterion::{black_box, criterion_group, criterion_main, Criterion};
use santorini_minimax::game_state::{ContinuousBlockId, ContinuousId, GameState, SimplifiedState, StaticEvaluation};
use rand::{Rng, SeedableRng};
use santorini_minimax::game_state::binary_3bit_game_state::Binary3BitGameState;
use santorini_minimax::generic_game_state::generic_4x4_game_state::Generic4x4GameState;
use santorini_minimax::generic_game_state::GenericGameState;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);

    let random_states: Vec<Binary3BitGameState> = (0..1000000).map(|_| Binary3BitGameState::from_generic_game_state(&Generic4x4GameState::generate_random_state())).collect();
    let random_simplified_states: Vec<Binary3BitGameState> = random_states.iter().map(|state| state.get_simplified_state()).collect();
    let random_continuous_ids: Vec<u64> = (0..1000000).map(|_| rng.gen_range(0..Binary3BitGameState::get_continuous_id_count())).collect();
    let random_continuous_block_ids: Vec<(u64, u64)> = random_simplified_states.iter().map(|state| (state.get_block_count(), state.get_continuous_block_id())).collect();

    let mut group = c.benchmark_group("GameState Benchmarks");

    // Here we specify the desired sample size
    group.sample_size(10);

    group.bench_function("generate 1,000,000 next states", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.get_children_states());
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
            black_box(Binary3BitGameState::from_continuous_id(continuous_id.clone()));
        }
    }));

    group.bench_function("get continuous block id for 1,000,000 simplified states", |b| b.iter(|| {
        for state in &random_simplified_states {
            black_box(state.get_continuous_block_id());
        }
    }));

    group.bench_function("generate 1,000,000 states from continuous block id", |b| b.iter(|| {
        for (block_count, continuous_block_id) in &random_continuous_block_ids {
            black_box(Binary3BitGameState::from_continuous_block_id(*block_count as usize, *continuous_block_id));
        }
    }));

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);