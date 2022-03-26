use super::{
    directive::{DirectiveTrait, Jump},
    parse::{Context, Directive, Script, ScriptIter},
};

pub struct Engine {
    pub script: Script,
}

impl Engine {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            script: Script::from_file(path)?,
        })
    }

    pub fn next<'s>(
        &'s mut self,
    ) -> Option<Result<Context, nom::Err<nom::error::VerboseError<&'s str>>>> {
        match self.script.peek() {
            Some(ctx) => match ctx {
                Ok(ctx) => match ctx {
                    Context::Dialogue(dialogue) => Some(Ok(Context::Dialogue(dialogue))),
                    Context::Directive(directive) => {
                        match &directive {
                            Directive::Jump(j) => {}
                        };
                        Some(Ok(Context::Directive(directive)))
                    }
                    Context::Comment => Some(Ok(Context::Comment)),
                },
                Err(error) => Some(Err(error)),
            },
            None => None,
        }

        // let ctx = self.script.peek();

        // if let Some(Ok(Context::Directive(ref directive))) = ctx {
        //     match directive {
        //         Directive::Jump(j) => j.exec(self),
        //     }
        // }

        // self.script.next()
    }

    pub fn peek(&self) -> Option<Result<Context, nom::Err<nom::error::VerboseError<&str>>>> {
        self.script.peek()
    }
}
