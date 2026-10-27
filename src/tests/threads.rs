//! Threads of control: coroutines as BLIB runs them (`createco`, `callco`,
//! `cowait`, `resumeco`, `deleteco`), and `level`/`longjump` out of a deep
//! recursion.

use crate::judge::judge;

#[test]
fn a_coroutine_is_a_generator() {
    // The body runs as `fn(arg)` on the first callco; each cowait hands a
    // value out and waits; when the body returns, the coroutine is fresh.
    let v = judge(
        "LET squares(n) = VALOF
         $( FOR i = 1 TO n DO cowait(i * i)
            RESULTIS 0
         $)
         LET start() = VALOF
         $( LET g = createco(squares, 200)
            LET s = 0
            LET x = callco(g, 4)
            WHILE x DO
            $( s := s + x
               x := callco(g, 0)
            $)
            RESULTIS deleteco(g) -> s, -1
         $)",
        100_000,
    );
    assert_eq!(v.result, 1 + 4 + 9 + 16, "{:?}", v.error);
}

#[test]
fn resumeco_hands_on_the_parent() {
    let v = judge(
        "STATIC $( a = 0; b = 0 $)
         LET first(x) = VALOF
         $( writef(\"first %n \", x)
            x := resumeco(b, x + 1)
            writef(\"first again %n \", x)
            RESULTIS x * 10
         $)
         LET second(x) = VALOF
         $( writef(\"second %n \", x)
            RESULTIS cowait(x + 1)
         $)
         LET start() = VALOF
         $( a := createco(first, 200)
            b := createco(second, 200)
            LET r = callco(a, 1)
            writef(\"start %n \", r)
            r := callco(a, 7)
            writef(\"start %n\", r)
            RESULTIS r
         $)",
        100_000,
    );
    assert_eq!(v.out, "first 1 second 2 start 3 first again 7 start 70", "{:?}", v.error);
    assert_eq!(v.result, 70);
}

#[test]
fn the_main_program_stops_by_cowait_stop_or_abort() {
    assert_eq!(judge("LET start() = VALOF $( cowait(42); RESULTIS 0 $)", 1000).result, 42);
    assert_eq!(judge("LET start() = VALOF $( stop(7); RESULTIS 0 $)", 1000).result, 7);
    let v = judge("LET start() = VALOF $( abort(3); RESULTIS 0 $)", 1000);
    assert_eq!(v.error.as_deref(), Some("abort(3)"));
    let v = judge(
        "LET body(x) = callco(x, 0)
         LET start() = VALOF $( LET c = createco(body, 100)\n RESULTIS callco(c, c) $)",
        10_000,
    );
    assert!(v.error.unwrap_or_default().contains("abort 110"), "a running coroutine cannot be called again");
}

#[test]
fn longjump_leaves_a_deep_recursion() {
    let v = judge(
        "GLOBAL $( errlev: 0; errlab: 1 $)
         LET deep(n) = VALOF
         $( IF n = 0 DO longjump(errlev, errlab)
            RESULTIS deep(n - 1) + 1
         $)
         LET start() = VALOF
         $( errlev := level()
            errlab := fail
            writef(\"%n \", deep(30) + 100)
            RESULTIS 0
         fail:
            writes(\"recovered\")
            RESULTIS 99
         $)",
        100_000,
    );
    assert_eq!((v.out.as_str(), v.result), ("recovered", 99), "{:?}", v.error);
}
