//! The conditions of satisfaction, fixed in docs/178 before the build.

use crate::judge::judge;

#[test]
fn fact_and_the_loop() {
    // 1. fact(6) = 720 recursively, sum 1..100 = 5050 by loop.
    let v = judge("LET fact(n) = n = 0 -> 1, n * fact(n - 1)\nLET start() = fact(6)", 100_000);
    assert_eq!((v.compiled, v.error, v.result), (true, None, 720));
    let v = judge(
        "LET start() = VALOF
         $( LET s = 0
            FOR i = 1 TO 100 DO s := s + i
            RESULTIS s
         $)",
        100_000,
    );
    assert_eq!(v.result, 5050, "{:?}", v.error);
}

#[test]
fn the_store_is_real() {
    // 2. VEC + ! + @: write through an address, read it back.
    let v = judge(
        "LET start() = VALOF
         $( LET v = VEC 10
            AND p = 0
            FOR i = 0 TO 10 DO v!i := i * i
            p := @v
            RESULTIS v!7 + (!p)!3   // 49 + v!3 = 49 + 9
         $)",
        100_000,
    );
    assert_eq!(v.result, 58, "{:?}", v.error);
}

#[test]
fn the_judge_reads_what_was_written() {
    // 3. writef's output, exactly.
    let v = judge(
        "LET fact(n) = n = 0 -> 1, n * fact(n - 1)
         LET start() = VALOF
         $( writef(\"fact(%n) = %n*n\", 5, fact(5))
            RESULTIS 0
         $)",
        100_000,
    );
    assert_eq!(v.out, "fact(5) = 120\n");
    assert!(v.rows("fact").iter().any(|r| r.kind == "wrote"));
}

#[test]
fn a_broken_source_is_a_verdict_not_a_panic() {
    // 4. generated code must be judgeable.
    let v = judge("LET start() = VALOF $( RESULTIS", 1000);
    assert!(!v.compiled && v.error.is_some());
    let v = judge("LET start() = start()", 10_000);
    assert!(v.compiled && v.error.is_some(), "infinite recursion runs out of steps or store");
    let v = judge("LET start() = 1 / 0", 1000);
    assert_eq!(v.error.as_deref(), Some("division by zero"));
}

#[test]
fn the_language_holds_together() {
    // The structured commands, the loops, BREAK/LOOP, strings, characters.
    let v = judge(
        "LET gcd(a, b) = VALOF
         $( UNTIL b = 0 DO
            $( LET t = b
               b := a REM b
               a := t
            $)
            RESULTIS a
         $)
         LET start() = VALOF
         $( LET n = 0
            FOR i = 1 TO 30 DO
            $( IF i REM 3 = 0 LOOP
               IF i > 20 BREAK
               n := n + 1
            $)
            TEST gcd(48, 36) = 12 THEN n := n + 100 ELSE n := n - 100
            WHILE n < 200 DO n := n + ('b' - 'a')
            RESULTIS n
         $)",
        1_000_000,
    );
    assert_eq!(v.result, 200, "{:?}", v.error);
}
