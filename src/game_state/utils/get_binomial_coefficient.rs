// This function can overflow for large values of n and k, but it is not a problem for the current use case
pub const fn get_binomial_coefficient(n: u64, k: u64) -> u64 {
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