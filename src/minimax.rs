pub mod minimax_cache;

use std::sync::Arc;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use tokio::sync::RwLock;
use crate::game_state::{GameState, MinimaxReady};
use crate::minimax::minimax_cache::{Bounds, MinimaxCache};

#[inline(always)]
fn order_children_states<GS: GameState + MinimaxReady>(children_states: &mut Vec<GS>, maximizing: bool) {
    // Create a vector of tuples with the static evaluation and the GameState
    let mut children_evaluations: Vec<(f32, &mut GS)> = children_states.into_iter().map(|state| (state.get_child_evaluation(), state)).collect();
    // Sort the vector by the static evaluation
    if maximizing {
        children_evaluations.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    } else {
        children_evaluations.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }
    // Replace the children_states vector with the sorted vector
    *children_states = children_evaluations.iter().map(|(_, state)| **state).collect();
}


fn simple_minimax_internal<GS: GameState>(
    game_state: &GS,
    maximizing_player: bool,
    depth: usize,
    reused_children_vec: &mut Vec<GS>,
    evaluated_states: &mut usize,
) -> f32 {
    *evaluated_states += 1;

    if game_state.has_player_a_won() {
        return f32::INFINITY;
    } else if game_state.has_player_b_won() {
        return f32::NEG_INFINITY;
    }

    if depth == 0 {
        return 0.0;
    }

    game_state.get_children_states_reuse_vec(reused_children_vec);

    let mut reusable_vec_for_children = Vec::with_capacity(32);
    if maximizing_player {
        let mut max_evaluation = f32::NEG_INFINITY;
        for child in reused_children_vec {
            let evaluation = simple_minimax_internal(child, false, depth - 1, &mut reusable_vec_for_children, evaluated_states);
            if evaluation > max_evaluation {
                max_evaluation = evaluation;
            }
        }
        return max_evaluation;
    } else {
        let mut min_evaluation = f32::INFINITY;
        for child in reused_children_vec {
            let evaluation = simple_minimax_internal(child, true, depth - 1, &mut reusable_vec_for_children, evaluated_states);
            if evaluation < min_evaluation {
                min_evaluation = evaluation;
            }
        }
        return min_evaluation;
    }
}

pub fn simple_minimax<GS: GameState>(game_state: &GS, depth: usize) -> (f32, usize) {
    let mut evaluated_states = 0;
    let mut reused_children_vec = Vec::with_capacity(32);
    let result = simple_minimax_internal(game_state, game_state.is_player_a_turn(), depth, &mut reused_children_vec, &mut evaluated_states);
    return (result, evaluated_states);
}


fn alpha_beta_minimax_internal<GS: GameState>(
    game_state: &GS,
    maximizing_player: bool,
    depth: usize,
    mut alpha: f32,
    mut beta: f32,
    reused_children_vec: &mut Vec<GS>,
    evaluated_states: &mut usize,
) -> f32 {
    *evaluated_states += 1;

    if game_state.has_player_a_won() {
        return f32::INFINITY;
    } else if game_state.has_player_b_won() {
        return f32::NEG_INFINITY;
    }

    if depth == 0 {
        return 0.0;
    }

    game_state.get_children_states_reuse_vec(reused_children_vec);

    let mut reusable_vec_for_children = Vec::with_capacity(32);
    if maximizing_player {
        let mut max_evaluation = f32::NEG_INFINITY;
        for child in reused_children_vec {
            let evaluation = alpha_beta_minimax_internal(child, false, depth - 1, alpha, beta, &mut reusable_vec_for_children, evaluated_states);
            if evaluation > max_evaluation {
                max_evaluation = evaluation;
                if max_evaluation >= beta {
                    break;
                }
                if max_evaluation > alpha {
                    alpha = max_evaluation;
                }
            }
        }
        return max_evaluation;
    } else {
        let mut min_evaluation = f32::INFINITY;
        for child in reused_children_vec {
            let evaluation = alpha_beta_minimax_internal(child, true, depth - 1, alpha, beta, &mut reusable_vec_for_children, evaluated_states);
            if evaluation < min_evaluation {
                min_evaluation = evaluation;
                if min_evaluation <= alpha {
                    break;
                }
                if min_evaluation < beta {
                    beta = min_evaluation;
                }
            }
        }
        return min_evaluation;
    }
}

