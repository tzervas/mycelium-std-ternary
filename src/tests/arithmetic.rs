//! White-box tests for [`crate::arithmetic`] (extracted from its former inline `#[cfg(test)]`
//! module per the house test-layout rule, as-touched by E-W1/M-1119).

use crate::arithmetic::*;
use crate::primitives::Trit;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Enumerate every integer in the `m`-trit range and its encoded form.
fn each_in_range(m: u32, mut f: impl FnMut(i128, Vec<Trit>)) {
    let max = max_magnitude(m).expect("m is small");
    for v in -max..=max {
        f(v, int_to_trits(v, m).expect("in range"));
    }
}

// ── trits_to_int ──────────────────────────────────────────────────────────────

#[test]
fn trits_to_int_empty_is_zero() {
    assert_eq!(trits_to_int(&[]), 0);
}

#[test]
fn worked_example_neg78_in_6_trits() {
    // binary-ternary.md §5: −78 in 6 trits is ⟨0,−1,0,0,+1,0⟩.
    let t = int_to_trits(-78, 6).expect("in range");
    use Trit::{Neg, Pos, Zero};
    assert_eq!(t, vec![Zero, Neg, Zero, Zero, Pos, Zero]);
    assert_eq!(trits_to_int(&t), -78);
}

// ── int_to_trits ──────────────────────────────────────────────────────────────

#[test]
fn range_is_symmetric() {
    // (3^6 - 1)/2 = 364
    assert_eq!(max_magnitude(6), Some(364));
    assert!(int_to_trits(364, 6).is_some());
    assert!(int_to_trits(-364, 6).is_some());
    // Mutant witness: if the range check were removed, 365 would produce Some(wrong value).
    assert_eq!(
        int_to_trits(365, 6),
        None,
        "just-past-max must be None (C1)"
    );
    assert_eq!(int_to_trits(-365, 6), None, "just-past-min must be None");
}

#[test]
fn codec_round_trips_exhaustively_at_small_widths() {
    // property: trits_to_int(int_to_trits(v, m)) == v for every v in the m-trit range.
    for m in 1..=5 {
        each_in_range(m, |v, t| {
            assert_eq!(t.len(), m as usize, "width={m}");
            assert_eq!(trits_to_int(&t), v, "round-trip at m={m}");
        });
    }
}

/// **E-W1/M-1119 DoD item 2 — the headline `None`→`Some` flip.** Pre-widening, `m = 41` caused
/// `3^41` to overflow `i64` and `max_magnitude` returned `None` (this test used to be named
/// `max_magnitude_overflows_at_m41` and asserted exactly that). Post-widening (`i128`), `m = 41`
/// — the W-1 canonical `Ternary` width (`docs/spec/swaps/binary-ternary.md` §A.3) — succeeds, and
/// a full `Binary{64}`-range round-trip through it is exact.
#[test]
fn max_magnitude_succeeds_at_m41_round_trip() {
    assert_eq!(max_magnitude(41), Some(18_236_498_188_585_393_201));
    // A full Binary{64}-range value round-trips exactly at m=41 (3^40 < 2^64 <= 3^41, so 40
    // trits do not suffice but 41 do — the exact reason this pair is now the W-1 canon).
    for v in [i64::MIN, -1, 0, 1, i64::MAX] {
        let value = i128::from(v);
        let t = int_to_trits(value, 41).unwrap_or_else(|| panic!("{v} must fit in 41 trits"));
        assert_eq!(t.len(), 41);
        assert_eq!(trits_to_int(&t), value, "round-trip at m=41 for v={v}");
    }
    // 0 fits in any width regardless of the ceiling (int_to_trits does not call max_magnitude).
    assert!(int_to_trits(0, 41).is_some());
    // A large value that is out of range for m=6 is still the real C1 test, unaffected by the
    // widening (both the old and new ceiling are far above m=6).
    assert_eq!(
        int_to_trits(365, 6),
        None,
        "value past 6-trit max must be None"
    );
    assert_eq!(
        int_to_trits(-365, 6),
        None,
        "value past 6-trit min must be None"
    );
}

/// The widened ceiling itself: `i128` carries `max_magnitude` through `m = 80`; `m = 81` is the
/// new explicit boundary (`3^81` overflows `i128`) — never a silent truncation (C1).
#[test]
fn max_magnitude_new_ceiling_is_m81() {
    assert!(max_magnitude(80).is_some());
    assert_eq!(max_magnitude(81), None);
}

// ── neg ───────────────────────────────────────────────────────────────────────

