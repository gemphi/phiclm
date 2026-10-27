//! BLIB: what `writef` and the writers print, and what the readers and
//! arithmetic return.

use crate::judge::judge;

#[test]
fn blib_writes_as_blib_does() {
    // writef's width is the one character after the type (`blib.b:790`),
    // writehex writes capitals, a sign goes before zeroes and after spaces.
    let v = judge(
        "LET start() = VALOF
         $( writef(\"[%i5][%i0][%x4][%o3][%b8][%z4]*n\", 42, 7, 255, 8, 5, -42)
            writed(-7, 4); newline()
            writehex(#xBEEF, 4); writeoct(8, 4); newline()
            writef(\"%n item%ps, %n box%p\\\\es\\, %t6|%%*n\", 2, 2, 1, 1, \"ab\")
            writef(\"%5.2d %$%n*n\", 31415, 99, 7)
            RESULTIS compstring(\"Apple\", \"apple\") * 100 + compstring(\"ab\", \"abc\") + 10 * compch('a', 'B')
         $)",
        100_000,
    );
    assert_eq!(v.out, "[   42][7][00FF][010][00000101][-042]\n  -7\nBEEF0010\n2 items, 1 box, ab    |%\n314.15 7\n", "{:?}", v.error);
    assert_eq!(v.result, -11);
}

#[test]
fn blib_reads_its_input_and_does_its_arithmetic() {
    let v = crate::judge_with(
        "LET start() = VALOF
         $( LET a = readn()
            LET b = readn()
            LET c = readn()
            LET ok = result2
            LET ch = rdch()
            LET q = muldiv(1000000007, 1000000007, 1000000009)
            writef(\"%n %n %n %n %c %n %n\", a, b, c, ok, ch, rdch(), result2)
            RESULTIS q
         $)",
        "  42 -7 x",
        100_000,
    );
    let p = 1_000_000_007i128 * 1_000_000_007;
    assert_eq!(v.out, format!("42 -7 0 -1 x -1 {}", p % 1_000_000_009), "{:?}", v.error);
    assert_eq!(v.result as i128, p / 1_000_000_009);
    let v = judge(
        "LET start() = VALOF
         $( LET v = VEC 10
            LET s = VEC 10
            LET first, inside = 0, TRUE
            unpackstring(\"abc\", v)
            v!2 := 'X'
            packstring(v, s)
            writes(s)
            setseed(1)
            first := randno(6)
            FOR i = 1 TO 100 DO UNLESS 1 <= randno(6) <= 6 DO inside := FALSE
            setseed(1)
            RESULTIS inside & first = randno(6) -> 1, 0
         $)",
        100_000,
    );
    assert_eq!((v.out.as_str(), v.result), ("aXc", 1), "{:?}", v.error);
}
