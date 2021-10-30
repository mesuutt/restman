use std::path::PathBuf;

use nom::bytes::complete::{is_not, tag, take_while};
use nom::character::complete::{char, crlf, one_of};
use nom::character::is_alphanumeric;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::{delimited, tuple};
use nom_locate::LocatedSpan;

mod test;

pub type Span<'a> = LocatedSpan<&'a str>;

pub type IResult<'a, O> = nom::IResult<Span<'a>, O>;

pub type ParseResult<'a, T> = Result<T, ParseError<'a>>;

const CRLF: &str = "\r\n";

#[derive(Debug)]
pub enum ParseError<'a> {
    InvalidPath(&'a [u8]),
    ParseError,
}

#[derive(PartialEq, Debug)]
pub struct RequestLine<'a> {
    pub method: Span<'a>,
    pub path: Span<'a>,
    pub version: Version,
}

#[derive(PartialEq, Debug)]
pub struct Request<'a> {
    method: Method,
    path: String,
    version: Version,
    headers: Vec<Header<'a>>,
    body: MessageBody<'a>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Method {
    Get,
    Post,
    Head,
    Options,
    Put,
    Delete,
    Custom(String),
}

#[derive(PartialEq, Debug)]
pub enum MessageBody<'a> {
    Bytes(Span<'a>),
    File(PathBuf),
    Empty,
}

impl From<Span<'_>> for Method {
    fn from(i: Span) -> Self {
        match i.fragment().as_bytes() {
            b"GET" => Method::Get,
            b"POST" => Method::Post,
            b"HEAD" => Method::Head,
            b"OPTIONS" => Method::Options,
            b"PUT" => Method::Put,
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

fn request_line(i: Span) -> IResult<RequestLine> {
    let (i, method) = token(i)?;
    let (i, _) = sp(i)?;
    let (i, path) = vchar_1(i)?; // TODO: handle all valid urls, read rfc
    let (i, _) = take_while(is_space)(i)?;
    let (i, version) = http_version(i)?;
    let (i, _) = crlf(i)?;

    Ok((
        i,
        RequestLine {
            method,
            version,
            path,
        },
    ))
}

fn http_version(i: Span) -> IResult<Version> {
    let (i, t) = opt(tag("HTTP/1."))(i)?;
    if t.is_none() {
        return Ok((i, Version::V11));
    }

    let (i, minor) = one_of("01")(i)?;

    Ok((
        i,
        if minor == '0' {
            Version::V10
        } else {
            Version::V11
        },
    ))
}

fn header_line(i: Span) -> IResult<Header> {
    let (i, (name, _, _, value, _)) = tuple((
        token,
        tag(":"),
        take_while(is_space),
        take_while(is_header_value_char),
        crlf,
    ))(i)?;

    Ok((i, Header { name, value }))
}

fn parse_headers(i: Span) -> IResult<Vec<Header>> {
    let (i, headers) = many0(header_line)(i)?;
    Ok((i, headers))
}

fn request_body(i: Span) -> IResult<MessageBody> {
    // let (i, body) = block_parser(i, Span::new(CRLF), Span::new(CRLF))?;
    let (i, body) = delimited(tag(CRLF), is_not(CRLF), tag(CRLF))(i)?;

    if body.fragment() == &"" {
        return Ok((i, MessageBody::Empty));
    }

    Ok((i, MessageBody::Bytes(body)))
}

/*
fn block_parser(i: Span, start: Span, end: Span) -> IResult<Span> {
    delimited(tag(start), is_not(end), tag(end))(i)
}
*/

fn request(i: Span) -> ParseResult<Request> {
    let (i, line) = request_line(i).map_err(|_| ParseError::ParseError)?; // FIXME: fix error handling
    let (i, headers) = parse_headers(i).map_err(|_| ParseError::ParseError)?; // FIXME: fix error handling;
                                                                              //.map_or((i, None), |(x,y)| (i, if y.is_empty() { None } else {Some(y)}));//.map_err(|_| ParseError::ParseError)?; // FIXME:

    let (_i, body) = request_body(i).map_err(|_x| ParseError::ParseError)?; // FIXME: fix error handling // .map_or((i, None), |(x,y)| (i, Some(y)));

    Ok(Request {
        method: Method::from(line.method),
        path: line.path.fragment().to_string(),
        version: line.version,
        headers,
        body,
    })
}

fn print(label: &str, i: &[u8]) {
    println!("{}: {:?}", label, std::str::from_utf8(i));
}

fn is_token_char(i: char) -> bool {
    is_alphanumeric(i as u8) || "!#$%&'*+-.^_`|~".contains(i)
}

fn token(i: Span) -> IResult<Span> {
    take_while(is_token_char)(i)
}

fn is_vchar(i: char) -> bool {
    // c.is_alphabetic()
    i as u32 > 32 && i as u32 <= 126
}

fn vchar_1(i: Span) -> IResult<Span> {
    take_while(is_vchar)(i)
}

fn is_space(x: char) -> bool {
    x == ' '
}

fn sp(i: Span) -> IResult<char> {
    char(' ')(i)
}

fn is_header_value_char(i: char) -> bool {
    /*let i = match i.to_digit(10) {
        None => return false,
        Some(x) => x,
    };
    */
    let i = i as u32;

    i == 9 || (i >= 32 && i <= 126)
}
