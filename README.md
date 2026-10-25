# phiclm — Phi Compiler Language Model

In Phient's world, a word (w) is a seed: what it relates to, under what conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon. Phical stores no point: a word is 32–64 relations, each a complex phasor on a closed circle with a count; meaning is a side effect of pairs of words moving under a condition.

CLM means compiler language model: the compiler a model thinks with, for everything not related to memorizing. The lineage is CPL → BCPL → B → C → phiclm; what is replicated is BCPL's architecture, each stage one of ours: the lexer (`token.rs`), the parser (`parse/`), the tree (`tree.rs`), TRN to O-code (`trn/`, ~35 ops, op 0 a guard), the word machine (`machine.rs` — one word type, a linear store, `!` and `@`, P-frames in the store, a step limit, no panics), the global vector (`global.rs` — writef, writes, newline, writen, wrch), and the judge (`judge.rs`).

```rust
let v = phiclm::judge("LET start() = VALOF $( LET s = 0
  FOR i = 1 TO 10 DO s := s + i
  RESULTIS s $)", 100_000);
assert!(v.compiled);
assert_eq!(v.result, 55);
```

A broken source is a `Verdict`, never a panic — that is what makes it a judge a learning loop can lean on: [phiano](https://github.com/gemphi/phiano)'s writer learns to write programs against it with no model underneath, and the workshop verifies every page's compiler task against the student's own count. `tell.rs` renders O-code as rows, so the compiler's output is food for the phasor world like any other words.

Formerly named phicpl; renamed at round 195. Part of the [phinet](https://github.com/gemphi/phinet) workspace, the compiler half of [phient](https://github.com/gemphi/phient). MIT OR Apache-2.0.
