# phiclm — Phi Compiler Language Model

In Phient's world, a word (w) is a seed: what it relates to, under what conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon. Phical stores no point: a word is 32–64 relations, each a complex phasor on a closed circle; meaning is a side effect of pairs of words moving under a condition, and they move by the error of what was expected — toward what came, away from what did not.

CLM means compiler language model: the compiler a model thinks with, for everything not related to memorizing. The lineage is CPL → BCPL → B → C → phiclm; what is replicated is BCPL's architecture, each stage one of ours: the lexer (`token/`), the parser (`parse/`), the tree (`tree.rs`), TRN to O-code (`trn/`, 55 ops, op 0 a guard), the word machine (`machine/` — one word type, a linear store, `!` `%` and `@`, P-frames in the store, `getvec`'s heap from the top, a step limit, no panics), the global vector (`global/` — BLIB's writers, characters, strings and store), the one manifest (`manifest/` — `g/bcplfecg.h`'s numbering, every token, tree node and op placed on it by `kind()`), and the judge (`judge.rs`).

```rust
let v = phiclm::judge("LET start() = VALOF $( LET s = 0
  FOR i = 1 TO 10 DO s := s + i
  RESULTIS s $)", 100_000);
assert!(v.compiled);
assert_eq!(v.result, 55);
```

The architecture is drawn in [ARCHITECTURE.md](ARCHITECTURE.md), one diagram page per module in [`phinet/docs/phiclm/`](../../docs/phiclm/); what is left is in [TODO.md](TODO.md).

## What of BCPL is here, and what is not

Read against `ext/bcpl/cintcode` (`com/bcplfe.b`, `sysb/blib.b`, `g/libhdr.h`, `sysc/cintsys.c`), each with a judge test in `src/tests/`:

| covered | |
|---|---|
| lexer | keywords case-blind; `$( $) { }` with tags, a tagged `$)tag` closing every section back to its `$(tag`; `//` and nested `/* */` comments; `#x #b #o #` numbers and decimals with `_`; floating constants `1.5`, `2e-3`; strings and characters with every `*` escape (`*n *t *s *e *b *p *c *xhh *#hhhh *##hhhhhhhh`, `*"` `*'` `**`, the `*`…`*` gap); `\/ /\ \= \` and `::`; `[ ]`; the `.` that ends a section; conditional sections `$$tag $<tag $~tag $>tag` |
| declarations | `LET` functions, routines and variables, `LET a, b = x, y` and `AND` declared together (values read before the names are in scope); functions declared in blocks, mutually recursive, seeing the block's static names; `MANIFEST`, `GLOBAL`, `STATIC` blocks of constant expressions (a bare name is the previous value plus one), at top level and in blocks; `VEC k` with `k` constant; `GET "libhdr"` (`ug`, `bytesperword`, `bitsperbyte`, `bitsperword`, `minint`, `maxint`, `endstreamch`); `SECTION` and `NEEDS` read |
| commands | `:=` and `a, b := x, y` (left to right), routine calls, `IF` `UNLESS` `TEST … ELSE`/`OR`, `WHILE` `UNTIL` `REPEAT` `REPEATWHILE` `REPEATUNTIL`, `FOR … BY`, `BREAK` `LOOP`, `RESULTIS` `RETURN` `FINISH`, `SWITCHON … INTO` with `CASE` `DEFAULT` `ENDCASE` (fall-through kept), labels, `GOTO` a label or any expression whose value is one |
| expressions | BCPL's own priorities (`rexp(n)`): `-> ,`, `EQV NEQV XOR`, `\|`, `&`, `~` (over a relation: `~a = b` is `~(a = b)`), the relations (chained: `a < b < c`) and shifts, `+ -`, `* / REM MOD`, `! % OF` (to the left: `v!1!2` is `(v!1)!2`), `f(…)`, `v[i]` and the method call `E#(obj, args…)` (`((!obj)!E)(obj, args…)`); monads `- + ABS ! @ ~ RV LV`; `@` of locals, globals, statics and `v!i`; byte access `s%i`; `SLCT len:sh:offset` and `f OF p` (read and assign); floating `FLOAT FIX #+ #- #* #/ #ABS` and `#= #~= #< #> #<= #>=` (an `f64` in the word, `FIX` to the nearest); `TABLE`, `VALOF`, `TRUE FALSE ? BITSPERBCPLWORD`; constant folding by the machine's own arithmetic; a newline ends an expression unless an operator ends the line |
| BLIB | `writef` (`%s %c %# %n`, `%i %d %x %o %u %z %b %t` with BLIB's width rule, `%n.md`, `%$ %+ %-`, `%f`, `%p`), `writes newline writen wrch writed writez writeu writehex writeoct writebin writet codewrch`, `capitalch compch compstring`, `getvec freevec`, `rdch unrdch readn` (over `judge_with`'s input), `randno setseed`, `muldiv` (remainder in `result2`), `packstring unpackstring`, `createco callco cowait resumeco deleteco` (each coroutine on its own stack in a `getvec` block), `level longjump`, `stop abort`, streams `findinput findoutput selectinput selectoutput endread endwrite input output` (over `judge_files`' named texts; what is written comes back in `Verdict::written`) |

A byte is a word here: strings are one character per word with the length at word 0, so `s%0` is the length and `bytesperword` is 1; a character is a whole Unicode code. Words are 64 bits.

| not ported | why |
|---|---|
| CG: OCODE → Cintcode (`bcplcgcin.b`, `cinterp.c`) | a second interpreter for the same programs is the duplication the workspace rules out; its one gain, running on Richards' cintsys itself, needs packed bytes and BLIB's global numbers, a project of its own |
| `sys`, the console, real files | the judge has no operating system: named texts in, named texts out |
| `+:=` and the other assignment operators | xbcpl's; this distribution's `bcplfe.b` does not parse them either (their codes are on the ladder) |
| a non-constant top-level `LET` | the outer level has no frame: its values must fold |

A broken source is a `Verdict`, never a panic — that is what makes it a judge a learning loop can lean on: [phiano](https://github.com/gemphi/phiano)'s writer learns to write programs against it with no model underneath, and the workshop verifies every page's compiler task against the student's own count. `tell.rs` renders O-code as rows, so the compiler's output is food for the phasor world like any other words.

Formerly named phicpl; renamed at round 195. Part of the [phinet](https://github.com/gemphi/phinet) workspace, the compiler half of [phient](https://github.com/gemphi/phient). MIT OR Apache-2.0.
