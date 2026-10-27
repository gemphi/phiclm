//! Syntax as BCPL reads it: the priorities, lists, `libhdr` and constant
//! blocks, and the lexer's conditions and spellings.

use crate::judge::judge;

#[test]
fn eqv_and_neqv_sit_below_or() {
    let v = judge(
        "LET start() = VALOF
         $( LET a = #b1100
            LET b = #b1010
            writef(\"%b4 %b4 %b4*n\", a NEQV b, a XOR b, (a EQV b) & #b1111)
            RESULTIS (1 | 2 EQV 3) = TRUE & (TRUE EQV FALSE) = FALSE -> BITSPERBCPLWORD, 0
         $)",
        100_000,
    );
    assert_eq!(v.out, "0110 0110 1001\n", "{:?}", v.error);
    assert_eq!(v.result, 64);
}

#[test]
fn precedence_is_bcpls() {
    // `~` takes a relation, relations chain, `!` groups to the left, and an
    // operator that ends a line carries the expression onto the next.
    let v = judge(
        "LET start() = VALOF
         $( LET v = VEC 4
            LET w = VEC 2
            LET a, b = 3, 4
            v!1 := w
            w!2 := 77
            writef(\"%n %n %n %n %n*n\", ~ a = b, 1 < a < b, 1 < b < a, v!1!2, -a * b)
            RESULTIS a +
                     b
         $)",
        100_000,
    );
    assert_eq!(v.out, "-1 -1 0 77 -12\n", "{:?}", v.error);
    assert_eq!(v.result, 7);
}

#[test]
fn lets_and_assignments_take_lists() {
    // `LET a, b = b, a` reads the outer names; `a, b := x, y` goes left to
    // right (`bcplfe.b` `assign`), so `b` sees the new `a`.
    let v = judge(
        "LET start() = VALOF
         $( LET a, b = 1, 2
            $( LET a, b = b, a
               writef(\"%n%n \", a, b)
            $)
            a, b := 10, a + 5
            writef(\"%n %n\", a, b)
            RESULTIS 0
         $)",
        100_000,
    );
    assert_eq!(v.out, "21 10 15", "{:?}", v.error);
}

#[test]
fn libhdr_and_constant_blocks() {
    // GET "libhdr" declares the header's manifests; a block's values are
    // constant expressions; a local shadows a manifest of the same name.
    let v = judge(
        "GET \"libhdr\"
         GLOBAL $( count: ug; total: ug + 1 $)
         MANIFEST $( size = 256 / bytesperword; half = size / 2 $)
         LET bump() BE count := count + 1
         LET start() = VALOF
         $( LET maxint = 5
            LET buf = VEC size
            bump(); bump()
            total := maxint + half
            RESULTIS count * 1000 + total + (endstreamch = -1 -> 10000, 0)
         $)",
        100_000,
    );
    assert_eq!(v.result, 2 * 1000 + 5 + 128 + 10_000, "{:?}", v.error);
}

#[test]
fn the_lexer_s_conditions_and_spellings_compile() {
    let v = judge(
        "$$fast
         LET start() = VALOF
         $( LET v = VEC 3 /* a /* nested */ comment */
            v[0], v[1] := 1, 2
            writes(\"tab*there*n\")
            $<fast RESULTIS v[0] \\/ v[1] $>fast
            $<slow RESULTIS 0 $>slow
            RESULTIS -1
         $)
         .",
        100_000,
    );
    assert_eq!((v.out.as_str(), v.result), ("tab\there\n", 3), "{:?}", v.error);
}

#[test]
fn a_tagged_close_shuts_every_section_back_to_its_tag() {
    // `$)2` closes the IF's block and the FOR body: the doubling runs once,
    // after the loop. Were each `$)` one section, it would run inside it.
    let v = judge(
        "LET start() = VALOF
         $(1 LET s = 0
             FOR i = 1 TO 3 DO
             $(2 s := s + i
                 IF i = 2 DO $( s := s + 100 $)2
             s := s * 2
             RESULTIS s
         $)1",
        100_000,
    );
    assert_eq!(v.result, (1 + 2 + 100 + 3) * 2, "{:?}", v.error);
    assert!(!judge("LET start() = VALOF $( RESULTIS 1 $)9", 1000).compiled, "a tag no section opened");
}

#[test]
fn a_method_call_goes_through_the_object_s_table() {
    // `E#(obj, args…)` is `((!obj)!E)(obj, args…)`: word 0 of the object
    // holds its method table.
    let v = judge(
        "LET area(obj) = obj!1 * obj!2
         LET perimeter(obj) = 2 * (obj!1 + obj!2)
         LET scaled(obj, k) = k * area(obj)
         LET start() = VALOF
         $( LET mt = VEC 2
            LET rect = VEC 3
            mt!0, mt!1, mt!2 := area, perimeter, scaled
            rect!0, rect!1, rect!2 := mt, 3, 4
            RESULTIS 0#(rect) * 100 + 1#(rect) + 2#(rect, 10) * 10000
         $)",
        100_000,
    );
    assert_eq!(v.result, 12 * 100 + 14 + 120 * 10_000, "{:?}", v.error);
}
