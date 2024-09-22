use criterion::{black_box, criterion_group, criterion_main, Criterion};
use santorini_minimax::game_state::{ContinuousBlockId, ContinuousId, GameState, MinimaxReady, SimplifiedState};
use rand::{Rng, SeedableRng};
use santorini_minimax::game_state::game_state_4x4_binary_3bit::GameState4x4Binary3Bit;
use santorini_minimax::game_state::game_state_4x4_binary_4bit::GameState4x4Binary4Bit;
use santorini_minimax::game_state::game_state_4x4_struct::GameState4x4Struct;
use santorini_minimax::game_state::game_state_5x5_5bit::GameState5x5Binary5bit;
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

    group.bench_function("get 1,000,000 player to move", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.is_player_a_turn());
        }
    }));

    group.bench_function("get 1,000,000 player a won", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.has_player_a_won());
        }
    }));

    group.bench_function("get 1,000,000 player b won", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.has_player_b_won());
        }
    }));

    group.bench_function("generate 1,000,000 children states", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.get_children_states());
        }
    }));

    group.bench_function("generate 1,000,000 children states with vec reuse", |b| b.iter(|| {
        let mut vec = Vec::with_capacity(32);
        for state in &random_states {
            black_box(state.get_children_states_reuse_vec(&mut vec));
        }
    }));

    group.finish();
}

fn benchmark_minimax<GS: GameState + MinimaxReady>(depth: usize, name: &str, c: &mut Criterion) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);

    let random_states: Vec<GS> = (0..1000000).map(|_| GS::from_generic_game_state(&GenericGameState::generate_random_state_rng(&mut rng))).collect();

    let mut group = c.benchmark_group(format!("{} - Minimax Benchmark", name));

    group.sample_size(10);

    group.bench_function("get 1,000,000 static evaluations", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.get_static_evaluation());
        }
    }));

    group.bench_function(format!("calculate minimax to depth {} for 100 states", depth), |b| b.iter(|| {
        for state in random_states.iter().take(100) {
            let mut minimax_cache = MinimaxCache::new();
            black_box(minimax(state, depth, f32::MIN, f32::MAX, &mut minimax_cache));
        }
    }));

    group.finish();
}

fn benchmark_simplified<GS: GameState + SimplifiedState>(name: &str, c: &mut Criterion) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);

    let random_states: Vec<GS> = (0..1000000).map(|_| GS::from_generic_game_state(&GenericGameState::generate_random_state_rng(&mut rng))).collect();

    let mut group = c.benchmark_group(format!("{} - Simplified State Benchmark", name));

    group.sample_size(10);

    group.bench_function("get 1,000,000 symmetric simplified states", |b| b.iter(|| {
        for state in &random_states {
            black_box(state.get_simplified_state());
        }
    }));

    group.finish();
}

fn benchmark_continuous_id<GS: GameState + ContinuousId + ContinuousBlockId>(name: &str, c: &mut Criterion) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);

    let random_states: Vec<GS> = (0..1000000).map(|_| GS::from_generic_game_state(&GenericGameState::generate_random_state_rng(&mut rng))).collect();
    let random_simplified_states: Vec<GS> = random_states.iter().map(|state| state.get_simplified_state()).collect();
    let random_continuous_ids: Vec<u64> = (0..1000000).map(|_| rng.gen_range(0..GameState4x4Binary3Bit::get_continuous_id_count())).collect();
    let random_continuous_block_ids: Vec<(u64, u64)> = random_simplified_states.iter().map(|state| (state.get_block_count(), state.get_continuous_block_id())).collect();

    let mut group = c.benchmark_group(format!("{} - Simplified State Benchmark", name));

    group.sample_size(10);

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

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_game_state::<GameState5x5Binary128bit>("Binary 128bit - 5x5", c);
    benchmark_game_state::<GameState5x5Struct>("Struct - 5x5", c);
    benchmark_game_state::<GameState5x5BinaryComposite>("Binary Composite - 5x5", c);
    benchmark_game_state::<GameState5x5Binary5bit>("Binary 5bit - 5x5", c);
    benchmark_game_state::<GameState4x4Binary3Bit>("Binary 3bit - 4x4", c);
    benchmark_game_state::<GameState4x4Binary4Bit>("Binary 4bit - 4x4", c);
    benchmark_game_state::<GameState4x4Struct>("Struct - 4x4", c);


    benchmark_minimax::<GameState5x5Binary128bit>(5, "Binary 128bit - 5x5", c);
    benchmark_minimax::<GameState5x5Struct>(5, "Struct - 5x5", c);
    benchmark_minimax::<GameState5x5BinaryComposite>(5, "Binary Composite - 5x5", c);
    benchmark_minimax::<GameState5x5Binary5bit>(5, "Binary 5bit - 5x5", c);
    benchmark_minimax::<GameState4x4Binary3Bit>(7, "Binary 3bit - 4x4", c);
    benchmark_minimax::<GameState4x4Binary4Bit>(7, "Binary 4bit - 4x4", c);
    benchmark_minimax::<GameState4x4Struct>(7, "Struct - 4x4", c);


    benchmark_simplified::<GameState4x4Binary3Bit>("Binary 3bit - 4x4", c);
    benchmark_simplified::<GameState4x4Binary4Bit>("Binary 4bit - 4x4", c);


    benchmark_continuous_id::<GameState4x4Binary3Bit>("Binary 3bit - 4x4", c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);