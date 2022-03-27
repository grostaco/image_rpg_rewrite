use std::{collections::HashMap, iter::Peekable};

use image::DynamicImage;

use super::{
    directive::DirectiveTrait,
    error::EngineError,
    parse::{Context, Directive, Script},
};

pub struct Engine {
    pub script: Peekable<Script>,
    pub bg_path: Option<String>,
    pub bgs_cache: HashMap<String, DynamicImage>,
    pub sprites: Vec<Sprite>,
    pub sprites_cache: HashMap<String, DynamicImage>,
}

pub struct Sprite {
    x: usize,
    y: usize,
    scale: f64,
    sprite: String,
    visible: bool,
}

impl Engine {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            script: Script::from_file(path)?.peekable(),
            bg_path: None,
            bgs_cache: HashMap::new(),
            sprites: Vec::new(),
            sprites_cache: HashMap::new(),
        })
    }

    pub fn next(&mut self) -> Option<Result<Context, EngineError>> {
        match self.script.next() {
            Some(ctx) => match ctx {
                Ok(ctx) => match ctx {
                    Context::Dialogue(dialogue) => Some(Ok(Context::Dialogue(dialogue))),
                    Context::Directive(directive) => {
                        match &directive {
                            Directive::Jump(j) => j.exec(self),
                            Directive::LoadBG(bg) => bg.exec(self),
                        };
                        Some(Ok(Context::Directive(directive)))
                    }
                    Context::Comment => Some(Ok(Context::Comment)),
                },
                Err(error) => Some(Err(error)),
            },
            None => None,
        }
    }

    pub fn peek(&mut self) -> Option<&Result<Context, EngineError>> {
        self.script.peek()
    }
}
