// This function can overflow for large values of n and k, but it is not a problem for the current use case
pub const fn calculate_binomial_coefficient(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    if k == 0 || k == n {
        return 1;
    }
    let mut result = 1;
    let mut i = 0;
    while i < k {
        result *= n - i;
        result /= i + 1;
        i += 1;
    }
    return result;
}

const PRECOMPUTED_BINOMIAL_COEFFICIENTS: [[u64; 17]; 17] = precompute_binomial_coefficients();

const fn precompute_binomial_coefficients() -> [[u64; 17]; 17] {
    let mut result = [[0; 17]; 17];
    let mut n = 0;
    while n <= 16 {
        let mut k = 0;
        while k <= n {
            result[n as usize][k as usize] = calculate_binomial_coefficient(n, k);
            k += 1;
        }
        n += 1;
    }
    return result;
}

// This only works for the precomputed values <= 16, which is all we need for the current use case
pub const fn get_binomial_coefficient(n: u64, k: u64) -> u64 {
    return PRECOMPUTED_BINOMIAL_COEFFICIENTS[n as usize][k as usize];
}