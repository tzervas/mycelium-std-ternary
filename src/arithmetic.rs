//! Balanced-ternary integer arithmetic — `add`, `neg`, `mul`, and the `int ↔ trits` codec
//! (M-111; `docs/spec/swaps/binary-ternary.md` §1).
//!
//! This is the Ring-1 wrapper around `mycelium_core::ternary`, surfacing the kernel codec and
//! arithmetic with the full contract: explicit `Option` on every fallible op, no unsafe code, and
//! no new trusted base (KC-3/C5).
//!
//! **E-W1/M-1119 widening (2026-07-18).** The kernel's conversion-utility ceiling was originally
//! documented here as `i64`-capped at `m ≤ 40` trits — that figure was itself inaccurate (the real
//! pre-widening ceiling was `m ≤ 39`; see `mycelium_core::ternary`'s module doc comment for the
//! full correction). Both this wrapper and the kernel now route `max_magnitude`/`int_to_trits`/
//! `trits_to_int` through `i128` (this crate's call sites agree with `mycelium_core`'s, per the
//! issue's DoD): widths up to `m ≈ 80` are exact; `m ≥ 81` is an explicit `None`. Full
//! arbitrary-width arithmetic (no ceiling at all) remains `mycelium_core::ternary::BigTernary`
//! (M-756) — out of scope for this Ring-1 wrapper (FLAG: Q4, narrowed but not closed).
//!
//! **Guarantee: `Exact` on every op.** The balanced-ternary algebra is an exact integer identity
//! (Knuth 4.1; `docs/spec/swaps/binary-ternary.md` §1); fallibility is the overflow/range
//! boundary, never a weakening of the tag (C2/VR-5).

use mycelium_core::ternary as kernel;

use crate::primitives::Trit;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert a slice of `Trit` (std surface) to a `Vec<mycelium_core::Trit>` (kernel type).
fn to_core(ts: &[Trit]) -> Vec<mycelium_core::Trit> {
    ts.iter().map(|&t| t.to_core()).collect()
}

/// Convert a `Vec<mycelium_core::Trit>` (kernel type) to a `Vec<Trit>` (std surface).
fn from_core(ts: Vec<mycelium_core::Trit>) -> Vec<Trit> {
    ts.into_iter().map(Trit::from_core).collect()
}

// ── Codec ─────────────────────────────────────────────────────────────────────

/// The integer denoted by an MSB-first trit string.
///
/// `value(t) = Σⱼ digit(tⱼ)·3^(m-1-j)` (Horner; `docs/spec/swaps/binary-ternary.md` §1).
/// The empty string denotes 0.
///
/// **Guarantee: `Exact`.** Total — the Horner sum is an integer identity with no approximation
/// (C2). Width ceiling: `i128` is exact for every width up to `m ≈ 80` (E-W1/M-1119).
///
/// **FLAG (Q4):** widths beyond the `i128` ceiling are not handled (full bignum out of scope for
/// this wrapper — use `mycelium_core::ternary::BigTernary`; C1 is preserved because the value
/// still fits in `i128` if the trits are well-formed at the widths this fn is safe for; the caller
/// controls width).
#[must_use]
pub fn trits_to_int(ts: &[Trit]) -> i128 {
    kernel::trits_to_int(&to_core(ts))
}

/// The unique `m`-trit balanced representation of `value`, MSB-first.
///
/// **Guarantee: `Exact`.** Returns `None` if `value ∉ [−(3^m−1)/2, +(3^m−1)/2]` — an explicit
/// out-of-range error, never a silent truncation or wrap (C1/G2). Also returns `None` if `3^m`
/// would overflow `i128` (`m ≥ 81`; FLAG Q4, widened by E-W1/M-1119 from the prior `m ≥ 41`).
/// (Mutant witness: if the range check were removed, a value of magnitude 365 in 6 trits would
/// produce a wrong trit string instead of `None`.)
#[must_use]
pub fn int_to_trits(value: i128, m: u32) -> Option<Vec<Trit>> {
    kernel::int_to_trits(value, m).map(from_core)
}

/// The maximum representable magnitude in `m` trits: `(3^m − 1) / 2`.
///
/// The symmetric range is `[−max, +max]`. Returns `None` if `3^m` would overflow `i128` (`m ≥ 81`;
/// FLAG Q4 — bignum ceiling, widened by E-W1/M-1119 from the prior `m ≥ 41`).
///
/// **Guarantee: `Exact`.** Total for valid `m`; explicit `None` for overflow (C1).
#[must_use]
pub fn max_magnitude(m: u32) -> Option<i128> {
    kernel::max_magnitude(m)
}

// ── Arithmetic ────────────────────────────────────────────────────────────────

/// Digit-wise negation of an `m`-trit balanced-ternary number.
///
/// `value(neg a) = −value(a)` exactly (balanced ternary is sign-symmetric — no two's-complement
/// asymmetry; `docs/spec/swaps/binary-ternary.md` §1). Width-preserving.
///
/// **Guarantee: `Exact`.** Total — the range `[−max, +max]` is symmetric, so the negation of
/// every representable value is also representable (C2).
#[must_use]
pub fn neg(a: &[Trit]) -> Vec<Trit> {
    from_core(kernel::neg(&to_core(a)))
}

/// Fixed-width balanced-ternary addition `a + b`.
///
/// **Guarantee: `Exact`.** Returns `None` on fixed-width overflow — i.e. when the true sum
/// `trits_to_int(a) + trits_to_int(b)` lies outside `[−max_magnitude(m), +max_magnitude(m)]` —
/// and `None` if `a.len() != b.len()`. Never silently wraps (C1/G2). (Mutant witness: if the
/// carry check were removed, an overflowing sum would silently produce a wrong result.)
#[must_use]
pub fn add(a: &[Trit], b: &[Trit]) -> Option<Vec<Trit>> {
    kernel::add(&to_core(a), &to_core(b)).map(from_core)
}

/// Fixed-width balanced-ternary subtraction `a − b = add(a, neg(b))`.
///
/// **Guarantee: `Exact`.** Returns `None` on fixed-width overflow or unequal widths (C1/G2).
#[must_use]
pub fn sub(a: &[Trit], b: &[Trit]) -> Option<Vec<Trit>> {
    kernel::sub(&to_core(a), &to_core(b)).map(from_core)
}

/// Fixed-width balanced-ternary multiplication `a × b`.
///
/// Computes the full `2m`-trit product and returns the low `m` trits iff the high trits are all
/// zero — otherwise `None` (overflow, explicit). Also `None` if `a.len() != b.len()`.
///
/// **Guarantee: `Exact`.** Returns `None` on overflow — the mathematical product
/// `trits_to_int(a) * trits_to_int(b)` exceeds the `m`-trit range — never silently truncates
/// (C1/G2). (Mutant witness: if the high-trit check were replaced with `true`, overflow would
/// silently return a wrong low `m` trits.)
#[must_use]
pub fn mul(a: &[Trit], b: &[Trit]) -> Option<Vec<Trit>> {
    kernel::mul(&to_core(a), &to_core(b)).map(from_core)
}
