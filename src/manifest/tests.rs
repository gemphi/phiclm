//! The ladder's own tests: one numbering, contiguous groups, and every
//! token, node and op on its rung.

use super::*;

#[test]
fn the_ladder_is_one_numbering_and_the_groups_are_its_ranges() {
    assert_eq!(S::Number.code(), 1, "the header starts at one");
    assert!(S::Mul < S::Div && S::Div < S::Rem && S::Rem < S::Add && S::Add < S::Sub);
    // groups are contiguous, ordered and disjoint — the hierarchy is the order
    for w in GROUPS.windows(2) {
        assert!(w[0].last < w[1].first, "{} runs into {}", w[0].name, w[1].name);
    }
    assert_eq!(S::Add.group(), "arithmetic");
    assert_eq!(S::Ge.group(), "relation");
    assert_eq!(S::Let.group(), "declaration");
    assert_eq!(S::Stind.group(), "ocode");
    assert_eq!(S::Add.name(), "s_add");
    assert_eq!(Sf::Neqv as u8, 16, "the sub-enum is the header's too");
}

#[test]
fn our_ocode_climbs_the_same_ladder() {
    use crate::ocode::Op;
    assert_eq!(S::from(&Op::Add).group(), "arithmetic");
    assert_eq!(S::from(&Op::Jf(0)).group(), "ocode");
    assert_eq!(S::from(&Op::Num(7)), S::Ln);
    assert_eq!(S::from(&Op::Getbyte).group(), "ocode");
    assert_eq!(S::from(&Op::Neqv).group(), "logic");
    assert_eq!(S::from(&Op::Fix).group(), "floating");
    assert_eq!(S::from(&Op::Goto).group(), "command");
}

#[test]
fn every_token_has_its_rung() {
    use crate::token::{lex, Token, KEYWORDS};
    // Every keyword lands on the ladder.
    for k in KEYWORDS {
        assert_ne!(Token::Key(k).kind(), S::None, "{k} has no rung");
    }
    assert_eq!(Token::Key("float").kind().group(), "floating");
    assert_eq!(Token::Key("then").kind(), S::Do, "THEN is s_do");
    assert_eq!(Token::Key("xor").kind(), S::Neqv, "XOR is s_neqv");
    assert_eq!(Token::Key("mod").kind(), S::Rem, "MOD is s_rem");
    // Every piece of punctuation the lexer makes.
    let all = lex("$( $) { } := ~= <= >= << >> -> ( ) [ ] , ! @ + - * / = < > & | ~ ? : % :: #+ #- #* #/ #= #~= #< #> #<= #>= #abs 1.5").expect("lexes");
    for t in &all {
        assert_ne!(t.kind(), S::None, "{t:?} has no rung");
    }
    let groups: Vec<&str> = all.iter().map(|t| t.kind().group()).collect();
    assert_eq!(&groups[..4], ["lexical"; 4]);
    assert_eq!(Token::Pun("%").kind().group(), "selection");
    assert_eq!(Token::Pun(":=").kind().group(), "command");
    assert_eq!(Token::Num(3).kind().group(), "constant");
    assert_eq!(Token::Sep.kind().group(), "lexical");
    assert_eq!(Token::Pun("#<=").kind().group(), "floating");
    assert_eq!(Token::Fnum(1.5).kind(), S::Fnum);
}

#[test]
fn every_tree_node_lands_in_its_group() {
    use crate::tree::{Command, Declaration, Element, Expression};
    let n = || Box::new(Expression::Num(1));
    let c = || Box::new(Command::Return);
    let expressions = [
        (Expression::Num(1), "constant"),
        (Expression::Name("x".into()), "constant"),
        (Expression::Str("s".into()), "constant"),
        (Expression::True, "constant"),
        (Expression::False, "constant"),
        (Expression::Monad("-", n()), "monadic"),
        (Expression::Monad("abs", n()), "monadic"),
        (Expression::Monad("!", n()), "access"),
        (Expression::Monad("@", n()), "access"),
        (Expression::Monad("~", n()), "logic"),
        (Expression::Dyad("!", n(), n()), "access"),
        (Expression::Dyad("%", n(), n()), "selection"),
        (Expression::Dyad("rem", n(), n()), "arithmetic"),
        (Expression::Dyad(">=", n(), n()), "relation"),
        (Expression::Dyad("eqv", n(), n()), "logic"),
        (Expression::Cond(n(), n(), n()), "logic"),
        (Expression::Call(n(), vec![]), "access"),
        (Expression::Valof(c()), "access"),
        (Expression::Vec(n()), "declaration"),
        (Expression::Table(vec![]), "logic"),
        (Expression::Fnum(1.5), "floating"),
        (Expression::Monad("float", n()), "floating"),
        (Expression::Monad("#abs", n()), "floating"),
        (Expression::Dyad("#+", n(), n()), "floating"),
        (Expression::Dyad("#<", n(), n()), "floating"),
        (Expression::Dyad("of", n(), n()), "selection"),
        (Expression::Slct(n(), n(), n()), "selection"),
    ];
    for (e, group) in &expressions {
        assert_eq!(e.kind().group(), *group, "{e:?}");
    }
    let commands = [
        Command::Assign(Expression::Num(0), Expression::Num(0)),
        Command::Do(Expression::Num(0)),
        Command::If(Expression::Num(0), c()),
        Command::Unless(Expression::Num(0), c()),
        Command::Test(Expression::Num(0), c(), c()),
        Command::While(Expression::Num(0), c()),
        Command::Until(Expression::Num(0), c()),
        Command::Repeat(c()),
        Command::RepeatWhile(c(), Expression::Num(0)),
        Command::RepeatUntil(c(), Expression::Num(0)),
        Command::For("i".into(), Expression::Num(0), Expression::Num(1), None, c()),
        Command::Resultis(Expression::Num(0)),
        Command::Return,
        Command::Finish,
        Command::Break,
        Command::Loop,
        Command::Switchon(Expression::Num(0), c()),
        Command::Case(Expression::Num(0), c()),
        Command::Default(c()),
        Command::Endcase,
        Command::Goto(Expression::Name("l".into())),
        Command::Label("l".into(), c()),
    ];
    for k in &commands {
        assert_eq!(k.kind().group(), "command", "{k:?}");
    }
    assert_eq!(Command::Block(vec![]).kind(), S::Seq, "a block is a sequence");
    assert_eq!(Element::Let(vec![]).kind().group(), "declaration");
    assert_eq!(Element::Do(Command::Endcase).kind(), S::Endcase);
    let declarations = [
        Declaration::Function("f".into(), vec![], Expression::Num(0)),
        Declaration::Routine("r".into(), vec![], Command::Return),
        Declaration::Variable("v".into(), Expression::Num(0)),
        Declaration::Manifest(vec![]),
        Declaration::Global(vec![]),
        Declaration::Static(vec![]),
    ];
    for d in &declarations {
        assert_eq!(d.kind().group(), "declaration", "{d:?}");
    }
}
