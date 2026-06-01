use std::collections::HashSet;

use anyhow::bail;
use logos::Span;

use crate::language::{
    ast::{self},
    lexer::{self, SpannedToken, SyntaxKind, Token},
    utils::merge_span,
};

pub struct Parser<'s> {
    tokens: Vec<SpannedToken<'s>>,
    source: String,
    pos: usize,
}

impl<'s> Parser<'s> {
    pub fn from_tokens(src: &str, tokens: &Vec<SpannedToken<'s>>) -> anyhow::Result<Parser<'s>> {
        Ok(Parser {
            tokens: tokens.clone(),
            source: src.to_string(),
            pos: 0,
        })
    }

    pub fn from_src(src: &str) -> anyhow::Result<Parser<'_>> {
        let tokens = lexer::lex(src)?;

        Ok(Parser {
            tokens: tokens,
            source: src.to_string(),
            pos: 0,
        })
    }

    fn eof_token(&self) -> SpannedToken<'_> {
        SpannedToken {
            token: Token::EOF,
            span: Span {
                start: self.source.len(),
                end: self.source.len(),
            },
        }
    }

    pub fn is_eof(&self, offset: usize) -> bool {
        return (self.pos + offset) >= self.tokens.len();
    }

    pub fn peek(&self, offset: usize) -> SpannedToken<'_> {
        if self.is_eof(offset) {
            return self.eof_token();
        }

        self.tokens[self.pos].clone()
    }

    pub fn next(&mut self) -> SpannedToken<'_> {
        if self.is_eof(0) {
            return self.eof_token();
        }

        let token = self.tokens[self.pos].clone();
        self.pos += 1;

        token
    }

    pub fn expect(&mut self, kind: SyntaxKind) -> anyhow::Result<SpannedToken<'_>> {
        let peek = self.peek(0);
        let peek_span = peek.span;
        let peek_token = peek.token;

        if SyntaxKind::from(peek_token.clone()) == kind {
            Ok(self.next())
        } else {
            bail!("expected {:?} but {:?} ({:?})", kind, peek_token, peek_span)
        }
    }

    fn peek_kind(&mut self, offset: usize) -> SyntaxKind {
        SyntaxKind::from(self.peek(offset))
    }
}

impl<'s> Parser<'s> {
    pub fn parse(&mut self) -> anyhow::Result<ast::SeruProgram> {
        Ok(ast::SeruProgram {
            top_declarations: self.parse_top_decls()?,
        })
    }

    fn parse_top_decls(&mut self) -> anyhow::Result<Vec<ast::TopDeclaration>> {
        let mut decls = Vec::new();

        loop {
            let peek = self.peek(0);
            decls.push(match peek.token.into() {
                SyntaxKind::Const => {
                    ast::TopDeclaration::ConstDeclaration(self.parse_const_decl()?)
                }
                SyntaxKind::Component => {
                    ast::TopDeclaration::ComponentDeclaration(self.parse_component_decl()?)
                }
                SyntaxKind::EOF => break,
                kind => bail!(
                    "expected ConstDeclaration or ComponentDeclaration but {:?} ({:?})",
                    kind,
                    peek.span
                ),
            });
        }

        Ok(decls)
    }

    fn parse_const_decl(&mut self) -> anyhow::Result<ast::ConstDeclaration> {
        self.expect(SyntaxKind::Const)?;

        let name = {
            let Token::Identifier(name) = self.expect(SyntaxKind::Identifier)?.token else {
                unreachable!()
            };

            name.to_string()
        };

        self.expect(SyntaxKind::Assign)?;

        let value = self.parse_expr()?;

        Ok(ast::ConstDeclaration { name, value })
    }

    pub fn parse_expr(&mut self) -> anyhow::Result<ast::Expr> {
        self.parse_expr_bp(0)
    }

    pub fn parse_component_decl(&mut self) -> anyhow::Result<ast::ComponentDeclaration> {
        self.expect(SyntaxKind::Component)?;

        let name = {
            let Token::Identifier(name) = self.expect(SyntaxKind::Identifier)?.token else {
                unreachable!()
            };

            name.to_string()
        };

        let arg_declarations = self.parse_arg_decls()?;

        self.expect(SyntaxKind::Colon)?;

        let body = self.parse_child_node()?;

        Ok(ast::ComponentDeclaration {
            name,
            arg_declarations,
            body,
        })
    }

