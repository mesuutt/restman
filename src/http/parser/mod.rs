use std::path::PathBuf;

use nom::bytes::complete::{is_not, take_while};
use nom::bytes::streaming::tag;
use nom::character::complete::{char, crlf, one_of};
use nom::character::{is_alphanumeric, is_space};
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::{delimited, tuple};
use nom::IResult;

mod test;

pub type ParseResult<'a, T> = Result<T, ParseError<'a>>;

const CRLF: &[u8] = b"\r\n";

#[derive(Debug)]
pub enum ParseError<'a> {
    InvalidPath(&'a [u8]),
    ParseError,
}

#[derive(PartialEq, Debug)]
pub struct RequestLine<'a> {
    pub method: &'a [u8],
    pub path: &'a [u8],
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
    Bytes(&'a [u8]),
    File(PathBuf),
    Empty,
}

impl From<&[u8]> for Method {
    fn from(i: &[u8]) -> Self {
        match i {
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

#[derive(PartialEq, Debug)]
pub struct Header<'a> {
    pub name: &'a [u8],
    pub value: &'a [u8],
}

fn request_line(i: &[u8]) -> IResult<&[u8], RequestLine> {
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

fn http_version(i: &[u8]) -> IResult<&[u8], Version> {
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

fn header_line(i: &[u8]) -> IResult<&[u8], Header> {
    let (i, (name, _, _, value, _)) = tuple((
        token,
        tag(":"),
        take_while(is_space),
        take_while(is_header_value_char),
        crlf,
    ))(i)?;

    Ok((i, Header { name, value }))
}

fn parse_headers(i: &[u8]) -> IResult<&[u8], Vec<Header>> {
    let (i, headers) = many0(header_line)(i)?;
    Ok((i, headers))
}

fn request_body(i: &[u8]) -> IResult<&[u8], MessageBody> {
    let (i, body) = block_parser(i, CRLF, CRLF)?;

    if body == b"" {
        return Ok((i, MessageBody::Empty));
    }

    Ok((i, MessageBody::Bytes(body)))
}

fn block_parser<'a>(i: &'a [u8], start: &'a [u8], end: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    delimited(tag(start), is_not(end), tag(end))(i)
}

fn request(i: &[u8]) -> ParseResult<Request> {
    let (i, line) = request_line(i).map_err(|_| ParseError::ParseError)?; // FIXME: fix error handling
    let (i, headers) = parse_headers(i).map_err(|_| ParseError::ParseError)?; // FIXME: fix error handling;
                                                                              //.map_or((i, None), |(x,y)| (i, if y.is_empty() { None } else {Some(y)}));//.map_err(|_| ParseError::ParseError)?; // FIXME:
    let path = std::str::from_utf8(line.path).map_err(|_x| ParseError::InvalidPath(line.path))?;

    let (_i, body) = request_body(i).map_err(|_x| ParseError::ParseError)?; // FIXME: fix error handling // .map_or((i, None), |(x,y)| (i, Some(y)));

    Ok(Request {
        method: Method::from(line.method),
        path: path.to_string(),
        version: line.version,
        headers,
        body,
    })
}

fn print(label: &str, i: &[u8]) {
    println!("{}: {:?}", label, std::str::from_utf8(i));
}

fn is_token_char(i: u8) -> bool {
    is_alphanumeric(i) || b"!#$%&'*+-.^_`|~".contains(&i)
}

fn token(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while(is_token_char)(i)
}

fn is_vchar(i: u8) -> bool {
    i > 32 && i <= 126
}

fn vchar_1(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while(is_vchar)(i)
}

fn sp(i: &[u8]) -> IResult<&[u8], char> {
    char(' ')(i)
}

fn is_header_value_char(i: u8) -> bool {
    i == 9 || (i >= 32 && i <= 126)
}
