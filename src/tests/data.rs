//! Data in the store: bytes, TABLEs, STATICs, the heap, field selectors
//! and floating point.

use crate::judge::judge;

#[test]
fn bytes_make_the_form_rules() {
    // `s%i` reads and writes one character; `s%0` is the length. The form
    // rule docs/192 waited for: a plural, by the word's last letter.
    let v = judge(
        "LET plural(s, v) BE
         $( LET n = s%0
            FOR i = 0 TO n DO v%i := s%i
            SWITCHON s%n INTO
            $( CASE 'y': v%n := 'i'; v%(n+1) := 'e'; v%(n+2) := 's'; v%0 := n + 2; ENDCASE
               CASE 's': CASE 'x': v%(n+1) := 'e'; v%(n+2) := 's'; v%0 := n + 2; ENDCASE
               DEFAULT: v%(n+1) := 's'; v%0 := n + 1
            $)
         $)
         LET start() = VALOF
         $( LET v = VEC 32
            plural(\"city\", v); writef(\"%s \", v)
            plural(\"box\", v); writef(\"%s \", v)
            plural(\"book\", v); writes(v)
            FOR i = 1 TO v%0 DO v%i := capitalch(v%i)
            writef(\" %s\", v)
            RESULTIS \"abc\"%0
         $)",
        100_000,
    );
    assert_eq!(v.out, "cities boxes books BOOKS", "{:?}", v.error);
    assert_eq!(v.result, 3);
}

#[test]
fn a_table_is_a_constant_vector() {
    let v = judge(
        "MANIFEST $( feb = 28 $)
         LET start() = VALOF
         $( LET days = TABLE 31, feb, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31
            LET s = 0
            FOR i = 0 TO 11 DO s := s + days!i
            writef(\"%c\", (TABLE 'a', 'b', 'c')!2)
            RESULTIS s + (TABLE -1, feb / 2)!1
         $)",
        100_000,
    );
    assert_eq!((v.out.as_str(), v.result), ("c", 365 + 14), "{:?}", v.error);
}

#[test]
fn statics_keep_their_value_between_calls() {
    let v = judge(
        "STATIC $( calls = 0 $)
         LET count() = VALOF
         $( STATIC $( seen = 100 $)
            seen := seen + 1
            calls := calls + 1
            RESULTIS seen
         $)
         LET start() = VALOF
         $( LET p = @calls
            count(); count()
            !p := !p + 10
            RESULTIS count() * 1000 + calls
         $)",
        100_000,
    );
    assert_eq!(v.result, 103 * 1000 + 13, "{:?}", v.error);
}

#[test]
fn getvec_takes_from_the_top_and_freevec_gives_back() {
    let v = judge(
        "LET start() = VALOF
         $( LET v = getvec(9)
            LET w = 0
            FOR i = 0 TO 9 DO v!i := i * i
            w := getvec(4)
            IF w + 5 ~= v RESULTIS -1
            freevec(v)
            freevec(w)
            RESULTIS v!9 + (getvec(9) = v -> 1000, 0) + (getvec(1000000) = 0 -> 10000, 0)
         $)",
        100_000,
    );
    assert_eq!(v.result, 81 + 1000 + 10000, "{:?}", v.error);
}

#[test]
fn selectors_read_and_write_fields() {
    let v = judge(
        "MANIFEST $( lo = SLCT 8:0:0; hi = SLCT 8:8:0; next = SLCT 1 $)
         LET start() = VALOF
         $( LET v = VEC 2
            v!0, v!1 := 0, 0
            lo OF v := #x34
            hi OF v := #x12
            next OF v := 99
            writef(\"%x4 %n %n*n\", v!0, hi :: v, next OF v)
            hi OF v := #x1FF
            RESULTIS v!0
         $)",
        100_000,
    );
    assert_eq!(v.out, "1234 18 99\n", "{:?}", v.error);
    assert_eq!(v.result, 0xFF34, "only the field's eight bits change");
}

#[test]
fn floating_point_is_a_word_too() {
    let v = judge(
        "MANIFEST $( half = 1.0 #/ 2.0 $)
         LET start() = VALOF
         $( LET r = 2.0
            LET area = 3.14159 #* r #* r
            LET c = FLOAT 7 #/ 2.0
            writef(\"%n %n %n %n %n*n\", FIX area, FIX c, FIX (#- 2.5), area #> 12.0, FIX ((TABLE 0, half)!1 #* 10.0))
            RESULTIS FIX (c #* 100.0) + (1.5e1 #= 15.0 -> 1000, 0)
         $)",
        100_000,
    );
    assert_eq!(v.out, "13 4 -3 -1 5\n", "{:?}", v.error);
    assert_eq!(v.result, 1350);
}
