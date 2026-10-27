//! Control: SWITCHON, labels and GOTO (named and computed), TEST … OR,
//! and functions declared in blocks.

use crate::judge::judge;

#[test]
fn switchon_dispatches_falls_through_and_ends() {
    // CASE on numbers, characters, manifests and negatives; fall-through
    // into the next CASE; ENDCASE; DEFAULT, and no DEFAULT (`bcplfe.b:2039`).
    let v = judge(
        "MANIFEST $( red = 1; green; blue $)
         LET name(n) = VALOF
         $( SWITCHON n INTO
            $( CASE 1: RESULTIS 10
               CASE 2:
               CASE 3: RESULTIS 23
               CASE -4: RESULTIS 40
               DEFAULT: RESULTIS 0
            $)
         $)
         LET start() = VALOF
         $( LET s = 0
            FOR i = 0 TO 4 DO
              SWITCHON i INTO
              $( CASE red:   s := s + 1; ENDCASE
                 CASE green: s := s + 10
                 CASE blue:  s := s + 100; ENDCASE
              $)
            RESULTIS s + name(1) + name(3) + name(-4) + name(9)
         $)",
        100_000,
    );
    assert_eq!(v.result, 211 + 10 + 23 + 40, "{:?}", v.error);
}

#[test]
fn goto_reaches_labels_before_and_after() {
    let v = judge(
        "LET start() = VALOF
         $( LET n = 0
            LET s = 0
         again:
            n := n + 1
            s := s + n
            IF n < 10 GOTO again
            GOTO done
            s := 999
         done:
            RESULTIS s
         $)",
        100_000,
    );
    assert_eq!(v.result, 55, "{:?}", v.error);
}

#[test]
fn test_or_and_the_computed_goto() {
    let v = judge(
        "LET start() = VALOF
         $( LET n = 0
            LET there = 0
            TEST n = 0 THEN n := 5 OR n := 6
            there := n = 5 -> five, six
            GOTO there
         five: RESULTIS 50
         six:  RESULTIS 60
         $)",
        100_000,
    );
    assert_eq!(v.result, 50, "{:?}", v.error);
}

#[test]
fn functions_can_be_declared_in_blocks() {
    // A block's functions recurse into each other and see the block's
    // manifests and statics; a frame's variable is out of their reach.
    let v = judge(
        "LET start() = VALOF
         $( MANIFEST $( base = 100 $)
            STATIC $( calls = 0 $)
            LET even(n) = VALOF $( calls := calls + 1; RESULTIS n = 0 -> TRUE, odd(n - 1) $)
            AND odd(n) = n = 0 -> FALSE, even(n - 1)
            LET say(n) BE writef(\"%n \", base + n)
            say(1)
            RESULTIS (even(10) -> 1, 0) * 1000 + calls
         $)",
        100_000,
    );
    assert_eq!((v.out.as_str(), v.result), ("101 ", 1006), "{:?}", v.error);
    let v = judge("LET start() = VALOF $( LET x = 1\n LET f() = x\n RESULTIS f() $)", 1000);
    assert!(!v.compiled && v.error.unwrap_or_default().contains("unknown name x"));
}
