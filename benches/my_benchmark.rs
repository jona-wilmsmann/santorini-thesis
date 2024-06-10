use criterion::{black_box, criterion_group, criterion_main, Criterion};
use santorini_minimax::game_state::GameState;
use santorini_minimax::game_state::utils::generate_random_state::generate_random_state;

fn criterion_benchmark(c: &mut Criterion) {
    let random_states: [GameState; 1000000] = [GameState::from_generic_game_state(&generate_random_state()); 1000000];

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


    group.bench_function("get symmetric transposition for 1,000,000 states", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.symmetric_transpose());
        }
    }));

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);