//! `std.ternary` — Ring-1 / Tier-A capability surface (M-517; RFC-0016 §4.2/§4.3).
//!
//! The ergonomic, documented home for Mycelium's ternary differentiator (FR-M2; M-111):
//! - **First-class `Trit`/`Bit` primitives** with their identities (FR-M2).
//! - **Exact balanced-ternary integer arithmetic** — `add`, `neg`, `mul`, and the `int ↔ trits`
//!   codec — fixed-width with explicit out-of-range (M-111; `docs/spec/swaps/binary-ternary.md` §1).
//! - **Packed-ternary helpers** — I2S/TL1/TL2 codecs (RFC-0004 §5) with inspectable
//!   `Meta.physical` records, never a hidden lowering pass (C3/NFR-1/DN-01).
//! - **Guarantee matrix** — the load-bearing deliverable: every exported op's tag/fallibility/
//!   effects/explainability encoded as data and asserted in tests (RFC-0016 §4.5).
//!
//! ## Contract (C1–C6 from RFC-0016 §4.1)
//!
//! - **C1 (never-silent):** every fallible op returns `Option`/`Result`; off-range/off-domain is
//!   an explicit error — never a sentinel, silent clamp, or re-round.
//! - **C2 (honest per-op tag):** all ops tag `Exact` — the balanced-ternary algebra and the
//!   I2S/TL1/TL2 codecs are exact (VR-5). The range boundary is fallibility, not a weakened tag.
//! - **C3 (no black boxes / EXPLAIN):** pack/unpack expose the scheme via [`packing::scheme_of`]
//!   and [`packing::explain`]; packing is never a hidden lowering pass (RFC-0004 §5; DN-01).
//! - **C4 (content-addressed, value-semantic):** `Trit`/`Bit`/`Trits`/`Packed` are immutable;
//!   ops are pure functions of their inputs (no effects). Packing is not identity (DN-01; ADR-003).
//! - **C5 (above the kernel):** this crate wraps [`mycelium_core::ternary`] and adds no new
//!   trusted code (KC-3). `#![forbid(unsafe_code)]` is enforced.
//! - **C6 (declared, bounded effects):** every op is pure (effects = `none`; RFC-0014).
//!
//! ## Open questions (FLAGs — do not silently resolve)
//!
//! - **Q1:** `Bit`/`Trit` spelling pending the DN-02/06 lexicon decision (ternary.md §7-Q1).
//! - **Q2:** A future lossy packing scheme is out of scope; it would require a tag below `Exact`
//!   and cannot be silently folded in (ternary.md §7-Q2).
//! - **Q3:** The split between "caller names scheme" (v0) and "RFC-0005 selector chooses + emits
//!   EXPLAIN" (M-519) needs a cross-module design pass (ternary.md §7-Q3).
//! - **Q4:** Width ceiling mirrors the M-111 `i64` ceiling (`m ≤ 40`); bignum is out of scope
//!   for v0 (ternary.md §7-Q4). Out-of-range is explicit `None` (C1).
//!
//! ## Ambient Representation (RFC-0012 §8-Q3)
//!
//! This crate's public API participates in the RFC-0012 ambient-representation contract:
//! the representation choice (binary/ternary/dense/VSA) is implicit at the call site but
//! always reified, queryable, and EXPLAIN-able — never a black box (C3/SC-3).
//! [Declared per RFC-0012; direction accepted in DN-07 §8-Q3; per-ring pass scheduled as M-540.]
//!
//! **For this crate (Ring 1, Tier A):** Ternary ops are representation-aware by construction
//! (RFC-0001 §3.3); no implicit binary fallback exists. A `Trit` value is always in the
//! `Ternary` paradigm; packing is always named (`scheme_of` / `explain`) and never a hidden
//! lowering pass — the packed form's `Meta.physical` is an inspectable record, not an opaque
//! layout (C3/NFR-1/DN-01).
//!
//! # Stability (DN-66 freeze, 2026-07-01)
//!
//! This crate's public API, as documented in `docs/spec/stdlib/ternary.md` (spec status:
//! Accepted (2026-06-20)) and asserted by its guarantee-matrix table, is the **frozen baseline** per
//! [DN-66](../../../docs/notes/DN-66-Stdlib-Stable-API-Freeze-And-Rust-Crate-Retirement-Status.md).
//! A future breaking change here needs a spec amendment + changelog entry, not a silent edit (G2).
//!
//! **Unfrozen for the gap-closure window (ADR-045, Accepted 2026-07-10):** this DN-66 freeze is
//! lifted per [ADR-045](../../../docs/adr/ADR-045-Kernel-And-Lexicon-Unfreeze-For-Early-Gap-Closure.md)
//! for a bounded window; a breaking change here still needs a spec amendment + changelog entry
//! (G2), not a silent edit — the window closes per the ADR-045 §2.4 re-freeze conditions, bound by
//! the DN-99 residual worklist.
//! It remains the RFC-0031 D6 differential-oracle reference. A `.myc` port now exists
//! (`lib/std/ternary.myc`, M-933 — kickoff `opp`, RFC-0031 D5), with this crate as its live
//! Rust oracle (`crates/mycelium-l1/tests/std_ternary.rs`); per D6 the crate is **retained**, not
//! retired (retirement is the post-1.0 M-867 decision), and no item here is `#[deprecated]`.

#![forbid(unsafe_code)]

pub mod arithmetic;
pub mod guarantee_matrix;
pub mod packing;
pub mod primitives;

// Re-export the primary surface so callers can use `mycelium_std_ternary::Trit` etc.
pub use arithmetic::{add, int_to_trits, max_magnitude, mul, neg, sub, trits_to_int};
pub use packing::{explain, pack, scheme_of, unpack, ExplainRecord, PackError, Packed, Scheme};
pub use primitives::{Bit, Trit};
