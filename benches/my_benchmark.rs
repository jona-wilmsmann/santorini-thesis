use criterion::{black_box, criterion_group, criterion_main, Criterion};
use santorini_minimax::game_state::GameState;
use santorini_minimax::game_state::utils::random_state_generation::generate_random_state;
use rand::Rng;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    let random_states: Vec<GameState> = (0..1000000).map(|_| GameState::from_generic_game_state(&generate_random_state())).collect();
    let random_simplified_states: Vec<GameState> = random_states.iter().map(|state| state.get_symmetric_simplified_state()).collect();
    let random_continuous_ids: Vec<u64> = (0..1000000).map(|_| rng.gen_range(0..GameState::CONTINUOUS_ID_COUNT)).collect();

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
            black_box(state.static_evaluation());
        }
    }));

    group.bench_function("get symmetric simplified state for 1,000,000 states", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.get_symmetric_simplified_state());
        }
    }));


    group.bench_function("get continuous id for 1,000,000 simplified states", |b| b.iter(|| {
        for state in &random_simplified_states {
            black_box(state.get_continuous_id());
        }
    }));

    group.bench_function("generate 1,000,000 states from continuous id", |b| b.iter(|| {
        for continuous_id in &random_continuous_ids {
            black_box(GameState::from_continuous_id(continuous_id.clone()));
        }
    }));

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);