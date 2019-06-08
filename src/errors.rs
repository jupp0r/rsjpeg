#[derive(Fail, Debug, PartialEq, Eq)]
#[fail(display = "Parser Error: {}", reason)]
pub struct ParserError {
    pub reason: String,
}