    fn parse_child_node(&mut self) -> anyhow::Result<ast::ChildNode> {
        match self.peek(0).token {
            Token::If => Ok(ast::ChildNode::If(self.parse_if_node()?)),
            Token::For => Ok(ast::ChildNode::For(self.parse_for_node()?)),
            Token::Identifier(_) => Ok(ast::ChildNode::ComponentCall(self.parse_component_call()?)),
            tok => anyhow::bail!("expect node but {:?}", tok),
        }
    }

    fn parse_if_node(&mut self) -> anyhow::Result<ast::IfBlock> {
        self.expect(SyntaxKind::If)?;

        self.expect(SyntaxKind::LParen)?;

        let condition = self.parse_expr()?;

        self.expect(SyntaxKind::RParen)?;

        let body = self.parse_child_nodes()?;

        Ok(ast::IfBlock { condition, body })
    }

    fn parse_for_node(&mut self) -> anyhow::Result<ast::ForBlock> {
        self.expect(SyntaxKind::For)?;

        self.expect(SyntaxKind::LParen)?;

        let name1 = {
            let Token::Identifier(name) = self.expect(SyntaxKind::Identifier)?.token else {
                unreachable!()
            };

            name.to_string()
        };

        let mut name2 = None;

        if self.peek_kind(0) == SyntaxKind::Comma {
            self.next();

            name2 = {
                let Token::Identifier(name) = self.expect(SyntaxKind::Identifier)?.token else {
                    unreachable!()
                };

                Some(name.to_string())
            };
        }

        self.expect(SyntaxKind::In)?;

        let iterable = self.parse_expr()?;

        self.expect(SyntaxKind::RParen)?;

        let body = self.parse_child_nodes()?;

        Ok(match name2 {
            Some(name2) => {
                // idx, val
                ast::ForBlock {
                    index_name: Some(name1),
                    item_name: name2,
                    body,
                    iterable,
                }
            }
            None => {
                // val
                ast::ForBlock {
                    index_name: None,
                    item_name: name1,
                    body,
                    iterable,
                }
            }
        })
    }

    fn parse_component_call(&mut self) -> anyhow::Result<ast::ComponentCall> {
        let name = {
            let Token::Identifier(name) = self.expect(SyntaxKind::Identifier)?.token else {
                unreachable!()
            };

            name.to_string()
        };

        let args = self.parse_args()?;

        let body = self.parse_child_nodes()?;

        Ok(ast::ComponentCall { name, args, body })
    }

    fn parse_child_nodes(&mut self) -> anyhow::Result<Vec<ast::ChildNode>> {
        let mut body = Vec::new();
        if self.peek_kind(0) == SyntaxKind::LBracket {
            self.next();

            loop {
                if self.peek_kind(0) == SyntaxKind::RBracket {
                    break;
                }

                body.push(self.parse_child_node()?);
            }

            self.expect(SyntaxKind::RBracket)?;
        }

        Ok(body)
    }

    fn parse_args(&mut self) -> anyhow::Result<Vec<ast::Arg>> {
        self.expect(SyntaxKind::LParen)?;

        let mut args = Vec::new();

        if self.peek_kind(0) != SyntaxKind::RParen {
            loop {
                let name = {
                    let Token::Identifier(name) = self.expect(SyntaxKind::Identifier)?.token else {
                        unreachable!()
                    };

                    name.to_string()
                };

                self.expect(SyntaxKind::Assign)?;

                let value = self.parse_expr()?;

                args.push(ast::Arg { name, value });

                if self.peek_kind(0) != SyntaxKind::Comma {
                    break;
                }

                self.next();

                if self.peek_kind(0) == SyntaxKind::RParen {
                    break;
                }
            }
        }

        self.expect(SyntaxKind::RParen)?;

        Ok(args)
    }

