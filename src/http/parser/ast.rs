use crate::http::parser::Span;

#[derive(PartialEq, Debug)]
pub struct Request<'a> {
    pub method: Method,
    pub path: String,
    pub version: Version,
    pub headers: Vec<Header<'a>>,
    pub body: MessageBody<'a>,
    pub title: String,
    pub script: String,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Method {
    Get,
    Post,
    Head,
    Options,
    Put,
    Patch,
    Delete,
    Custom(String),
}

#[derive(PartialEq, Debug)]
pub enum MessageBody<'a> {
    Bytes(Span<'a>),
    Empty,
    File(Span<'a>),
}

impl From<Span<'_>> for Method {
    fn from(i: Span) -> Self {
        match i.fragment().as_bytes() {
            b"GET" => Method::Get,
            b"POST" => Method::Post,
            b"HEAD" => Method::Head,
            b"OPTIONS" => Method::Options,
            b"PUT" => Method::Put,
            b"PATCH" => Method::Patch,
            b"DELETE" => Method::Delete,
            x => Method::Custom(String::from_utf8_lossy(x).to_string()),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Version {
    V10,
    V11,
}

#[derive(Debug)]
pub struct Header<'a> {
    pub name: Span<'a>,
    pub value: Span<'a>,
}

impl<'a> PartialEq for Header<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name.fragment() == other.name.fragment()
            && self.value.fragment() == other.value.fragment()
    }

    fn ne(&self, other: &Self) -> bool {
        self.name.fragment() != other.name.fragment()
            || self.value.fragment() != other.value.fragment()
    }
}
