use std::iter::Peekable;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_until, take_until1, take_while, take_while1},
    character::{complete, is_alphabetic},
    error::{convert_error, ParseError, VerboseErrorKind},
    sequence::{delimited, terminated, tuple},
    Err::{Error, Failure, Incomplete},
    Offset,
};

use super::{
    directive::{DirectiveTrait, Jump, LoadBG},
    error::{DirectiveError, EngineError},
    util::{failure_case, line_chars},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Context {
    Dialogue(Dialogue),
    Directive(Directive),
    Comment,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dialogue {
    pub name: String,
    pub dialogue: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Directive {
    Jump(Jump),
    LoadBG(LoadBG),
}

pub struct Script {
    ctx: String,
    cur: usize,
}

pub struct ScriptIter<'s> {
    cur: &'s str,
    ctx: &'s str,
}

impl<'s> Iterator for ScriptIter<'s> {
    type Item = Result<Context, EngineError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.cur = self.cur.trim();
        match self.cur {
            ctx if Dialogue::hint(ctx) => match Dialogue::parse(ctx) {
                Ok((r, o)) => {
                    self.cur = r;
                    Some(Ok(Context::Dialogue(o)))
                }
                Err(e) => Some(Err(e)),
            },
            ctx if Directive::hint(ctx) => match Directive::parse(self.cur) {
                Ok((r, o)) => {
                    self.cur = r;
                    Some(Ok(Context::Directive(o)))
                }
                Err(e) => Some(Err(e)),
            },
            ctx if ctx.starts_with('#') => {
                self.cur = &ctx[ctx.find('\n').unwrap_or(ctx.len()) - 1..];
                Some(Ok(Context::Comment))
            }
            _ => None,
        }
    }
}

impl Script {
    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            ctx: std::fs::read_to_string(path)?,
            cur: 0,
        })
    }

    pub fn chars(&self) -> usize {
        self.cur
    }

    pub fn ctx(&self) -> &str {
        &self.ctx
    }

    pub fn cur(&self) -> &str {
        self.ctx.get(self.cur..).unwrap()
    }

    pub fn line_chars(&self) -> (usize, usize) {
        line_chars(&self.ctx, self.ctx.get(self.cur..).unwrap())
    }

    // pub fn next(&mut self) -> Option<Result<Context, EngineError>> {
    //     let cur = self.ctx.get(self.cur..).unwrap().trim();
    //     self.cache = None;

    //     match cur {
    //         ctx if Dialogue::hint(ctx) => match Dialogue::parse(ctx) {
    //             Ok((r, o)) => {
    //                 self.cur = self.ctx.offset(r);
    //                 Some(Ok(Context::Dialogue(o)))
    //             }
    //             Err(e) => Some(Err(e)),
    //         },
    //         ctx if Directive::hint(ctx) => match Directive::parse(cur) {
    //             Ok((r, o)) => {
    //                 self.cur = self.ctx.offset(r);
    //                 Some(Ok(Context::Directive(o)))
    //             }
    //             Err(e) => Some(Err(e)),
    //         },
    //         ctx if ctx.starts_with('#') => {
    //             self.cur += ctx.find('\n').unwrap_or(ctx.len() - 1) + 1;
    //             Some(Ok(Context::Comment))
    //         }
    //         _ => None,
    //     }
    // }

    // pub fn peek(&self) -> Option<Result<Context, EngineError>> {
    //     let cur = self.ctx.get(self.cur..).unwrap().trim();
    //     match cur {
    //         ctx if Dialogue::hint(ctx) => match Dialogue::parse(ctx) {
    //             Ok((_, o)) => Some(Ok(Context::Dialogue(o))),
    //             Err(e) => Some(Err(e)),
    //         },
    //         ctx if Directive::hint(ctx) => match Directive::parse(cur) {
    //             Ok((_, o)) => Some(Ok(Context::Directive(o))),
    //             Err(e) => Some(Err(e)),
    //         },
    //         ctx if ctx.starts_with('#') => Some(Ok(Context::Comment)),
    //         _ => None,
    //     }
    // }

    pub fn iter<'s>(&'s self) -> ScriptIter<'s> {
        ScriptIter {
            ctx: self.ctx.as_str(),
            cur: self.ctx.as_str(),
        }
    }
}

impl Iterator for Script {
    type Item = Result<Context, EngineError>;
    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.ctx.get(self.cur..).unwrap().trim();