pub fn alpha_beta_minimax<GS: GameState>(game_state: &GS, depth: usize) -> (f32, usize) {
    let mut evaluated_states = 0;
    let mut reused_children_vec = Vec::with_capacity(32);
    let result = alpha_beta_minimax_internal(game_state, game_state.is_player_a_turn(), depth, f32::NEG_INFINITY, f32::INFINITY, &mut reused_children_vec, &mut evaluated_states);
    return (result, evaluated_states);
}


fn alpha_beta_sorted_minimax_internal<GS: GameState + MinimaxReady, const MIN_DEPTH_TO_SORT: usize>(
    game_state: &GS,
    maximizing_player: bool,
    depth: usize,
    mut alpha: f32,
    mut beta: f32,
    reused_children_vec: &mut Vec<GS>,
    evaluated_states: &mut usize,
) -> f32 {
    *evaluated_states += 1;

    if game_state.has_player_a_won() {
        return f32::INFINITY;
    } else if game_state.has_player_b_won() {
        return f32::NEG_INFINITY;
    }

    if depth == 0 {
        return 0.0;
    }

    game_state.get_children_states_reuse_vec(reused_children_vec);
    if !reused_children_vec.is_empty() {
        if depth >= MIN_DEPTH_TO_SORT {
            order_children_states(reused_children_vec, maximizing_player);
        }
    }

    let mut reusable_vec_for_children = Vec::with_capacity(32);
    if maximizing_player {
        let mut max_evaluation = f32::NEG_INFINITY;
        for child in reused_children_vec {
            let evaluation = alpha_beta_sorted_minimax_internal::<GS, MIN_DEPTH_TO_SORT>(child, false, depth - 1, alpha, beta, &mut reusable_vec_for_children, evaluated_states);
            if evaluation > max_evaluation {
                max_evaluation = evaluation;
                if max_evaluation >= beta {
                    break;
                }
                if max_evaluation > alpha {
                    alpha = max_evaluation;
                }
            }
        }
        return max_evaluation;
    } else {
        let mut min_evaluation = f32::INFINITY;
        for child in reused_children_vec {
            let evaluation = alpha_beta_sorted_minimax_internal::<GS, MIN_DEPTH_TO_SORT>(child, true, depth - 1, alpha, beta, &mut reusable_vec_for_children, evaluated_states);
            if evaluation < min_evaluation {
                min_evaluation = evaluation;
                if min_evaluation <= alpha {
                    break;
                }
                if min_evaluation < beta {
                    beta = min_evaluation;
                }
            }
        }
        return min_evaluation;
    }
}

pub fn alpha_beta_sorted_minimax<GS: GameState + MinimaxReady, const MIN_DEPTH_TO_SORT: usize>(game_state: &GS, depth: usize) -> (f32, usize) {
    let mut evaluated_states = 0;
    let mut reused_children_vec = Vec::with_capacity(32);
    let result = alpha_beta_sorted_minimax_internal::<GS, MIN_DEPTH_TO_SORT>(
        game_state,
        game_state.is_player_a_turn(),
        depth,
        f32::NEG_INFINITY,
        f32::INFINITY,
        &mut reused_children_vec,
        &mut evaluated_states,
    );
    return (result, evaluated_states);
}

