use engine::parse::Script;

mod engine;

fn main() {
    let mut s = Script::from_file("script.txt").unwrap();
    while let Some(ctx) = s.next() {
        match ctx {
            Ok(ctx) => {
                println!("{:#?} {:#?}", ctx, s.line_chars());
            }
            Err(error) => {
                println!("{:#?}", error);
            }
        }
    }
}
