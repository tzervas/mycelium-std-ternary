# Spec — `std.ternary` (balanced-ternary arithmetic, first-class `Bit`/`Trit`, packed-ternary helpers)

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-06-20, maintainer-ratified per DN-07 — guarantee matrix asserted in tests; open §7/§8 questions are design/scope calls, not contract violations; was *Implemented (Rust-first) — pending ratification* 2026-06-18, Draft/needs-design 2026-06-17) — the Rust-first code landed as `mycelium-std-ternary` (M-517, Batch P5-A; guarantee matrix asserted in tests). The Mycelium-lang migration (M-502-gated) remains. |
| **Module / Ring** | `std.ternary` · Ring 1 (RFC-0016 §4.2) · Tier A |
| **Tracks** | M-517 (#159) — the Phase-5 task this spec delivers (RFC-0016 §4.3, the `ternary` row). |
| **Scope** | The ternary-native differentiator surface: exact balanced-ternary integer arithmetic (`add`/`neg`/`mul` and the `int ↔ trits` codec), first-class `Bit` and `Trit{−1,0,+1}` with their identities (FR-M2; M-111), and the packed-ternary helpers — the I2_S/TL1/TL2 codecs (RFC-0004 §5 / DN-01) — exposed as **inspectable** representation choices, never a hidden lowering. |
| **Boundary** | Out of scope: a *representation change* between paradigms (binary↔ternary) is `std.swap` (M-516) over the certified swap (RFC-0002 / `docs/spec/swaps/binary-ternary.md`), **not** a `std.ternary` op; the packing *selector / cost model* is the RFC-0005 mechanism surfaced by `std.select` (M-519), which this module *consumes* but does not own; lowering itself is RFC-0004's, not this library's. |
| **Depends on** | FR-M2 (first-class binary + balanced-ternary {−1,0,+1} with balanced-arithmetic identities); RFC-0004 §5 (schedule-staged packing; the I2_S/TL1/TL2 codecs); DN-01 (Resolved — lossless packing is not a type distinction); RFC-0016 §4.1 (the contract); RFC-0001 (the value model — `Trit` payload, `Meta.physical`, the guarantee lattice). |
| **Grounds on** | M-111 (the balanced-ternary kernel `int ↔ trits` + digit-wise arithmetic, reused by the interpreter's `trit.*` primitives); `docs/spec/swaps/binary-ternary.md` §1 (the algebra identities); the `physical-layout` schema (`{ "layout": "TritPacked", "scheme": "TL2" }`) — KC-3: above the kernel, consuming it. |

---

## 1. Summary

`std.ternary` is the ergonomic, documented home for Mycelium's ternary differentiator: first-class
`Bit` and balanced `Trit{−1,0,+1}` values, **exact** balanced-ternary integer arithmetic, and the
packed-ternary codecs (I2_S/TL1/TL2). The balanced-ternary algebra is exact by construction (Knuth
4.1; `docs/spec/swaps/binary-ternary.md` §1), so its ops tag `Exact` honestly (VR-5/C2). The
module's **honesty crux**: a packing is a *visible, inspectable representation choice* recorded in
`Meta.physical` (RFC-0004 §5) — **never a hidden lowering pass** — and every pack/unpack with a
domain or width condition is a total function on its domain and an **explicit error** off it (no
silent wrap, clamp, or re-round; C1/G2). The module is Ring 1 and adds **no trusted code** (KC-3):
it consumes the M-111 kernel codec/arithmetic and the RFC-0004 packing schedules as a
certificate/EXPLAIN consumer.

## 2. Scope & module boundary

- **In scope:**
  - **`Trit` / `Bit` primitives** — the `{−1,0,+1}` and `{0,1}` digit types with their constructors
    and the FR-M2 identities (negation = digit-wise sign flip; the symmetric range).
  - **Balanced-ternary integer arithmetic** — `add`, `neg`, `mul`, and the `int ↔ trits` codec
    (`trits_to_int` / `int_to_trits`), fixed-width with explicit out-of-range (M-111).
  - **Packed-ternary helpers** — `pack`/`unpack` over the I2_S / TL1 / TL2 codecs (RFC-0004 §5),
    each exposing its chosen scheme as inspectable `Meta.physical`, with an EXPLAIN-able selection
    record when a selector chose it.
- **Out of scope (and who owns it):**
  - **Binary↔ternary paradigm change** → `std.swap` (M-516), over the RFC-0002 certified bijection
    (`docs/spec/swaps/binary-ternary.md`). A swap is never one of *these* ops.
  - **The packing *selector* / cost model** → the RFC-0005 selection mechanism surfaced by
    `std.select` / `explain` (M-519); `std.ternary` *consumes* a selection + its EXPLAIN record but
    does not re-implement the policy.
  - **Lowering / scheduling itself** → RFC-0004 §5 (the lowering stage that *binds* a packing). This
    library exposes the packing as a value-level helper + the inspectable record; it does not own the
    schedule pass.
- **Ring & layering:** Ring 1 (a capability surface). It **wraps** the M-111 balanced-ternary kernel
  (re-exporting its codec + arithmetic with the contract surface) and the RFC-0004 packing codecs;
  it **builds new** only the ergonomic `Bit`/`Trit` value surface and the inspectable-packing
  helpers. No new trusted base (KC-3); no `wild`/FFI (the codecs are pure integer/bit arithmetic).

## 3. Exported-op surface (design sketch)

Value-semantic, immutable-by-default. Fallible ops return `Option`/`Result` with an explicit error
set; no op here carries an effect (all are pure functions of their inputs). This is a **design
sketch** to fix the surface and feed §4, not a committed grammar.

```
// illustrative signatures (not a committed surface)

// --- primitives (FR-M2; M-111) ---
type Trit = { Neg | Zero | Pos }      // {−1, 0, +1}
type Bit  = { Zero | One }            // {0, 1}
fn trit(d: Int)   -> Option<Trit>     // None unless d ∈ {−1,0,+1}  (C1)
fn bit(d: Int)    -> Option<Bit>      // None unless d ∈ {0,1}       (C1)
fn digit(t: Trit) -> Int              // total: Neg↦−1, Zero↦0, Pos↦+1
fn neg(t: Trit)   -> Trit             // total: sign flip (identity)
fn and/or/xor(a: Bit, b: Bit) -> Bit  // total Boolean algebra

// --- balanced-ternary integers, fixed width m (M-111) ---
type Trits = [Trit]                                   // MSB-first
fn trits_to_int(ts: Trits)        -> Int              // total: Horner Σ dⱼ·3^(m−1−j)
fn int_to_trits(v: Int, m: Width) -> Option<Trits>    // None if v ∉ [−(3^m−1)/2, +(3^m−1)/2]  (C1)
fn add(a: Trits, b: Trits, m: Width) -> Option<Trits> // None on fixed-width overflow            (C1)
fn neg(a: Trits)                     -> Trits          // total: digit-wise flip (no 2's-comp asymmetry)
fn mul(a: Trits, b: Trits, m: Width) -> Option<Trits> // None on fixed-width overflow            (C1)

// --- packed-ternary codecs (RFC-0004 §5; the visible representation choice) ---
type Packed = { bytes: Bytes, physical: Meta.physical } // physical = { layout: TritPacked, scheme }
type Scheme = { I2_S | TL1 | TL2 }                       // RFC-0004 §5
fn pack(ts: Trits, scheme: Scheme)   -> Result<Packed, PackErr>   // PackErr = OffGrid | Misaligned (C1)
fn unpack(p: Packed)                 -> Trits                      // total: codecs are lossless (Exact)
fn scheme_of(p: Packed)              -> Scheme                     // inspectable (NFR-1/C3)
fn explain(p: Packed)                -> ExplainRecord              // why this scheme (when selected)
```

## 4. Guarantee matrix (the load-bearing deliverable — RFC-0016 §4.5)

Rows = exported ops. Encoded as a checked table (the RFC-0003 §4 template), asserted in tests once
code lands — never prose only. `total` = no failure on a well-formed input of the stated type.

| Op | Guarantee tag | Fallibility (explicit error set) | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `trit` / `bit` (constructors) | `Exact` | `None` if the integer is off-domain (`∉{−1,0,+1}` / `∉{0,1}`) | none | n/a |
| `digit` / `neg(Trit)` | `Exact` | total | none | n/a |
| `and` / `or` / `xor` (`Bit`) | `Exact` | total | none | n/a |
| `trits_to_int` | `Exact` | total | none | n/a |
| `int_to_trits` | `Exact` | `None` if `v ∉ [−(3^m−1)/2, +(3^m−1)/2]` (out-of-range, explicit) | none | n/a |
| `add` (balanced ternary) | `Exact` | `None` on fixed-width overflow (never silent wrap) | none | n/a |
| `neg` (balanced ternary) | `Exact` | total (symmetric range — no overflow) | none | n/a |
| `mul` (balanced ternary) | `Exact` | `None` on fixed-width overflow | none | n/a |
| `pack` (I2_S / TL1 / TL2) | `Exact` | `Err(OffGrid)` (a non-trit / out-of-alphabet input) or `Err(Misaligned)` (width not a multiple of the scheme's group / SIMD alignment) | none | yes (the `Meta.physical` scheme + selection record) |
| `unpack` | `Exact` | total — the codecs are **lossless** (RFC-0004 §5) | none | yes (scheme is inspectable) |
| `scheme_of` / `explain` | `Exact` | total | none | yes (this *is* the inspection surface) |

**Tag justification (VR-5 — downgrade rather than overclaim).**

- **Every row is `Exact`, and only because it truly is.** The balanced-ternary algebra is exact:
  `value(t) = Σ dⱼ·3^(m−1−j)` is an integer identity, negation is a digit-wise sign flip with **no
  two's-complement asymmetry**, and the representation is unique within the symmetric range (Knuth
  4.1; `docs/spec/swaps/binary-ternary.md` §1; M-111). There is no rounding, no ε, nothing to
  approximate — so `Proven`/`Empirical`/`Declared` would be a *dishonest downgrade* of an exact fact,
  not an honest one. The arithmetic ops earn `Exact` over their **in-range** domain; the range
  boundary is handled by **fallibility** (the `None`/`Err` column), not by weakening the tag.
- **The packing codecs are `Exact` because they are lossless re-encodings.** I2_S/TL1/TL2 carry the
  *same trits, same value, different bytes* (DN-01 §2 "lossless physical layout"; RFC-0004 §5 — "pack
  and unpack keeps int16 sums for lossless inference"). `pack` then `unpack` is the identity on a
  well-formed input. The `~0.01 PPL / 0.1% accuracy` figure RFC-0004 §5 cites is the *model-level*
  effect of the BitNet pipeline, **not** a codec error — the trit-level codec is bit-exact, so `Exact`
  is honest here and a model-accuracy claim is explicitly **not** made by this module (it would be a
  different op with an `Empirical` tag). **FLAG (Q2):** if a future scheme is added that is *not*
  bit-exact (a lossy quantizer), it must tag below `Exact` and is out of this module's exact-only
  contract — recorded as an open question, not silently admitted.
- **`EXPLAIN-able` for the pack ops is non-optional (C3).** A packed value is *not* an opaque buffer:
  its scheme is a queryable `Meta.physical` record (`{ layout: TritPacked, scheme: … }`), and when a
  selector chose the scheme, the choice is an inspectable RFC-0005 EXPLAIN record. The pure algebra
  rows are `n/a` for EXPLAIN — they select/convert/approximate nothing, so there is nothing to
  explain (C3 applies only to ops that do).

## 5. §4.1 contract conformance (C1–C6)

- **C1 — never-silent (G2):** Every range/domain boundary is an explicit `Option`/`Result` that
  propagates: `int_to_trits` and `add`/`mul` return `None` on out-of-range / fixed-width overflow
  (M-111 — "never a silent wrap"); constructors return `None` off-alphabet; `pack` returns
  `Err(OffGrid | Misaligned)` on an off-grid trit or a width that does not align to the scheme's
  group / SIMD width (RFC-0004 §5 "align to SIMD width"). No sentinel, no clamp, no re-round.
- **C2 — honest per-op tag (VR-5):** All ops tag `Exact`, justified in §4 by the exactness of the
  balanced-ternary algebra and the losslessness of the codecs; the boundary is carried by
  fallibility, not by a weakened tag. A future lossy scheme would be FLAGGED and tagged below `Exact`
  (§7-Q2), never silently absorbed.
- **C3 — no black boxes / EXPLAIN (SC-3/G11):** The packing is **reified and inspectable**:
  `scheme_of`/`explain` expose the chosen scheme and (when applicable) the RFC-0005 selection record.
  The packing is *never* a hidden lowering pass from the user's perspective — it is a visible
  `Meta.physical` value-level choice (RFC-0004 §5 — "recorded as inspectable `Meta.physical`"; NFR-1).
- **C4 — content-addressed, value-semantic (ADR-003):** `Trit`/`Bit`/`Trits`/`Packed` are immutable
  values; ops are pure functions of their inputs (no effects). Crucially (DN-01, Resolved), **the
  packing is *not* part of the value's type or identity** — a re-pack does not fork content-addressed
  identity; `Meta.physical` is the *inspectable record* of the chosen schedule, and **metadata is not
  identity** (ADR-003). Two packings of the same trits are the same value.
- **C5 — above the kernel (KC-3):** The module consumes the M-111 balanced-ternary kernel and the
  RFC-0004 packing codecs; it enlarges no trusted base and uses no `wild`/FFI (pure integer/bit
  arithmetic). It is a certificate/EXPLAIN consumer (Ring 1).
- **C6 — declared, bounded effects (RFC-0014):** Every exported op is pure (effects column = `none`
  throughout §4). No IO, time, randomness, or hidden allocation budget; nothing to declare beyond the
  bounded allocation of the output buffer.

## 6. Grounding

- The first-class `Bit`/balanced-`Trit{−1,0,+1}` surface and its identities trace to **FR-M2**
  ("first-class binary and logical-ternary {−1,0,+1} values with balanced-arithmetic identities:
  negation = digit flip; rounding ≡ truncation") and **M-111** (the kernel home for `int ↔ trits` +
  digit-wise arithmetic, reused by the interpreter's `trit.*` primitives).
- The algebra's exactness (Horner value, symmetric range `[−(3^m−1)/2, +(3^m−1)/2]`, negation =
  digit-wise flip, unique representation) is grounded in **`docs/spec/swaps/binary-ternary.md` §1**
  (Knuth 4.1) and the M-111 kernel, where fixed-width overflow is already an explicit `None`.
- The packing codecs (**I2_S** default / **TL1** / **TL2**, lossless, SIMD-aligned) and the
  inspectable-`Meta.physical` / no-silent-layout discipline are **RFC-0004 §5** (schedule-staged
  packing, normative; confirmed by T1.4), resting on **DN-01** (Resolved — lossless packing is not a
  type distinction; recorded as inspectable metadata, validated against the reference semantics). The
  inspectable record's shape is the `physical-layout` schema
  (`{ "layout": "TritPacked", "scheme": "TL2" }`).
- The per-op contract (C1–C6) and the guarantee-matrix obligation are **RFC-0016 §4.1 / §4.5**; the
  module's Ring-1 / Tier-A placement and its grounding row are **RFC-0016 §4.2 / §4.3**.
- House rules: never-silent **G2**, dual projection / EXPLAIN **G11/SC-3**, honest tags **VR-5**,
  small kernel **KC-3**, inspectability **NFR-1/NFR-4**.

## 7. Open questions (FLAGGED — resolve before ratification)

- **(Q1) Surface naming — `Bit`/`Trit` vs the fungal lexicon.** RFC-0016 §8-Q2 leaves module/type
  naming a DN-level decision; whether the digit types keep the plain `Bit`/`Trit` spellings or take
  themed names is not settled here. *Disposition:* default to the M-111 kernel spellings (`Trit`,
  `digit`, `neg`) pending the DN; FLAGGED, not silently chosen.
- **(Q2) A future lossy / non-bit-exact packing scheme.** This module's contract is **exact-only**:
  I2_S/TL1/TL2 are all bit-exact codecs. If a later quantizing scheme is added, it must tag below
  `Exact` and carry its method — it is **not** admissible under the current `Exact`-everywhere matrix.
  *Disposition:* out of scope for v0; FLAGGED so it cannot be silently folded in (the §4 honesty
  analogue of G2). Ties to the model-accuracy figure being a *pipeline* property, not a codec one.
- **(Q3) Where the packing *selector* lives, and how much is explicit at the call site.** RFC-0016
  §8-Q3 (ergonomics vs the contract) applies: `pack` here takes an explicit `scheme`, but a
  policy-driven selection (RFC-0005, via `std.select`, M-519) may choose it. The split between
  "caller names the scheme" and "selector chooses + emits EXPLAIN" needs a design pass against the
  M-519 surface. *Disposition:* FLAGGED; cross-module, ties to RFC-0016 §8-Q3 and the M-519 boundary
  in §2.
- **(Q4) Width / bignum ceiling.** The M-111 kernel is `i64`-exact up to `m = 40` trits; larger
  widths are out of scope there until a bignum need appears. Whether `std.ternary` exposes only the
  `i64`-bounded surface or anticipates arbitrary-width trits is open. *Disposition:* mirror the M-111
  ceiling for v0 (with the boundary an explicit `None`, per C1); FLAGGED for a later bignum pass.

## Amendment — 2026-07-18: E-W1/M-1119 conversion-utility ceiling widening (append-only)

**Status of this amendment.** Captured 2026-07-18 as part of the W-1 corrective's E-W1 enablement
item (`docs/spec/swaps/binary-ternary.md` §A.5; `docs/planning/design-steer-2026-07-17/
PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §2.2 item 4, tracked as **M-1119**). This is a
**breaking Rust-signature change** to the DN-66-frozen (ADR-045-unfrozen-for-gap-closure) surface,
so per this spec's own §5.5-analogue obligation and ADR-045 §2's "still needs a spec amendment +
changelog entry (G2), not a silent edit," it is recorded here rather than silently landed in code
alone. Status field unchanged (**Accepted**, 2026-06-20, DN-07) — additive amendment, not a
re-ratification.

**What changes.** §7-Q4's flagged width/bignum ceiling is **narrowed, not closed**: `trits_to_int`/
`int_to_trits`/`max_magnitude`'s concrete Rust types move from `i64` to `i128` in both
`mycelium-std-ternary::arithmetic` and `mycelium_core::ternary` (the two crates' call sites agree,
per the M-1119 issue's DoD). §3's design sketch already used the abstract `Int`/`Width` names, so
the *exported-op surface* text is unaffected; only the concrete Rust type behind `Int` in this
crate's actual signatures widens. Consequently:

- `max_magnitude`'s ceiling moves from `m ≥ 41` (in practice `m ≥ 40`, per the corrected finding
  below) to `m ≥ 81` — an explicit `None`, never a silent truncation (C1 unchanged).
- The **W-1 canonical pair** `Binary{64} ↔ Ternary{41}` (`docs/spec/swaps/binary-ternary.md` §A.3)
  is now constructible through this crate's conversion-utility fast path (previously blocked).
- **Corrected finding (VR-5 — grounded by direct computation, not asserted):** the pre-widening
  `i64` ceiling was documented here and in the kernel as `m ≤ 40`, but the actual ceiling — because
  `max_magnitude`'s `3^m` computation itself overflows `i64` one trit before the final quotient
  would have — was `m ≤ 39`. This matches `mycelium-mlir::swap_codegen::MAX_TERNARY_WIDTH_I64 = 39`'s
  independently-documented figure. The `i64 → i128` widening in this amendment supersedes both the
  `m ≤ 40` figure in §7-Q4 above and the narrower actual `m ≤ 39` it should have read.
- **Q4 remains open** (full arbitrary-width `std.ternary` support is still undecided) — only the
  concrete ceiling number moves; the disposition ("mirror the kernel ceiling for v0, explicit `None`
  at the boundary") is unchanged in shape, just at a wider bound. Full arbitrary-width arithmetic
  remains `mycelium_core::ternary::BigTernary` (M-756), not this Ring-1 wrapper's concern; `M-758`
  (`PackedTernary`) stays YAGNI/benchmark-gated, unaffected by this amendment.

**Changelog entry:** FLAGGED for the integrating parent to append to `CHANGELOG.md` (shared file,
not edited directly by this change) — a breaking-signature note for `mycelium-std-ternary`'s
`trits_to_int`/`int_to_trits`/`max_magnitude` (`i64` → `i128`), landed together with the matching
`mycelium-core::ternary` widening and the `mycelium-cert` call-site fixes it required.

## Meta — changelog

- **2026-07-18 — Amendment (append-only; see the Amendment section above).** Records the E-W1/
  M-1119 `i64 → i128` conversion-utility widening as a breaking-but-documented change to this
  DN-66-frozen surface (ADR-045 §2 obligation): `trits_to_int`/`int_to_trits`/`max_magnitude` widen
  from `i64` to `i128`, narrowing (not closing) §7-Q4's open ceiling question from `m ≥ 41`
  (corrected finding: the real prior ceiling was `m ≥ 40`) to `m ≥ 81`, and unblocking the W-1
  canonical `Binary{64} ↔ Ternary{41}` pair through this crate's fast path. `Status` field unchanged
  (**Accepted**, 2026-06-20, DN-07). Append-only.

- **2026-06-17 — Draft (needs-design).** Stands up the `std.ternary` module design spec (M-517, #159;
  Ring 1, Tier A) decomposing RFC-0016 §4.3's ternary differentiator. Fixes the scope/boundary
  (balanced-ternary algebra + `Bit`/`Trit` + packed-ternary helpers, with binary↔ternary swap and the
  packing selector explicitly delegated to M-516/M-519), the exported-op surface sketch, and — the
  load-bearing deliverable — the §4 guarantee matrix: the balanced-ternary algebra and the lossless
  I2_S/TL1/TL2 codecs all tag **`Exact`** (VR-5 — exact because the algebra is exact and the codecs
  are bit-exact, with the range boundary carried by explicit `None`/`Err` fallibility, never a
  weakened tag), and the honesty crux that packing is **inspectable `Meta.physical`** (RFC-0004 §5 /
  DN-01), never a hidden lowering pass (C3/NFR-1). §5 walks the C1–C6 conformance; §6 grounds every
  claim in FR-M2 / M-111 / RFC-0004 §5 / DN-01 / `binary-ternary.md` §1 / RFC-0016. Four questions
  FLAGGED (naming, a future lossy scheme, the selector boundary, the width/bignum ceiling), tied to
  RFC-0016 §8 where applicable. No code; no kernel change (KC-3). Append-only.

- **2026-06-20 — Accepted (maintainer ratification, DN-07).** The maintainer ratified this Rust-first spec: the §4.5 guarantee matrix is asserted in tests, never-silent fallibility and honest per-op tags hold, and the open §7/§8 questions are design/scope calls, not contract violations. No guarantee tag was upgraded without a checked basis (VR-5). Status moves *Implemented (Rust-first) — pending ratification → Accepted*. Append-only; no kernel change (KC-3).