fn internal_cached_minimax<GS: GameState + MinimaxReady, const MIN_DEPTH_TO_SORT: usize, const MIN_DEPTH_TO_CACHE: usize>(
    game_state: &GS,
    maximizing_player: bool,
    depth: usize,
    mut alpha: f32,
    mut beta: f32,
    cache: &mut MinimaxCache<GS, 100>,
    reused_children_vec: &mut Vec<GS>,
    evaluated_states: &mut usize,
) -> f32 {
    *evaluated_states += 1;

    if game_state.has_player_a_won() {
        return f32::INFINITY;
    } else if game_state.has_player_b_won() {
        return f32::NEG_INFINITY;
    }

    if depth == 0 {
        return 0.0;
    }

    let mut reusable_vec_for_children = Vec::with_capacity(32);

    if maximizing_player {
        let original_alpha = alpha;
        let mut max_evaluation = f32::NEG_INFINITY;

        if depth >= MIN_DEPTH_TO_CACHE {
            if let Some(cached_value) = cache.get_valuation_bounds(depth, game_state) {
                if cached_value.alpha <= alpha && cached_value.beta >= beta {
                    return cached_value.value;
                }

                if cached_value.beta <= beta && cached_value.value > alpha {
                    if cached_value.value >= beta {
                        return cached_value.value;
                    }
                    alpha = cached_value.value;
                    max_evaluation = alpha;
                }
            }
        }

        game_state.get_children_states_reuse_vec(reused_children_vec);
        if !reused_children_vec.is_empty() {
            if depth >= MIN_DEPTH_TO_SORT {
                order_children_states(reused_children_vec, maximizing_player);
            }
        }

        for child in reused_children_vec {
            let evaluation = internal_cached_minimax::<GS, MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_CACHE>(child, false, depth - 1, alpha, beta, cache, &mut reusable_vec_for_children, evaluated_states);
            if evaluation > max_evaluation {
                max_evaluation = evaluation;
                if max_evaluation >= beta {
                    break;
                }
                if max_evaluation > alpha {
                    alpha = max_evaluation;
                }
            }
        }

        if depth >= MIN_DEPTH_TO_CACHE {
            cache.insert_valuation_bounds(depth, *game_state, Bounds { value: max_evaluation, alpha: original_alpha, beta });
        }
        return max_evaluation;

    } else {
        let original_beta = beta;
        let mut min_evaluation = f32::INFINITY;

        if depth >= MIN_DEPTH_TO_CACHE {
            if let Some(cached_value) = cache.get_valuation_bounds(depth, game_state) {
                if cached_value.alpha <= alpha && cached_value.beta >= beta {
                    return cached_value.value;
                }

                if cached_value.alpha >= alpha && cached_value.value < beta {
                    if cached_value.value <= alpha {
                        return cached_value.value;
                    }
                    beta = cached_value.value;
                    min_evaluation = beta;
                }
            }
        }

        game_state.get_children_states_reuse_vec(reused_children_vec);
        if !reused_children_vec.is_empty() {
            if depth >= MIN_DEPTH_TO_SORT {
                order_children_states(reused_children_vec, maximizing_player);
            }
        }

        for child in reused_children_vec {
            let evaluation = internal_cached_minimax::<GS, MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_CACHE>(child, true, depth - 1, alpha, beta, cache, &mut reusable_vec_for_children, evaluated_states);
            if evaluation < min_evaluation {
                min_evaluation = evaluation;
                if min_evaluation <= alpha {
                    break;
                }
                if min_evaluation < beta {
                    beta = min_evaluation;
                }
            }
        }

        if depth >= MIN_DEPTH_TO_CACHE {
            cache.insert_valuation_bounds(depth, *game_state, Bounds { value: min_evaluation, alpha, beta: original_beta });
        }
        return min_evaluation;
    }
}


pub fn cached_minimax<GS: GameState + MinimaxReady, const MIN_DEPTH_TO_SORT: usize, const MIN_DEPTH_TO_CACHE: usize>(game_state: &GS, depth: usize) -> (f32, usize) {
    let mut evaluated_states = 0;
    let mut cache = MinimaxCache::new();

    let result = internal_cached_minimax::<GS, MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_CACHE>(
        game_state,
        game_state.is_player_a_turn(),
        depth,
        f32::NEG_INFINITY,
        f32::INFINITY,
        &mut cache,
        &mut Vec::with_capacity(32),
        &mut evaluated_states,
    );

    return (result, evaluated_states);
}

