use super::{core::Engine, error::DirectiveError, parse::Script};

macro_rules! get {
    ($args: expr, $pos: expr, $name: expr) => {
        match $args.get($pos) {
            Some(arg) => arg,
            None => return Some(Err(DirectiveError::MissingArgument($pos, $name))),
        }
    };
}

pub trait DirectiveTrait {
    fn parse(name: &str, args: &[&str]) -> Option<Result<Self, DirectiveError>>
    where
        Self: Sized;
    fn exec(&self, engine: &mut Engine);
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Jump {
    pub choices: Option<(String, String)>,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LoadBG {
    pub bg: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sprite {
    pub bg: String,
}

impl DirectiveTrait for Jump {
    fn parse(name: &str, args: &[&str]) -> Option<Result<Self, DirectiveError>>
    where
        Self: Sized,
    {
        if name != "jump" {
            return None;
        }

        Some(match args {
            [endpoint] => Ok(Self {
                choices: None,
                path: endpoint.to_string(),
            }),
            [choice_a, choice_b, endpoint] => Ok(Self {
                choices: Some((choice_a.to_string(), choice_b.to_string())),
                path: endpoint.to_string(),
            }),
            _ => Err(DirectiveError::Custom(
                "jump directive take either 1 or 3 arguments",
            )),
        })
    }

    fn exec(&self, engine: &mut Engine) {
        engine.script = Script::from_file(&self.path).unwrap().peekable();
    }
}

impl DirectiveTrait for LoadBG {
    fn parse(name: &str, args: &[&str]) -> Option<Result<Self, DirectiveError>>
    where
        Self: Sized,
    {
        if name != "loadbg" {
            return None;
        }

        let bg_path = get!(args, 0, "bg path");
        Some(Ok(Self {
            bg: bg_path.to_string(),
        }))
    }

    fn exec(&self, engine: &mut Engine) {
        engine.bg_path = Some(self.bg.to_string());
    }
}
