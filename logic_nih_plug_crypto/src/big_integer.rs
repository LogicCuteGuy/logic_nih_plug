//! Arbitrary-precision unsigned integer arithmetic.
//!
//! [`BigInteger`] is a thin wrapper around `num-bigint`'s `BigUint`. JUCE's
//! `BigInteger` is also unsigned and provides a similar surface (parsing,
//! formatting, bit access, modular exponentiation, GCD), so the names and
//! shapes here intentionally mirror JUCE's class.
//!
//! Negative values are not represented; if you need signed arithmetic, drop
//! down to `num_bigint::BigInt` directly.

use std::fmt;

use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::Zero;

use crate::error::CryptoError;

/// Arbitrary-precision unsigned integer.
///
/// Backed by `num_bigint::BigUint`. `Clone`, `PartialEq`, `Eq`, `Hash`, and
/// `Ord` are all derived so `BigInteger` can be used as a hash-map key or
/// stored in a `BTreeMap`.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BigInteger(BigUint);

impl BigInteger {
    /// Creates the value `0`.
    pub fn new() -> Self {
        Self(BigUint::default())
    }

    /// Wraps an existing `num_bigint::BigUint`.
    pub fn from_big_uint(n: BigUint) -> Self {
        Self(n)
    }

    /// Returns the inner `BigUint`. Provided for interop with the rest of the
    /// `num-bigint` ecosystem.
    pub fn as_big_uint(&self) -> &BigUint {
        &self.0
    }

    /// Consumes the wrapper and returns the inner `BigUint`.
    pub fn into_big_uint(self) -> BigUint {
        self.0
    }

    /// Creates a value from a primitive integer.
    pub fn from_u32(n: u32) -> Self {
        Self(BigUint::from(n))
    }

    /// Creates a value from a primitive integer.
    pub fn from_u64(n: u64) -> Self {
        Self(BigUint::from(n))
    }

    /// Creates a value from a primitive integer.
    pub fn from_u128(n: u128) -> Self {
        Self(BigUint::from(n))
    }

    /// Creates a value from a big-endian byte string.
    pub fn from_bytes_be(bytes: &[u8]) -> Self {
        Self(BigUint::from_bytes_be(bytes))
    }

    /// Returns the value as a big-endian byte string.
    pub fn to_bytes_be(&self) -> Vec<u8> {
        self.0.to_bytes_be()
    }

    /// Parses a number written in the given `radix`. Supports radixes 2..=36,
    /// matching `num_bigint::BigUint::parse_bytes`. Leading whitespace is not
    /// trimmed and the string must consist entirely of valid digits (and an
    /// optional leading `+` for radix 10+).
    pub fn parse(input: &str, radix: u32) -> Result<Self, CryptoError> {
        if !(2..=36).contains(&radix) {
            return Err(CryptoError::BigIntParse {
                input: input.to_owned(),
                radix,
            });
        }
        BigUint::parse_bytes(input.as_bytes(), radix)
            .map(Self)
            .ok_or_else(|| CryptoError::BigIntParse {
                input: input.to_owned(),
                radix,
            })
    }

    /// Convenience: parse as decimal (radix 10).
    pub fn parse_decimal(input: &str) -> Result<Self, CryptoError> {
        Self::parse(input, 10)
    }

    /// Convenience: parse as hexadecimal (radix 16). Accepts both upper- and
    /// lower-case digits.
    pub fn parse_hex(input: &str) -> Result<Self, CryptoError> {
        Self::parse(input, 16)
    }

    /// Renders the value as a string in the given `radix`. Supports the same
    /// radixes as [`BigInteger::parse`].
    pub fn to_string_radix(&self, radix: u32) -> String {
        self.0.to_str_radix(radix)
    }

    /// Convenience: render as decimal.
    pub fn to_string_decimal(&self) -> String {
        self.0.to_str_radix(10)
    }

    /// Convenience: render as lowercase hex.
    pub fn to_string_hex(&self) -> String {
        self.0.to_str_radix(16)
    }

    /// Returns `true` if the value is `0`.
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Returns `true` if the value is `1`.
    pub fn is_one(&self) -> bool {
        // BigUint doesn't expose is_one(); check manually.
        self.0 == BigUint::from(1u32)
    }

