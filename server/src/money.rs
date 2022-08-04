use iso_currency::Currency;

pub struct Money {
    pub currency: Currency,

    /// The whole units of the amount.
    /// For example if `currencyCode` is `"USD"`, then 1 unit is one US dollar.
    pub units: i64,

    /// Number of nano (10^-9) units of the amount.
    /// The value must be between -999,999,999 and +999,999,999 inclusive.
    /// If `units` is positive, `nanos` must be positive or zero.
    /// If `units` is zero, `nanos` can be positive, zero, or negative.
    /// If `units` is negative, `nanos` must be negative or zero.
    /// For example $-1.75 is represented as `units`=-1 and `nanos`=-750,000,000.
    pub nanos: i32,
}

impl Money {
    pub fn units_as_subunits(&self) -> i64 {
        let subunit_fraction = self.currency.subunit_fraction().unwrap_or(1u16);
        self.units * (subunit_fraction as i64)
    }

    pub fn nanos_as_subunits(&self) -> f32 {
        let subunit_fraction = self.currency.subunit_fraction().unwrap_or(1u16);
        ((self.nanos as f32) / (10_i32.pow(9) as f32)) * (subunit_fraction as f32)
    }

    pub fn subunits_truncated(&self) -> i64 {
        self.units_as_subunits() + (self.nanos_as_subunits() as i64)
    }

    pub fn subunits_rounded(&self) -> i64 {
        self.units_as_subunits() + (self.nanos_as_subunits().round() as i64)
    }

    pub fn stripe_currency(&self) -> Result<stripe::Currency, stripe::ParseCurrencyError> {
        self.currency.code().to_string().to_lowercase().parse()
    }
}

#[cfg(test)]
mod tests {
    use iso_currency::Currency;

    use super::*;

    #[test]
    pub fn subunits_truncated() {
        // $1.75 and 6/10 of cent
        let money = Money {
            currency: Currency::USD,
            units: 1,
            nanos: 756_000_000,
        };
        assert_eq!(money.subunits_truncated(), 175);
    }

    #[test]
    pub fn subunits_rounded() {
        // $1.75 and 6/10 of cent
        let money = Money {
            currency: Currency::USD,
            units: 1,
            nanos: 756_000_000,
        };
        assert_eq!(money.subunits_rounded(), 176);
    }
}
