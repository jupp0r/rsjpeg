#[derive(Fail, Debug)]
#[fail(display = "Parser Error: {}", reason)]
pub struct ParserError {
    pub reason: String,
}