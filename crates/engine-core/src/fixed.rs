//! Fixed-point helpers. 1.0 == 1000 milli.

pub type Fp = i64;
pub const FP_ONE: Fp = 1000;

#[inline]
pub fn fp_from_i64(v: i64) -> Fp {
    v.saturating_mul(FP_ONE)
}

#[inline]
pub fn fp_mul(a: Fp, b: Fp) -> Fp {
    // (a * b) / FP_ONE with i128 intermediate
    ((a as i128 * b as i128) / FP_ONE as i128) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fp_mul_identity_and_integers() {
        assert_eq!(fp_mul(FP_ONE, FP_ONE), FP_ONE);
        assert_eq!(fp_mul(fp_from_i64(2), fp_from_i64(3)), fp_from_i64(6));
        // 1.5 * 2.0 = 3.0 in milli-units
        assert_eq!(fp_mul(1500, 2000), 3000);
    }

    #[test]
    fn fp_from_i64_scales() {
        assert_eq!(fp_from_i64(0), 0);
        assert_eq!(fp_from_i64(1), FP_ONE);
        assert_eq!(fp_from_i64(-2), -2000);
    }
}
