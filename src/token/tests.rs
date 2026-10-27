//! The lexer against BCPL's spellings, escapes, comments and conditions.

use super::*;

#[test]
fn the_lexer_reads_bcpl_spellings() {
    use Token::*;
    let t = lex("x::p \\/ a /\\ b \\= c #x1F_FF #b1100_0000 #777 1_000 1.5 2e-3 #+ #<= #ABS v[1]").unwrap();
    assert_eq!(
        t,
        vec![
            Name("x".into()), Key("of"), Name("p".into()), Pun("|"), Name("a".into()), Pun("&"), Name("b".into()),
            Pun("~="), Name("c".into()), Num(0x1fff), Num(0b1100_0000), Num(0o777), Num(1000), Fnum(1.5),
            Fnum(2e-3), Pun("#+"), Pun("#<="), Pun("#abs"), Name("v".into()), Pun("["), Num(1), Pun("]"),
        ]
    );
}

#[test]
fn escapes_comments_and_conditions() {
    let t = lex("\"a*tb*x41*#263A*\"*s*n\" '*'' /* a /* nested */ comment */ $$on $<on kept $>on $<off dropped $>off $~on gone $>on").unwrap();
    assert_eq!(t, vec![Token::Str("a\tbA\u{263A}\" \n".into()), Token::Chr('\'' as i64), Token::Name("kept".into())]);
    let t = lex("\"one *\n    * two\"").unwrap();
    assert_eq!(t, vec![Token::Str("one  two".into())]);
    assert!(lex("\"bad *q\"").is_err());
    assert!(lex("/* open").is_err());
}
