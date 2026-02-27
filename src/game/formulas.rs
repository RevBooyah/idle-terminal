/// Calculate cost of the Nth building (0-indexed count of currently owned).
/// cost(n) = base_cost * multiplier^n
pub fn building_cost(base_cost: f64, cost_multiplier: f64, count: u32) -> f64 {
    base_cost * cost_multiplier.powi(count as i32)
}

/// Calculate production per tick for a building type.
/// production = count * base_production * (1 + level_bonus * level) * global_multiplier
pub fn building_production(
    count: u32,
    base_production: f64,
    level: u32,
    level_bonus: f64,
    global_multiplier: f64,
) -> f64 {
    count as f64 * base_production * (1.0 + level_bonus * level as f64) * global_multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_cost() {
        let base = 10.0;
        let mult = 1.15;
        assert!((building_cost(base, mult, 0) - 10.0).abs() < 0.001);
        assert!((building_cost(base, mult, 1) - 11.5).abs() < 0.001);
        assert!((building_cost(base, mult, 10) - 10.0 * 1.15_f64.powi(10)).abs() < 0.01);
    }

    #[test]
    fn test_building_production() {
        // 5 buildings, 1.0 base production, level 2, 0.5 level bonus, 1.0 global
        let prod = building_production(5, 1.0, 2, 0.5, 1.0);
        assert!((prod - 10.0).abs() < 0.001); // 5 * 1.0 * (1 + 0.5*2) * 1.0 = 10.0
    }
}