    fn parse_arg_decl(&mut self) -> anyhow::Result<ast::ArgDeclaration> {
        let (name, name_span) = {
            let spanned = self.expect(SyntaxKind::Identifier)?;

            let Token::Identifier(name) = spanned.token else {
                unreachable!();
            };

            (name.to_string(), spanned.span)
        };

        let mut default_expr = None;

        // default
        if self.peek_kind(0) == SyntaxKind::Assign {
            self.next();

            default_expr = Some(self.parse_expr()?);
        }

        let span = match default_expr.as_ref() {
            Some(expr) => merge_span(&name_span, &expr.span),
            None => name_span,
        };

        Ok(ast::ArgDeclaration {
            name: name.to_string(),
            default: default_expr.map(Box::new),
            span,
        })
    }

    fn parse_arg_decls(&mut self) -> anyhow::Result<Vec<ast::ArgDeclaration>> {
        self.expect(SyntaxKind::LParen)?;

        let mut args = Vec::new();
        let mut registered_args: HashSet<String> = HashSet::new();

        if self.peek_kind(0) != SyntaxKind::RParen {
            loop {
                let arg = self.parse_arg_decl()?;

                if !registered_args.insert(arg.name.clone()) {
                    anyhow::bail!("already decleared arg: {} ({:?})", arg.name, arg.span)
                }

                args.push(arg);

                if self.peek_kind(0) == SyntaxKind::Comma {
                    self.next();

                    if self.peek_kind(0) == SyntaxKind::RParen {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        self.expect(SyntaxKind::RParen)?.span;

        Ok(args)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> anyhow::Result<ast::Expr> {
        let mut lhs = match self.peek_kind(0) {
            SyntaxKind::Plus | SyntaxKind::Minus => {
                let op = match self.peek_kind(0) {
                    SyntaxKind::Plus => ast::UnaryOp::Plus,
                    SyntaxKind::Minus => ast::UnaryOp::Minus,
                    _ => unreachable!(),
                };

                let op_span = self.next().span;
                let ((), r_bp) = prefix_binding_power(&op);
                let rhs = self.parse_expr_bp(r_bp)?;
                let span = merge_span(&op_span, &rhs.span);

                ast::Expr {
                    kind: ast::ExprKind::Unary {
                        op,
                        expr: Box::new(rhs),
                    },
                    span,
                }
            }
            _ => self.parse_atom()?,
        };

        loop {
            match self.peek_kind(0) {
                SyntaxKind::LParen => {
                    lhs = self.parse_call(lhs)?;
                    continue;
                }

                SyntaxKind::LBracket => {
                    lhs = self.parse_index(lhs)?;
                    continue;
                }

                _ => {}
            }

            let op = match self.peek_kind(0) {
                SyntaxKind::Plus => ast::BinOp::Plus,
                SyntaxKind::Minus => ast::BinOp::Minus,
                SyntaxKind::Multiple => ast::BinOp::Multiply,
                SyntaxKind::Divide => ast::BinOp::Divide,
                SyntaxKind::Eq => ast::BinOp::Eq,
                SyntaxKind::Neq => ast::BinOp::Neq,
                _ => break,
            };

            let (l_bp, r_bp) = infix_binding_power(&op);
            if l_bp < min_bp {
                break;
            }

            self.next();

            let rhs = self.parse_expr_bp(r_bp)?;
            let span = merge_span(&lhs.span, &rhs.span);
            lhs = ast::Expr {
                kind: ast::ExprKind::Bin {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                },
                span,
            }
        }

        Ok(lhs)
    }

    pub fn parse_call(&mut self, callee: ast::Expr) -> anyhow::Result<ast::Expr> {
        self.expect(SyntaxKind::LParen)?.span;

        let mut args = Vec::new();

        if self.peek_kind(0) != SyntaxKind::RParen {
            loop {
                args.push(self.parse_expr_bp(0)?);

                if self.peek_kind(0) != SyntaxKind::Comma {
                    break;
                }

                self.next();

                if self.peek_kind(0) == SyntaxKind::RParen {
                    break;
                }
            }
        }

        let end_span = self.expect(SyntaxKind::RParen)?.span;
        let span = merge_span(&callee.span, &end_span);

        Ok(ast::Expr {
            kind: ast::ExprKind::Call {
                callee: Box::new(callee),
                args,
            },
            span,
        })
    }

    pub fn parse_index(&mut self, target: ast::Expr) -> anyhow::Result<ast::Expr> {
        self.expect(SyntaxKind::LBracket)?.span;

        let index = self.parse_expr_bp(0)?;

        let end_span = self.expect(SyntaxKind::RBracket)?.span;

        let span = merge_span(&target.span, &end_span);

        Ok(ast::Expr {
            kind: ast::ExprKind::Index {
                target: Box::new(target),
                index: Box::new(index),
            },
            span,
        })
    }

    pub fn parse_atom(&mut self) -> anyhow::Result<ast::Expr> {
        let peek = self.peek(0);
        match peek.clone().into() {
            SyntaxKind::LParen => {
                self.next();

                let expr = self.parse_expr_bp(0)?;

                self.expect(SyntaxKind::RParen)?;

                return Ok(expr);
            }

            SyntaxKind::Identifier => {
                let spanned = self.next();

                let Token::Identifier(name) = spanned.token else {
                    unreachable!();
                };

                return Ok(ast::Expr {
                    kind: ast::ExprKind::Ident(name.to_string()),
                    span: spanned.span,
                });
            }

            SyntaxKind::True | SyntaxKind::False => {
                let spanned = self.next();
                let span = spanned.span;

                Ok(ast::Expr {
                    kind: ast::ExprKind::Bool(SyntaxKind::from(spanned.token) == SyntaxKind::True),
                    span,
                })
            }

            SyntaxKind::Null => {
                let span = self.next().span;

                Ok(ast::Expr {
                    kind: ast::ExprKind::Null,
                    span,
                })
            }

            SyntaxKind::String => {
                let src = self.source.clone();

                let (parts, span) = {
                    let spanned = self.next();

                    let Token::String(parts) = &spanned.token else {
                        unreachable!();
                    };

                    (parts.clone(), spanned.span)
                };

                let mut ast_parts = Vec::new();

                for part in parts {
                    ast_parts.push(ast::ParsedStringPart {
                        kind: match &part.kind {
                            lexer::StringPartKind::Text(text) => {
                                ast::ParsedStringPartKind::Text(text.to_string())
                            }
                            lexer::StringPartKind::Interpolation(raw_expr) => {
                                let tokens = lexer::lex_with_offset(raw_expr, part.span.start)?;

                                if tokens.len() <= 0 {
                                    anyhow::bail!("empty string interpolation {:?}", part.span)
                                }

                                let mut parser = Parser::from_tokens(&src, &tokens)?;
                                ast::ParsedStringPartKind::Interpolation(Box::new(
                                    parser.parse_expr()?,
                                ))
                            }
                        },
                        span: part.span.clone(),
                    });
                }

                Ok(ast::Expr {
                    kind: ast::ExprKind::StringInterpolation(ast_parts),
                    span,
                })
            }

            SyntaxKind::Int | SyntaxKind::Float => {
                let spanned = self.next();
                let val = match spanned.token {
                    Token::Int(num) => num,
                    Token::Float(num) => num,
                    _ => unreachable!(),
                };

                Ok(ast::Expr {
                    kind: ast::ExprKind::Number(val.parse()?),
                    span: spanned.span,
                })
            }

            SyntaxKind::LBracket => {
                let start_span = self.next().span;

                let mut values = Vec::new();
                if self.peek_kind(0) != SyntaxKind::RBracket {
                    loop {
                        values.push(self.parse_expr()?);

                        if self.peek_kind(0) != SyntaxKind::Comma {
                            break;
                        }

                        self.next();

                        if self.peek_kind(0) == SyntaxKind::RBracket {
                            break;
                        }
                    }
                }

                let end_span = self.expect(SyntaxKind::RBracket)?.span;

                Ok(ast::Expr {
                    kind: ast::ExprKind::Array(values),
                    span: merge_span(&start_span, &end_span),
                })
            }

            kind => anyhow::bail!("expected atom but {:?} {:?}", kind, peek.span),
        }
    }
}

fn infix_binding_power(op: &ast::BinOp) -> (u8, u8) {
    match op {
        ast::BinOp::Eq | ast::BinOp::Neq => (1, 2),
        ast::BinOp::Plus | ast::BinOp::Minus => (3, 4),
        ast::BinOp::Multiply | ast::BinOp::Divide => (5, 6),
    }
}

fn prefix_binding_power(op: &ast::UnaryOp) -> ((), u8) {
    match op {
        ast::UnaryOp::Plus | ast::UnaryOp::Minus => ((), 8),
        ast::UnaryOp::Not => ((), 10),
    }
}
