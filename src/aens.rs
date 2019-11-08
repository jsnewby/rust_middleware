const MULTIPLIER_14: u128 = 100000000000000;
const MULTIPLIER_DAY: u16 = 480;

pub fn name_claim_size_fee(length: u32) -> u128 {
    match length {
        30 => 5 * MULTIPLIER_14,
        29 => 8 * MULTIPLIER_14,
        28 => 13 * MULTIPLIER_14,
        27 => 21 * MULTIPLIER_14,
        26 => 34 * MULTIPLIER_14,
        25 => 55 * MULTIPLIER_14,
        24 => 89 * MULTIPLIER_14,
        23 => 144 * MULTIPLIER_14,
        22 => 233 * MULTIPLIER_14,
        21 => 377 * MULTIPLIER_14,
        20 => 610 * MULTIPLIER_14,
        19 => 987 * MULTIPLIER_14,
        18 => 1597 * MULTIPLIER_14,
        17 => 2584 * MULTIPLIER_14,
        16 => 4181 * MULTIPLIER_14,
        15 => 6765 * MULTIPLIER_14,
        14 => 10946 * MULTIPLIER_14,
        13 => 17711 * MULTIPLIER_14,
        12 => 28657 * MULTIPLIER_14,
        11 => 46368 * MULTIPLIER_14,
        10 => 75025 * MULTIPLIER_14,
        9 => 121393 * MULTIPLIER_14,
        8 => 196418 * MULTIPLIER_14,
        7 => 317811 * MULTIPLIER_14,
        6 => 514229 * MULTIPLIER_14,
        5 => 832040 * MULTIPLIER_14,
        4 => 1346269 * MULTIPLIER_14,
        3 => 2178309 * MULTIPLIER_14,
        2 => 3524578 * MULTIPLIER_14,
        1 => 5702887 * MULTIPLIER_14,
        _ => 3 * MULTIPLIER_14,
    }
}

#[test]
fn test_name_claim_size_fee() {}

pub fn name_claim_timeout(length: u32) -> u16 {
    match length {
        0 => 0, // should not happen
        1..=4 => MULTIPLIER_DAY,
        5..=31 => 31 * MULTIPLIER_DAY,
        _ => 0,
    }
}