#[async_recursion::async_recursion]
async fn internal_parallel_minimax<GS: GameState + MinimaxReady + 'static, const MIN_DEPTH_TO_SORT: usize, const MIN_DEPTH_TO_PARALLELIZE: usize>(
    game_state: GS,
    maximizing_player: bool,
    depth: usize,
    mut alpha: f32,
    mut beta: f32,
) -> f32 {
    if depth < MIN_DEPTH_TO_PARALLELIZE {
        return alpha_beta_sorted_minimax_internal::<GS, MIN_DEPTH_TO_SORT>(&game_state, maximizing_player, depth, alpha, beta, &mut Vec::with_capacity(32), &mut 0);
    }

    if game_state.has_player_a_won() {
        return f32::INFINITY;
    } else if game_state.has_player_b_won() {
        return f32::NEG_INFINITY;
    }

    if depth == 0 {
        return 0.0;
    }

    let mut children_states = game_state.get_children_states();
    if children_states.is_empty() {
        return if maximizing_player {
            f32::NEG_INFINITY
        } else {
            f32::INFINITY
        };
    } else if depth >= MIN_DEPTH_TO_SORT {
        order_children_states(&mut children_states, maximizing_player);
    }

    if maximizing_player {
        let mut max_evaluation = f32::NEG_INFINITY;

        let first_child = children_states.first().expect("It was just checked that the vector is not empty");
        let first_evaluation = internal_parallel_minimax::<GS, MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_PARALLELIZE>(*first_child, false, depth - 1, alpha, beta).await;
        if first_evaluation > max_evaluation {
            max_evaluation = first_evaluation;
            if max_evaluation >= beta {
                return max_evaluation;
            }
            if max_evaluation > alpha {
                alpha = max_evaluation;
            }
        }

        let mut tasks = FuturesUnordered::new();

        for child in children_states.into_iter().skip(1) {
            tasks.push(tokio::spawn(async move {
                return alpha_beta_sorted_minimax_internal::<GS, MIN_DEPTH_TO_SORT>(&child, false, depth - 1, alpha, beta, &mut Vec::with_capacity(32), &mut 0);
            }));
        }

        while let Some(task) = tasks.next().await {
            let evaluation = task.expect("Task failed");
            if evaluation > max_evaluation {
                max_evaluation = evaluation;
                if max_evaluation >= beta {
                    break;
                }
                if max_evaluation > alpha {
                    alpha = max_evaluation;
                }
            }
        }

        return max_evaluation;

    } else {
        let mut min_evaluation = f32::INFINITY;

        let first_child = children_states.first().expect("It was just checked that the vector is not empty");
        let first_evaluation = internal_parallel_minimax::<GS, MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_PARALLELIZE>(*first_child, true, depth - 1, alpha, beta).await;
        if first_evaluation < min_evaluation {
            min_evaluation = first_evaluation;
            if min_evaluation <= alpha {
                return min_evaluation;
            }
            if min_evaluation < beta {
                beta = min_evaluation;
            }
        }

        let mut tasks = FuturesUnordered::new();

        for child in children_states.into_iter().skip(1) {
            tasks.push(tokio::spawn(async move {
                return alpha_beta_sorted_minimax_internal::<GS, MIN_DEPTH_TO_SORT>(&child, true, depth - 1, alpha, beta, &mut Vec::with_capacity(32), &mut 0);
            }));
        }

        while let Some(task) = tasks.next().await {
            let evaluation = task.expect("Task failed");
            if evaluation < min_evaluation {
                min_evaluation = evaluation;
                if min_evaluation <= alpha {
                    break;
                }
                if min_evaluation < beta {
                    beta = min_evaluation;
                }
            }
        }

        return min_evaluation;
    }
}


