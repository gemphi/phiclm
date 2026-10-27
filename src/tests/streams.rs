//! Streams over the judge's world: named inputs opened with `findinput`,
//! outputs written with `findoutput` and handed back in the verdict, and the
//! judge's own input and output current again after `endread`/`endwrite`.

use crate::judge::judge_files;

#[test]
fn a_program_reads_a_file_and_writes_another() {
    let v = judge_files(
        "LET start() = VALOF
         $( LET in = findinput(\"words\")
            LET out = findoutput(\"plural\")
            LET n, ch = 0, 0
            IF in = 0 RESULTIS -1
            IF findinput(\"missing\") ~= 0 RESULTIS -2
            selectinput(in)
            selectoutput(out)
            ch := rdch()
            UNTIL ch = -1 DO
            $( TEST ch = ' ' THEN $( wrch('s'); wrch(' '); n := n + 1 $) ELSE wrch(ch)
               ch := rdch()
            $)
            wrch('s'); n := n + 1
            endwrite()
            endread()
            writef(\"%n words\", n)
            RESULTIS (output() = 2 & input() = 1 -> 1, 0) + readn() * 10
         $)",
        "7",
        &[("words", "cat dog")],
        100_000,
    );
    assert_eq!(v.out, "2 words", "{:?}", v.error);
    assert_eq!(v.written, vec![("plural".to_string(), "cats dogs".to_string())]);
    assert_eq!(v.result, 71, "the judge's own streams are current again");
}

#[test]
fn a_closed_stream_cannot_be_selected() {
    let v = judge_files(
        "LET start() = VALOF
         $( LET out = findoutput(\"log\")
            selectoutput(out)
            writes(\"kept\")
            endwrite()
            selectoutput(out)
            RESULTIS 0
         $)",
        "",
        &[],
        10_000,
    );
    assert!(v.error.unwrap_or_default().contains("closed"));
    assert_eq!(v.written, vec![("log".to_string(), "kept".to_string())], "what was written before the close is kept");
}