    /// Returns the number of bits required to represent the value — i.e. the
    /// position of the highest set bit plus one (zero for `0`).
    pub fn bit_length(&self) -> u64 {
        self.0.bits()
    }

    /// Returns the number of set bits (popcount).
    pub fn count_bits(&self) -> u64 {
        // BigUint doesn't expose popcount() directly; iterate over limbs.
        let mut count = 0u64;
        for limb in self.0.iter_u64_digits() {
            count += limb.count_ones() as u64;
        }
        count
    }

    /// Returns the value of the bit at position `n` (zero-indexed from the
    /// least-significant end).
    pub fn get_bit(&self, n: u64) -> bool {
        // BigUint::bit panics on huge indices; clamp for safety.
        if n >= self.0.bits() {
            return false;
        }
        self.0.bit(n)
    }

    /// Sets the bit at position `n` to `1`.
    pub fn set_bit(&mut self, n: u64) {
        self.0.set_bit(n, true);
    }

    /// Clears the bit at position `n` to `0`.
    pub fn clear_bit(&mut self, n: u64) {
        self.0.set_bit(n, false);
    }

    /// Returns the lowest `num_bits` bits starting at bit `start_bit`.
    /// The result is widened to fill a [`BigInteger`].
    pub fn get_bit_range(&self, start_bit: u64, num_bits: u64) -> BigInteger {
        if num_bits == 0 || self.is_zero() {
            return BigInteger::new();
        }
        let mut result = BigUint::default();
        for i in 0..num_bits {
            if self.get_bit(start_bit + i) {
                result.set_bit(i, true);
            }
        }
        BigInteger(result)
    }

    /// Sets the lowest `num_bits` bits starting at bit `start_bit` to the
    /// corresponding bits in `value`. Higher bits of `value` are ignored.
    pub fn set_bit_range(&mut self, start_bit: u64, num_bits: u64, value: &BigInteger) {
        for i in 0..num_bits {
            if value.get_bit(i) {
                self.0.set_bit(start_bit + i, true);
            } else {
                self.0.set_bit(start_bit + i, false);
            }
        }
    }

    /// Returns the position of the highest set bit, or `None` if the value is
    /// zero.
    pub fn highest_bit(&self) -> Option<u64> {
        if self.is_zero() {
            None
        } else {
            Some(self.0.bits() - 1)
        }
    }

    /// Returns the position of the lowest set bit, or `None` if the value is
    /// zero.
    pub fn lowest_bit(&self) -> Option<u64> {
        if self.is_zero() {
            return None;
        }
        for i in 0..self.0.bits() {
            if self.get_bit(i) {
                return Some(i);
            }
        }
        // Unreachable: is_zero() returned false above.
        Some(0)
    }

    /// Wrapping addition.
    pub fn plus(&self, rhs: &BigInteger) -> BigInteger {
        BigInteger(&self.0 + &rhs.0)
    }

    /// Subtraction. Returns [`CryptoError::BigIntArithmetic`] on underflow.
    pub fn minus(&self, rhs: &BigInteger) -> Result<BigInteger, CryptoError> {
        if self.0 < rhs.0 {
            return Err(CryptoError::BigIntArithmetic(
                "subtraction would underflow an unsigned big integer",
            ));
        }
        Ok(BigInteger(&self.0 - &rhs.0))
    }

    /// Multiplication.
    pub fn multiplied_by(&self, rhs: &BigInteger) -> BigInteger {
        BigInteger(&self.0 * &rhs.0)
    }

    /// Integer division. Returns [`CryptoError::BigIntArithmetic`] when
    /// `rhs` is zero.
    pub fn divided_by(&self, rhs: &BigInteger) -> Result<BigInteger, CryptoError> {
        if rhs.0.is_zero() {
            return Err(CryptoError::BigIntArithmetic("division by zero"));
        }
        Ok(BigInteger(&self.0 / &rhs.0))
    }

    /// Remainder. Returns [`CryptoError::BigIntArithmetic`] when `rhs` is
    /// zero.
    pub fn modulo(&self, rhs: &BigInteger) -> Result<BigInteger, CryptoError> {
        if rhs.0.is_zero() {
            return Err(CryptoError::BigIntArithmetic("modulo by zero"));
        }
        Ok(BigInteger(&self.0 % &rhs.0))
    }

