use crate::http::parser::Span;

#[derive(Debug, PartialEq)]
pub struct ParseErr<'a> {
    span: Span<'a>,
    message: Option<String>,
}

impl<'a> ParseErr<'a> {
    pub fn new(message: String, span: Span<'a>) -> Self {
        Self {
            span,
            message: Some(message),
        }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn line(&self) -> u32 {
        self.span().location_line()
    }

    pub fn offset(&self) -> usize {
        self.span().location_offset()
    }
}

impl<'a> nom::error::ParseError<Span<'a>> for ParseErr<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::new(format!("parse error {:?}", kind), input)
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self::new(format!("unexpected character '{}'", c), input)
    }

    fn or(self, other: Self) -> Self {
        let message = format!(
            "{}\tOR\n{}\n",
            self.message.unwrap_or("".to_string()),
            other.message.unwrap_or("".to_string())
        );
        Self::new(message, other.span)
    }
}
