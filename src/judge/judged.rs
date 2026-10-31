//! The judge's caps, tested rather than trusted: the declared pair, the
//! time cap's own path, and Property 42 over generated source.

use super::source::{Kind, Source};
use super::{judge_capped, judge_under, Cap, Caps, Verdict, SECONDS_CAP, STEP_CAP};
use phistd::gen::{for_all, replay, CASES_MACHINE};

/// The run's declared seed. A failure names it beside the case.
const SEED: u64 = 0x5EED_C0DE_0000_0042;

#[test]
fn the_declared_caps_are_a_million_steps_and_ten_seconds() {
    assert_eq!((STEP_CAP, SECONDS_CAP), (1_000_000, 10));
    assert_eq!(Caps::declared(), Caps { steps: STEP_CAP, seconds: Some(SECONDS_CAP) });
    assert_eq!(Caps::default(), Caps::declared());
    // The old entry points watch no clock, so their behaviour is unchanged.
    assert_eq!(Caps::steps(7), Caps { steps: 7, seconds: None });
}

#[test]
fn an_endless_program_reaches_the_step_cap_and_is_unsupported() {
    let v = judge_capped("LET start() = VALOF $( LET n = 0\n WHILE TRUE DO n := n + 1\n RESULTIS n $)");
    assert_eq!((v.compiled, v.capped, v.steps), (true, Some(Cap::Steps), STEP_CAP));
    assert!(v.error.is_some() && !v.supported(), "{v:?}");
    assert!(v.rows("p").iter().any(|r| r.kind == "capped" && r.other == "steps"), "{:?}", v.rows("p"));
}

#[test]
fn an_endless_program_reaches_the_time_cap_and_is_unsupported() {
    // A zero-second deadline has passed by the first clock reading, so the
    // wall-clock path is taken in milliseconds rather than in ten seconds.
    let v = judge_under(
        "LET start() = VALOF $( LET n = 0\n WHILE TRUE DO n := n + 1\n RESULTIS n $)",
        "",
        &[],
        Caps { steps: usize::MAX, seconds: Some(0) },
    );
    assert_eq!((v.compiled, v.capped), (true, Some(Cap::Seconds)));
    assert!(v.steps < STEP_CAP, "stopped by the clock, not the steps: {}", v.steps);
    assert!(v.error.is_some() && !v.supported(), "{v:?}");
}

#[test]
fn a_program_that_stops_reaches_no_cap() {
    let v = judge_capped("LET start() = VALOF $( RESULTIS 6 * 7 $)");
    assert_eq!((v.compiled, v.result, v.capped), (true, 42, None), "{:?}", v.error);
    assert!(v.supported() && v.steps < STEP_CAP);
}

/// Every field a `Verdict` must record for every outcome, and what the five
/// of them may say together. A broken invariant is named, not just counted.
fn fields(s: &Source, v: &Verdict) -> Result<(), String> {
    let say = |m: &str| Err(format!("{m}\nkind {:?}, source {:?}\nverdict {v:?}", s.kind, s.text));
    if v.steps > STEP_CAP {
        return say("the step count ran past the declared cap");
    }
    if !v.compiled
        && (v.error.is_none()
            || v.steps != 0
            || !v.out.is_empty()
            || v.result != 0
            || v.capped.is_some())
    {
        return say("a source that did not compile must carry an error and nothing else");
    }
    if v.error.is_none() && (!v.compiled || v.capped.is_some() || !v.supported()) {
        return say("a run with no error must be a compiled, uncapped, supported one");
    }
    match v.capped {
        None => {}
        Some(Cap::Steps) if v.steps != STEP_CAP => {
            return say("the step cap was named with fewer than the capped steps taken")
        }
        Some(_) if v.error.is_none() || v.supported() => {
            return say("a capped run must carry an error and be unsupported")
        }
        Some(_) => {}
    }
    if s.kind == Kind::Endless && v.capped != Some(Cap::Steps) {
        return say("an endless program must reach the step cap");
    }
    // A runaway recursion runs its frames into the heap before either cap
    // reaches it. That is a fault, not a cap — but it is never supported.
    if s.kind == Kind::Runaway && v.supported() {
        return say("a runaway recursion must not come back supported");
    }
    if s.kind == Kind::Fine && !v.compiled {
        return say("a well-formed program must compile");
    }
    Ok(())
}

/// One case: the judge is called on the drawn source — returning at all is
/// the no-panic half of the property — and its `Verdict` is read.
fn checks(s: &Source) -> Result<(), String> {
    fields(s, &judge_capped(&s.text))
}

/// Feature: phinet-64-operators-learning-ladder, Property 42: The judge
/// never panics on generated source, and a capped run is unsupported. For
/// any source text, phiclm's judge returns a Verdict rather than panicking,
/// all five of its fields are recorded, and a run that did not compile or
/// that reached the step or time cap is unsupported with the error and the
/// cap recorded.
///
/// **Validates: Requirements 8.6, 8.13, 7.14**
#[test]
fn the_judge_never_panics_on_generated_source_and_a_capped_run_is_unsupported() {
    // The 64 cases must reach all six arms, or the property passes on a
    // narrower input space than it claims. Drawn, not run: no cost.
    let drawn: Vec<Kind> =
        (0..CASES_MACHINE).map(|c| replay::<Source>(SEED, c).kind).collect();
    for arm in
        [Kind::Fine, Kind::Truncated, Kind::Unbalanced, Kind::Bytes, Kind::Endless, Kind::Runaway]
    {
        assert!(drawn.contains(&arm), "seed {SEED:#018x} draws no {arm:?} in {CASES_MACHINE} cases");
    }
    if let Err(f) = for_all::<Source>(SEED, CASES_MACHINE, |s| checks(s).is_ok()) {
        let why = checks(&f.value).unwrap_err();
        panic!("{f}\n{why}");
    }
}