    /// Modular exponentiation: `self^exp mod m`.
    pub fn mod_pow(
        &self,
        exp: &BigInteger,
        modulus: &BigInteger,
    ) -> Result<BigInteger, CryptoError> {
        if modulus.0.is_zero() {
            return Err(CryptoError::BigIntArithmetic("mod_pow with zero modulus"));
        }
        Ok(BigInteger(self.0.modpow(&exp.0, &modulus.0)))
    }

    /// Greatest common divisor. `gcd(0, x) = x` and `gcd(0, 0) = 0`.
    pub fn gcd(&self, rhs: &BigInteger) -> BigInteger {
        BigInteger(self.0.gcd(&rhs.0))
    }

    /// Swaps the values of `self` and `other`.
    pub fn swap_with(&mut self, other: &mut BigInteger) {
        std::mem::swap(&mut self.0, &mut other.0);
    }

    /// Returns the raw little-endian limb representation. Provided for
    /// interop with low-level algorithms.
    pub fn to_limbs(&self) -> Vec<u32> {
        self.0.iter_u32_digits().collect()
    }

    /// Builds a value from a little-endian limb representation.
    pub fn from_limbs(limbs: &[u32]) -> BigInteger {
        BigInteger(BigUint::from_slice(limbs))
    }
}

impl fmt::Display for BigInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_str_radix(10))
    }
}

impl From<u32> for BigInteger {
    fn from(n: u32) -> Self {
        Self::from_u32(n)
    }
}

impl From<u64> for BigInteger {
    fn from(n: u64) -> Self {
        Self::from_u64(n)
    }
}

impl From<u128> for BigInteger {
    fn from(n: u128) -> Self {
        Self::from_u128(n)
    }
}

