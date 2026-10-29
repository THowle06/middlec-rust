use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*+[^*/])*\*+/")]
pub enum Token {
    // Keywords
    #[token("int")]
    KwInt,
    #[token("char")]
    KwChar,
    #[token("bool")]
    KwBool,
    #[token("void")]
    KwVoid,
    #[token("if")]
    KwIf,
    #[token("else")]
    KwElse,
    #[token("while")]
    KwWhile,
    #[token("for")]
    KwFor,
    #[token("return")]
    KwReturn,
    #[token("break")]
    KwBreak,
    #[token("continue")]
    KwContinue,
    #[token("struct")]
    KwStruct,
    #[token("union")]
    KwUnion,
    #[token("enum")]
    KwEnum,
    #[token("sizeof")]
    KwSizeof,
    #[token("true")]
    KwTrue,
    #[token("false")]
    KwFalse,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority = 1)]
    Ident(String),

    // Numbers
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    IntLit(i64),

    // Character literals
    #[regex(r"'(\\.|[^\\'])'", |lex| {
        let s = lex.slice();
        if s.starts_with(r"'\") {
            match s.chars().nth(2) {
                Some('n') => Some('\n'),
                Some('t') => Some('\t'),
                Some('0') => Some('\0'),
                Some(c) => Some(c),
                None => None,
            }
        } else {
            s.chars().nth(1)
        }
    })]
    CharLit(char),

    // String literals
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len() - 1].to_string()
    })]
    StringLit(String),

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(";")]
    Semi,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("?")]
    Question,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("=")]
    Assign,
    #[token("+=")]
    AddAssign,
    #[token("-=")]
    SubAssign,
    #[token("*=")]
    MulAssign,
    #[token("/=")]
    DivAssign,
    #[token("%=")]
    ModAssign,
    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("!")]
    Bang,
    #[token("&")]
    Ampersand,
    #[token(".")]
    Dot,
    #[token("->")]
    Arrow,
}