pub async fn parallel_minimax<
    GS: GameState + MinimaxReady + 'static,
    const MIN_DEPTH_TO_SORT: usize,
    const MIN_DEPTH_TO_PARALLELIZE: usize,
>(game_state: GS, depth: usize) -> f32 {
    let result = internal_parallel_minimax::<GS, MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_PARALLELIZE>(
        game_state,
        game_state.is_player_a_turn(),
        depth,
        f32::NEG_INFINITY,
        f32::INFINITY,
    );

    return result.await;
}


pub fn minimax<GS: GameState + MinimaxReady>(game_state: &GS, depth: usize, mut alpha: f32, mut beta: f32, cache: &mut MinimaxCache<GS, 100>) -> f32 {
    if depth == 0 {
        return game_state.get_static_evaluation();
    }

    if game_state.has_player_a_won() {
        return f32::MAX;
    } else if game_state.has_player_b_won() {
        return f32::MIN;
    }


    let mut children_states = game_state.get_children_states();
    // TODO: This speeds things up for some states, but makes things slower for others. Think of ways to detect when to use it
    //children_states = children_states.iter().map(|child| child.get_simplified_state()).collect();
    if depth > 2 {
        order_children_states(&mut children_states, game_state.is_player_a_turn());
    }

    if children_states.len() == 0 {
        return if game_state.is_player_a_turn() {
            f32::MIN
        } else {
            f32::MAX
        };
    }

    /*
    // TODO: Double check if this all makes sense and if there are any other possible cases to cover
    if let Some(cached_value) = cache.get_valuation_bounds(depth, game_state) {
        if cached_value.alpha <= alpha && cached_value.beta >= beta {
            return cached_value.value;
        }
        if cached_value.value >= beta {
            return cached_value.value;
        }

        if cached_value.beta <= beta {
            if cached_value.value > alpha {
                alpha = cached_value.value;
                max_evaluation = alpha;
            }
        }
    }

     */


    let mut evaluated_children = 0;
    if game_state.is_player_a_turn() {
        // Player A is maximizing
        let mut max_evaluation = f32::NEG_INFINITY;

        if let Some(cached_value) = cache.get_valuation_bounds(depth, game_state) {
            if cached_value.alpha <= alpha && cached_value.beta >= beta {
                return cached_value.value;
            }
            if cached_value.value >= beta {
                return cached_value.value;
            }

            if cached_value.beta <= beta {
                if cached_value.value > alpha {
                    alpha = cached_value.value;
                    max_evaluation = alpha;
                }
            }
        }

        for child in &children_states {
            evaluated_children += 1;
            let evaluation = minimax(child, depth - 1, alpha, beta, cache);
            if evaluation > max_evaluation {
                max_evaluation = evaluation;
            }
            if evaluation > alpha {
                alpha = evaluation;
            }
            if beta <= alpha {
                break;
            }
        }

        cache.insert_valuation_bounds(depth, game_state.clone(), Bounds { value: max_evaluation, alpha, beta });

        return max_evaluation;
    } else {
        // Player B is minimizing
        let mut min_evaluation = f32::INFINITY;

        if let Some(cached_value) = cache.get_valuation_bounds(depth, game_state) {
            if cached_value.alpha <= alpha && cached_value.beta >= beta {
                return cached_value.value;
            }
            if cached_value.value <= alpha {
                return cached_value.value;
            }

            if cached_value.alpha >= alpha {
                if cached_value.value < beta {
                    beta = cached_value.value;
                    min_evaluation = beta;
                }
            }
        }

        for child in &children_states {
            evaluated_children += 1;
            let evaluation = minimax(child, depth - 1, alpha, beta, cache);
            if evaluation < min_evaluation {
                min_evaluation = evaluation;
            }
            if evaluation < beta {
                beta = evaluation;
            }
            if beta <= alpha {
                break;
            }
        }

        cache.insert_valuation_bounds(depth, game_state.clone(), Bounds { value: min_evaluation, alpha, beta });

        return min_evaluation;
    }
}