impl From<BigUint> for BigInteger {
    fn from(n: BigUint) -> Self {
        Self(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_default() {
        let z = BigInteger::new();
        assert!(z.is_zero());
        assert_eq!(z.bit_length(), 0);
        assert_eq!(z.highest_bit(), None);
    }

    #[test]
    fn from_u64_round_trip() {
        let n = BigInteger::from_u64(0xdead_beef);
        assert!(!n.is_zero());
        assert_eq!(n.bit_length(), 32);
        assert_eq!(n.highest_bit(), Some(31));
        assert_eq!(n.to_string_hex(), "deadbeef");
    }

    #[test]
    fn from_bytes_be_matches_to_bytes_be() {
        let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let n = BigInteger::from_bytes_be(&bytes);
        assert_eq!(n.to_bytes_be(), bytes);
    }

    #[test]
    fn parse_decimal_round_trip() {
        let s = "123456789012345678901234567890";
        let n = BigInteger::parse_decimal(s).unwrap();
        assert_eq!(n.to_string_decimal(), s);
    }

    #[test]
    fn parse_hex_round_trip() {
        let s = "deadbeefcafebabe";
        let n = BigInteger::parse_hex(s).unwrap();
        assert_eq!(n.to_string_hex(), s);
    }

    #[test]
    fn parse_rejects_bad_radix() {
        let err = BigInteger::parse("10", 1).unwrap_err();
        matches!(err, CryptoError::BigIntParse { .. });

        let err = BigInteger::parse("10", 99).unwrap_err();
        matches!(err, CryptoError::BigIntParse { .. });
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(BigInteger::parse_decimal("not a number").is_err());
    }

    #[test]
    fn bit_manipulation() {
        let mut n = BigInteger::new();
        n.set_bit(0);
        n.set_bit(3);
        n.set_bit(100);
        assert!(n.get_bit(0));
        assert!(!n.get_bit(1));
        assert!(n.get_bit(3));
        assert!(n.get_bit(100));
        assert_eq!(n.count_bits(), 3);
        assert_eq!(n.bit_length(), 101);
        n.clear_bit(3);
        assert!(!n.get_bit(3));
        assert_eq!(n.count_bits(), 2);
    }

    #[test]
    fn bit_range() {
        let n = BigInteger::from_u64(0xabcd_ef01_2345_6789);
        // Lowest 16 bits: 0x6789 (the LSB half-word of the u64).
        assert_eq!(n.get_bit_range(0, 16).to_string_hex(), "6789");
        // Bits 16..=31: 0x2345.
        assert_eq!(n.get_bit_range(16, 16).to_string_hex(), "2345");
        // Highest 16 bits: 0xabcd.
        assert_eq!(n.get_bit_range(48, 16).to_string_hex(), "abcd");
        // Crossing a byte boundary: bits 8..=23 = (0x45 << 8) | 0x67 = 0x4567.
        assert_eq!(n.get_bit_range(8, 16).to_string_hex(), "4567");
    }

    #[test]
    fn set_bit_range() {
        // 0xa5 = 0b1010_0101: bit 0=1, 1=0, 2=1, 3=0, 4=0, 5=1, 6=0, 7=1.
        let mut n = BigInteger::new();
        n.set_bit_range(4, 8, &BigInteger::from_u32(0xa5));
        assert!(n.get_bit(4));   // bit 0 of 0xa5 -> 1
        assert!(!n.get_bit(5));  // bit 1 -> 0
        assert!(n.get_bit(6));   // bit 2 -> 1
        assert!(!n.get_bit(7));  // bit 3 -> 0
        assert!(!n.get_bit(8));  // bit 4 -> 0
        assert!(n.get_bit(9));   // bit 5 -> 1
        assert!(!n.get_bit(10)); // bit 6 -> 0
        assert!(n.get_bit(11));  // bit 7 -> 1
    }

    #[test]
    fn arithmetic_basic() {
        let a = BigInteger::parse_decimal("1000000000000000000000").unwrap();
        let b = BigInteger::parse_decimal("999999999999999999999").unwrap();
        let sum = a.plus(&b);
        assert_eq!(sum.to_string_decimal(), "1999999999999999999999");
        let diff = a.minus(&b).unwrap();
        assert_eq!(diff, BigInteger::from_u32(1));
        assert!(a.minus(&a.plus(&BigInteger::from_u32(1))).is_err());
    }

    #[test]
    fn multiply_divide_modulo() {
        let a = BigInteger::parse_decimal("123456789012345678901234567890").unwrap();
        let b = BigInteger::from_u32(42);
        let product = a.multiplied_by(&b);
        let back = product.divided_by(&b).unwrap();
        assert_eq!(back, a);
        let remainder = product.modulo(&b).unwrap();
        assert_eq!(remainder, BigInteger::new());
        assert!(a.divided_by(&BigInteger::new()).is_err());
        assert!(a.modulo(&BigInteger::new()).is_err());
    }

    #[test]
    fn mod_pow_known_value() {
        // 2^10 mod 1000 = 24
        let base = BigInteger::from_u32(2);
        let exp = BigInteger::from_u32(10);
        let modulus = BigInteger::from_u32(1000);
        assert_eq!(base.mod_pow(&exp, &modulus).unwrap(), BigInteger::from_u32(24));
    }

    #[test]
    fn gcd_known_values() {
        assert_eq!(
            BigInteger::from_u32(12).gcd(&BigInteger::from_u32(18)),
            BigInteger::from_u32(6)
        );
        assert_eq!(
            BigInteger::from_u32(17).gcd(&BigInteger::from_u32(5)),
            BigInteger::from_u32(1)
        );
        // gcd(0, x) = x
        assert_eq!(
            BigInteger::new().gcd(&BigInteger::from_u32(42)),
            BigInteger::from_u32(42)
        );
    }

    #[test]
    fn swap() {
        let mut a = BigInteger::from_u32(1);
        let mut b = BigInteger::from_u32(2);
        a.swap_with(&mut b);
        assert_eq!(a, BigInteger::from_u32(2));
        assert_eq!(b, BigInteger::from_u32(1));
    }

    #[test]
    fn ordering() {
        let a = BigInteger::from_u32(1);
        let b = BigInteger::from_u32(2);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a.cmp(&a), std::cmp::Ordering::Equal);
    }

    #[test]
    fn limbs_round_trip() {
        let n = BigInteger::parse_hex("ffffffffffffffff").unwrap();
        let limbs = n.to_limbs();
        // u64::MAX = 0xffff_ffff_ffff_ffff = two u32::MAX limbs.
        assert_eq!(limbs, vec![u32::MAX, u32::MAX]);
        assert_eq!(BigInteger::from_limbs(&limbs), n);

        // A single-limb value: 0xffffffff fits in one u32 limb.
        let small = BigInteger::parse_hex("ffffffff").unwrap();
        assert_eq!(small.to_limbs(), vec![u32::MAX]);
        assert_eq!(BigInteger::from_limbs(&small.to_limbs()), small);
    }
}
