use engine::{
    error::{DirectiveError, EngineError},
    parse::Script,
    util::line_chars,
};

mod engine;

fn main() {
    let mut s = Script::from_file("script.txt").unwrap();
    while let Some(ctx) = s.next() {
        match ctx {
            Ok(ctx) => {
                println!("{:#?}", ctx);
            }
            Err(error) => {
                match error {
                    EngineError::Directive(d) => match d {
                        DirectiveError::UnknownDirective(_) => {
                            println!(
                                "{:#?}",
                                line_chars(
                                    s.ctx(),
                                    s.cur().get(..s.cur().find('(').unwrap()).unwrap()
                                ),
                            );
                        }
                        d => println!("{}", d),
                    },
                    EngineError::Incomplete(n) => println!("{:#?}", n),
                    EngineError::Nom(nom) => println!("{}", nom),
                };
                break;
            }
        }
    }
}