        match cur {
            ctx if Dialogue::hint(ctx) => match Dialogue::parse(ctx) {
                Ok((r, o)) => {
                    self.cur = self.ctx.offset(r);
                    Some(Ok(Context::Dialogue(o)))
                }
                Err(e) => Some(Err(e)),
            },
            ctx if Directive::hint(ctx) => match Directive::parse(cur) {
                Ok((r, o)) => {
                    self.cur = self.ctx.offset(r);
                    Some(Ok(Context::Directive(o)))
                }
                Err(e) => Some(Err(e)),
            },
            ctx if ctx.starts_with('#') => {
                self.cur += ctx.find('\n').unwrap_or(ctx.len() - 1) + 1;
                Some(Ok(Context::Comment))
            }
            _ => None,
        }
    }
}

impl<'s> ScriptIter<'s> {
    pub fn new(ctx: &'s str) -> Self {
        Self { ctx, cur: ctx }
    }
    pub fn peek(&self) -> Option<Result<Context, EngineError>> {
        let mut peek = Self {
            ctx: self.ctx,
            cur: self.cur,
        };
        peek.next()
    }

    pub fn line_chars(&self) -> (usize, usize) {
        let slice = self.ctx.get(..self.ctx.offset(self.cur)).unwrap();
        let lines = slice.chars().map(|c| (c == '\n') as usize).sum::<usize>() + 1;

        let chars = self
            .cur
            .chars()
            .position(|c| c == '\r' || c == '\n')
            .unwrap_or(self.cur.len() - 1)
            + 1;
        println!("{chars}\n{:#?}", self.cur);

        (lines, chars)
    }
}

impl Dialogue {
    #[inline]
    fn hint(ctx: &str) -> bool {
        ctx.starts_with('[')
    }

    fn parse(ctx: &str) -> Result<(&str, Self), EngineError> {
        let name = alt((
            delimited(
                complete::char('['),
                take_while(|c| c != '\n' && c != '\r' && c != ']'),
                complete::char(']'),
            ),
            failure_case(
                take_while(|c| c != '\r' && c != '\n' && c != ']'),
                |input, _| nom::error::VerboseError::from_char(input, ']'),
            ),
        ));

        let dialogue = alt((take_until("\n@"), take_until("\n#"), take_while(|_| true)));

        let (input, (name, dialogue)) = match tuple((name, dialogue))(ctx) {
            Ok(o) => o,
            Err(Incomplete(e)) => return Err(e.into()),
            Err(Error(e)) => return Err(convert_error(ctx, e).into()),
            Err(Failure(e)) => return Err(convert_error(ctx, e).into()),
        };

        Ok((
            input,
            Self {
                name: name.trim().to_string(),
                dialogue: dialogue
                    .split_whitespace()
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
        ))
    }
}

impl Directive {
    #[inline]
    fn hint(ctx: &str) -> bool {
        ctx.starts_with('@')
    }

    fn parse(ctx: &str) -> Result<(&str, Self), EngineError> {
        let directive = alt((
            delimited(
                complete::char('@'),
                take_while1(|c| is_alphabetic(c as u8)),
                complete::char('('),
            ),
            failure_case(take_until1("("), |input, output| nom::error::VerboseError {
                errors: vec![(
                    input,
                    VerboseErrorKind::Context(
                        "directive parse: expected attribute name, found nothing",
                    ),
                )],
            }),
            failure_case(
                delimited(
                    complete::char('@'),
                    take_while(|c| c != '(' && c != ')'),
                    complete::char(')'),
                ),
                |_, output| nom::error::VerboseError {
                    errors: vec![(
                        output,
                        VerboseErrorKind::Context(
                            "directive parse: expected matching parentheses (",
                        ),
                    )],
                },
            ),
        ));

        let args = alt((
            terminated(take_while1(|c| c != ')' && c != '\n'), tag(")")),
            failure_case(take_till1(|c| c == '\r' || c == '\n'), |input: &str, _| {
                nom::error::VerboseError {
                    errors: vec![(
                        input,
                        VerboseErrorKind::Context(
                            "directive parsing: expected matching parentheses )",
                        ),
                    )],
                }
            }),
        ));

        let (input, (directive, args)) = match tuple((directive, args))(ctx) {
            Ok(o) => o,
            Err(Incomplete(e)) => return Err(e.into()),
            Err(Error(e)) => return Err(convert_error(ctx, e).into()),
            Err(Failure(e)) => return Err(convert_error(ctx, e).into()),
        };
        let args = args.split(',').map(str::trim).collect::<Vec<&str>>();

        let j = Jump::parse(directive, &args).transpose()?;
        let bg = LoadBG::parse(directive, &args).transpose()?;
        let directive = j
            .map(Directive::Jump)
            .or_else(|| bg.map(Directive::LoadBG))
            .ok_or_else(|| DirectiveError::UnknownDirective(directive.to_string()))?;

        Ok((input, directive))
    }
}