#[test]
fn neg_is_value_negation_exhaustively() {
    // property: trits_to_int(neg(t)) == -trits_to_int(t) for every value in range.
    for m in 1..=5 {
        each_in_range(m, |v, t| {
            assert_eq!(trits_to_int(&neg(&t)), -v, "neg at m={m}");
        });
    }
}

#[test]
fn neg_is_involution_exhaustively() {
    // property: neg(neg(t)) == t — the balanced-ternary range is symmetric (no asymmetry).
    for m in 1..=5 {
        each_in_range(m, |_v, t| {
            assert_eq!(neg(&neg(&t)), t, "involution at m={m}");
        });
    }
}

// ── add ───────────────────────────────────────────────────────────────────────

#[test]
fn add_matches_integer_oracle_exhaustively() {
    // oracle: add(a, b) == int_to_trits(digit(a)+digit(b), m) for every pair in m-trit range.
    // Mutant witness: removing the carry check causes overflow to silently wrap.
    for m in 1..=4 {
        let max = max_magnitude(m).unwrap();
        for x in -max..=max {
            for y in -max..=max {
                let a = int_to_trits(x, m).unwrap();
                let b = int_to_trits(y, m).unwrap();
                let got = add(&a, &b);
                let expected = x + y;
                if expected.abs() <= max {
                    assert_eq!(got, int_to_trits(expected, m), "add({x},{y}) at m={m}");
                } else {
                    assert_eq!(got, None, "add({x},{y}) should overflow at m={m}");
                }
            }
        }
    }
}

#[test]
fn add_rejects_unequal_widths() {
    // C1: mismatched widths are an explicit None, not a silent partial result.
    let a = int_to_trits(1, 2).unwrap();
    let b = int_to_trits(1, 3).unwrap();
    assert_eq!(add(&a, &b), None, "unequal-width add must be None");
}

// ── sub ───────────────────────────────────────────────────────────────────────

#[test]
fn sub_matches_integer_oracle_exhaustively() {
    for m in 1..=4 {
        let max = max_magnitude(m).unwrap();
        for x in -max..=max {
            for y in -max..=max {
                let a = int_to_trits(x, m).unwrap();
                let b = int_to_trits(y, m).unwrap();
                let got = sub(&a, &b);
                let expected = x - y;
                if expected.abs() <= max {
                    assert_eq!(got, int_to_trits(expected, m), "sub({x},{y}) at m={m}");
                } else {
                    assert_eq!(got, None, "sub({x},{y}) should overflow at m={m}");
                }
            }
        }
    }
}

// ── mul ───────────────────────────────────────────────────────────────────────

#[test]
fn mul_matches_integer_oracle_exhaustively() {
    // Mutant witness: replacing the high-trit check with always-pass causes overflow to silently
    // return wrong low trits.
    for m in 1..=4 {
        let max = max_magnitude(m).unwrap();
        for x in -max..=max {
            for y in -max..=max {
                let a = int_to_trits(x, m).unwrap();
                let b = int_to_trits(y, m).unwrap();
                let got = mul(&a, &b);
                let expected = x * y;
                if expected.abs() <= max {
                    assert_eq!(got, int_to_trits(expected, m), "mul({x},{y}) at m={m}");
                } else {
                    assert_eq!(got, None, "mul({x},{y}) should overflow at m={m}");
                }
            }
        }
    }
}

#[test]
fn mul_rejects_unequal_widths() {
    let a = int_to_trits(1, 2).unwrap();
    let b = int_to_trits(1, 3).unwrap();
    assert_eq!(mul(&a, &b), None, "unequal-width mul must be None");
}

// ── algebraic identities ────────────────────────────────────────────────────────

#[test]
fn add_neg_b_equals_sub() {
    // property: add(a, neg(b)) == sub(a, b) for every pair in range.
    for m in 1..=4 {
        let max = max_magnitude(m).unwrap();
        for x in -max..=max {
            for y in -max..=max {
                let a = int_to_trits(x, m).unwrap();
                let b = int_to_trits(y, m).unwrap();
                let nb = neg(&b);
                assert_eq!(
                    add(&a, &nb),
                    sub(&a, &b),
                    "add(a,neg(b))==sub(a,b) at ({x},{y}),m={m}"
                );
            }
        }
    }
}

#[test]
fn neg_is_additive_inverse_when_sum_in_range() {
    // property: add(a, neg(a)) == zero for every a.
    for m in 1..=4 {
        let max = max_magnitude(m).unwrap();
        let zero = int_to_trits(0, m).unwrap();
        for x in -max..=max {
            let a = int_to_trits(x, m).unwrap();
            let na = neg(&a);
            assert_eq!(
                add(&a, &na),
                Some(zero.clone()),
                "a + neg(a) == 0 for x={x},m={m}"
            );
        }
    }
}
