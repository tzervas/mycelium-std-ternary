//! In-crate white-box test modules (test layout rule: no tests in logic files; one submodule per
//! source module under test). Extracted as-touched (M-797): `arithmetic` is extracted here by the
//! E-W1/M-1119 change (which had to touch `arithmetic.rs`'s logic anyway); the crate's other
//! `#[cfg(test)] mod tests { .. }` inline modules (`guarantee_matrix`, `packing`, `primitives`)
//! stay inline for now, per the lazy, as-touched retrofit policy — no big-bang refactor.

mod arithmetic;
