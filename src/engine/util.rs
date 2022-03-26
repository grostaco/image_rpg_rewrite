use nom::{error::ParseError, Err::*, IResult, Offset, Parser};

// Improvised from https://github.com/eignnx/affix-grammar/blob/master/libaffix/src/parser.rs
pub fn failure_case<I, O, E: ParseError<I>, F, G>(
    mut parser: F,
    err_constructor: G,
) -> impl FnMut(I) -> IResult<I, O, E>
where
    F: Parser<I, O, E>,
    G: Fn(I, O) -> E,
{
    move |input: I| match parser.parse(input) {
        Ok((remainder, output)) => {
            let err = err_constructor(remainder, output);
            Err(Failure(err))
        }
        Err(Failure(e)) => Err(Failure(e)),
        Err(Error(e)) => Err(Error(e)),
        Err(Incomplete(need)) => Err(Incomplete(need)),
    }
}

#[inline]
pub fn line_chars(start: &str, mid: &str) -> (usize, usize) {
    let slice = start.get(..start.offset(mid)).unwrap();
    let lines = slice.chars().map(|c| (c == '\n') as usize).sum::<usize>() + 1;

    let chars = mid
        .chars()
        .position(|c| c == '\r' || c == '\n')
        .unwrap_or(mid.len().max(1) - 1)
        + 1;
    //println!("{chars}\n{:#?}", mid);

    (lines, chars)
}
