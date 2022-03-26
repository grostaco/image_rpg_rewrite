use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_until, take_while, take_while1},
    character::{complete, is_alphabetic},
    error::{ParseError, VerboseError, VerboseErrorKind},
    sequence::{delimited, terminated, tuple},
    IResult, Offset,
};

use super::{
    directive::{DirectiveTrait, Jump},
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
}

pub struct Script {
    ctx: String,
    cur: usize,
    cache: Option<Context>,
}

pub struct ScriptIter<'s> {
    cur: &'s str,
    ctx: &'s str,
}

impl<'s> Iterator for ScriptIter<'s> {
    type Item = Result<Context, nom::Err<nom::error::VerboseError<&'s str>>>;

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
            cache: None,
        })
    }

    pub fn chars(&self) -> usize {
        self.cur
    }

    pub fn line_chars(&self) -> (usize, usize) {
        line_chars(&self.ctx, self.ctx.get(self.cur..).unwrap())
    }

    pub fn next<'s>(
        &'s mut self,
    ) -> Option<Result<Context, nom::Err<nom::error::VerboseError<&'s str>>>> {
        let cur = self.ctx.get(self.cur..).unwrap().trim();
        self.cache = None;

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

    pub fn peek<'s>(
        &'s self,
    ) -> Option<Result<Context, nom::Err<nom::error::VerboseError<&'s str>>>> {
        let cur = self.ctx.get(self.cur..).unwrap().trim();
        match cur {
            ctx if Dialogue::hint(ctx) => match Dialogue::parse(ctx) {
                Ok((_, o)) => Some(Ok(Context::Dialogue(o))),
                Err(e) => Some(Err(e)),
            },
            ctx if Directive::hint(ctx) => match Directive::parse(cur) {
                Ok((_, o)) => Some(Ok(Context::Directive(o))),
                Err(e) => Some(Err(e)),
            },
            ctx if ctx.starts_with('#') => Some(Ok(Context::Comment)),
            _ => None,
        }
    }

    pub fn iter<'s>(&'s self) -> ScriptIter<'s> {
        ScriptIter {
            ctx: self.ctx.as_str(),
            cur: self.ctx.as_str(),
        }
    }
}

impl<'s> ScriptIter<'s> {
    pub fn new(ctx: &'s str) -> Self {
        Self { ctx, cur: ctx }
    }
    pub fn peek(&self) -> Option<Result<Context, nom::Err<nom::error::VerboseError<&'s str>>>> {
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

    fn parse(ctx: &str) -> IResult<&str, Self, VerboseError<&str>> {
        let name = alt((
            delimited(
                complete::char('['),
                take_while(|c| c != '\n' && c != '\r' && c != ']'),
                complete::char(']'),
            ),
            failure_case(take_while(|c| c != '\n' && c != ']'), |_, output| {
                nom::error::VerboseError::from_char(output, ']')
            }),
        ));

        let dialogue = alt((take_until("\n@"), take_until("\n#"), take_while(|_| true)));

        let (input, (name, dialogue)) = match tuple((name, dialogue))(ctx) {
            Ok(o) => o,
            Err(e) => return Err(e),
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

    fn parse(ctx: &str) -> IResult<&str, Self, VerboseError<&str>> {
        let directive = alt((
            delimited(
                complete::char('@'),
                take_while1(|c| is_alphabetic(c as u8)),
                complete::char('('),
            ),
            failure_case(tag("@("), |_, output| nom::error::VerboseError {
                errors: vec![(
                    output,
                    VerboseErrorKind::Context("Expected attribute name, found nothing"),
                )],
            }),
        ));
        let args = alt((
            terminated(take_while1(|c| c != ')' && c != '\n'), tag(")")),
            failure_case(take_till1(|c| c == ')' || c == '\n'), |_, output| {
                nom::error::VerboseError {
                    errors: vec![(
                        output,
                        VerboseErrorKind::Context("Expected matching parentheses )"),
                    )],
                }
            }),
        ));

        let (input, (directive, args)) = tuple((directive, args))(ctx)?;
        let args = args.split(',').map(str::trim).collect::<Vec<&str>>();

        // name: directive.to_string(),
        // args: args.split(',').map(|s| s.trim().to_string()).collect(),
        Ok((
            input,
            Directive::Jump(Jump::parse(directive, &args).unwrap().unwrap()),
        ))
    }
}
