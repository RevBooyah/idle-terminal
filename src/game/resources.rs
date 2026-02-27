use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Resources {
    pub compute: f64,
    pub bandwidth: f64,
    pub storage: f64,
    pub reputation: f64,
    pub crypto: f64,
}

impl Resources {
    pub fn can_afford(&self, cost: &Resources) -> bool {
        self.compute >= cost.compute
            && self.bandwidth >= cost.bandwidth
            && self.storage >= cost.storage
            && self.reputation >= cost.reputation
            && self.crypto >= cost.crypto
    }

    pub fn subtract(&mut self, cost: &Resources) {
        self.compute -= cost.compute;
        self.bandwidth -= cost.bandwidth;
        self.storage -= cost.storage;
        self.reputation -= cost.reputation;
        self.crypto -= cost.crypto;
    }

    pub fn add(&mut self, other: &Resources) {
        self.compute += other.compute;
        self.bandwidth += other.bandwidth;
        self.storage += other.storage;
        self.reputation += other.reputation;
        self.crypto += other.crypto;
    }
}

/// Format a number with SI suffixes: 1.23K, 4.56M, etc.
pub fn format_si(value: f64) -> String {
    if value < 0.0 {
        return format!("-{}", format_si(-value));
    }
    let suffixes = ["", "K", "M", "B", "T", "Qa", "Qi"];
    let mut val = value;
    for suffix in &suffixes {
        if val < 1000.0 {
            if val < 10.0 {
                return format!("{:.2}{}", val, suffix);
            } else if val < 100.0 {
                return format!("{:.1}{}", val, suffix);
            } else {
                return format!("{:.0}{}", val, suffix);
            }
        }
        val /= 1000.0;
    }
    format!("{:.2e}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_si() {
        assert_eq!(format_si(0.0), "0.00");
        assert_eq!(format_si(1.5), "1.50");
        assert_eq!(format_si(42.3), "42.3");
        assert_eq!(format_si(999.0), "999");
        assert_eq!(format_si(1000.0), "1.00K");
        assert_eq!(format_si(1234.0), "1.23K");
        assert_eq!(format_si(1_000_000.0), "1.00M");
        assert_eq!(format_si(2_500_000_000.0), "2.50B");
    }

    #[test]
    fn test_can_afford() {
        let res = Resources {
            compute: 100.0,
            bandwidth: 50.0,
            ..Default::default()
        };
        let cost = Resources {
            compute: 80.0,
            ..Default::default()
        };
        assert!(res.can_afford(&cost));

        let expensive = Resources {
            compute: 200.0,
            ..Default::default()
        };
        assert!(!res.can_afford(&expensive));
    }
}
