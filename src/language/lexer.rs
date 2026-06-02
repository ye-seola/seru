use logos::{Logos, Span};

#[derive(Debug, PartialEq, Clone)]
pub struct SpannedToken<'s> {
    pub token: Token<'s>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StringPart {
    pub kind: StringPartKind,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StringPartKind {
    Text(String),
    Interpolation(String),
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"\s+")]
pub enum Token<'s> {
    #[token("if")]
    If,

    #[token("for")]
    For,

    #[token("in")]
    In,

    #[token("component")]
    Component,

    #[token("Slot")]
    Slot,

    #[token("const")]
    Const,

    // L, R
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    // comment
    #[regex(r"//[^\r\n]*", allow_greedy = true)]
    Comment(&'s str),

    // boolean
    #[token("true")]
    True,
    #[token("false")]
    False,

    // operator
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("/")]
    Divide,

    #[token("*")]
    Multiple,

    #[token("=")]
    Assign,

    // cond operator
    #[token("==")]
    Eq,

    #[token("!=")]
    Neq,

    // ETC
    #[token("null")]
    Null,

    #[token("\"", parse_string)]
    String(Vec<StringPart>),

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    // number
    #[regex(r"(0|[1-9][0-9]*)\.[0-9]+")]
    Float(&'s str),

    #[regex("0|[1-9][0-9]*")]
    Int(&'s str),

    // identifier
    #[regex(r"[_\p{XID_Start}]\p{XID_Continue}*")]
    Identifier(&'s str),

    EOF,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SyntaxKind {
    If,
    For,
    In,
    Component,
    Slot,
    Const,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comment,
    True,
    False,
    Plus,
    Minus,
    Divide,
    Multiple,
    Assign,
    Eq,
    Neq,
    Null,
    String,
    Comma,
    Colon,
    Float,
    Int,
    Identifier,
    EOF,
}

impl From<SpannedToken<'_>> for SyntaxKind {
    fn from(spanned_token: SpannedToken<'_>) -> Self {
        spanned_token.token.into()
    }
}

impl From<Token<'_>> for SyntaxKind {
    fn from(token: Token) -> Self {
        match token {
            Token::If => SyntaxKind::If,
            Token::For => SyntaxKind::For,
            Token::In => SyntaxKind::In,
            Token::Component => SyntaxKind::Component,
            Token::Slot => SyntaxKind::Slot,
            Token::Const => SyntaxKind::Const,
            Token::LParen => SyntaxKind::LParen,
            Token::RParen => SyntaxKind::RParen,
            Token::LBracket => SyntaxKind::LBracket,
            Token::RBracket => SyntaxKind::RBracket,
            Token::LBrace => SyntaxKind::LBrace,
            Token::RBrace => SyntaxKind::RBrace,
            Token::Comment(_) => SyntaxKind::Comment,
            Token::True => SyntaxKind::True,
            Token::False => SyntaxKind::False,
            Token::Plus => SyntaxKind::Plus,
            Token::Minus => SyntaxKind::Minus,
            Token::Divide => SyntaxKind::Divide,
            Token::Multiple => SyntaxKind::Multiple,
            Token::Assign => SyntaxKind::Assign,
            Token::Eq => SyntaxKind::Eq,
            Token::Neq => SyntaxKind::Neq,
            Token::Null => SyntaxKind::Null,
            Token::String(_) => SyntaxKind::String,
            Token::Comma => SyntaxKind::Comma,
            Token::Colon => SyntaxKind::Colon,
            Token::Float(_) => SyntaxKind::Float,
            Token::Int(_) => SyntaxKind::Int,
            Token::Identifier(_) => SyntaxKind::Identifier,
            Token::EOF => SyntaxKind::EOF,
        }
    }
}

pub fn lex_with_offset<'s>(src: &'s str, offset: usize) -> anyhow::Result<Vec<SpannedToken<'s>>> {
    let mut tokens = Vec::new();

    let mut lex = Token::lexer(src);

    while let Some(result) = lex.next() {
        let span = lex.span();
        let span = Span {
            start: offset + span.start,
            end: offset + span.end,
        };

        match result {
            Ok(Token::Comment(_)) => continue,
            Ok(token) => {
                tokens.push(SpannedToken { token, span });
            }
            Err(_) => {
                anyhow::bail!("invalid syntax: {:?}", span);
            }
        }
    }

    Ok(tokens)
}

pub fn lex<'s>(src: &'s str) -> anyhow::Result<Vec<SpannedToken<'s>>> {
    lex_with_offset(src, 0)
}

fn parse_string<'s>(lex: &mut logos::Lexer<'s, Token<'s>>) -> Option<Vec<StringPart>> {
    let rest = lex.remainder();
    let mut chars = rest.char_indices().peekable();

    let mut parts = Vec::new();
    let mut text = String::new();

    let string_start = lex.span().start + 1;
    let mut segment_start = 0;

    while let Some((i, ch)) = chars.next() {
        match ch {
            '"' => {
                if !text.is_empty() {
                    parts.push(StringPart {
                        kind: StringPartKind::Text(text),
                        span: Span {
                            start: string_start + segment_start,
                            end: string_start + i,
                        },
                    });
                }

                lex.bump(i + ch.len_utf8());
                return Some(parts);
            }

            '\\' => {
                let (_, escaped) = chars.next()?;

                let real = match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    '$' => '$',
                    other => other,
                };

                text.push(real);
            }

            '$' if matches!(chars.peek(), Some((_, '{'))) => {
                chars.next(); // consume {

                if !text.is_empty() {
                    parts.push(StringPart {
                        kind: StringPartKind::Text(std::mem::take(&mut text)),
                        span: Span {
                            start: string_start + segment_start,
                            end: string_start + i,
                        },
                    });
                }

                let expr_start = i + 2;
                let mut depth = 1;
                let mut expr_end = expr_start;

                while let Some((j, c)) = chars.next() {
                    match c {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                expr_end = j;
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if depth != 0 {
                    return None;
                }

                parts.push(StringPart {
                    kind: StringPartKind::Interpolation(rest[expr_start..expr_end].to_string()),
                    span: Span {
                        start: string_start + expr_start,
                        end: string_start + expr_end,
                    },
                });

                segment_start = expr_end + 1;
            }

            other => {
                text.push(other);
            }
        }
    }

    None
}
