use nom::bytes::complete::take_while;
use nom::bytes::streaming::tag;
use nom::character::{is_alphanumeric, is_space};
use nom::character::complete::{char, crlf, one_of};
use nom::IResult;
use nom::sequence::tuple;

mod test;

#[derive(PartialEq, Debug)]
pub struct RequestLine<'a> {
    pub method: &'a [u8],
    pub path: &'a [u8],
    pub version: Version,
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
    let (i, _) = sp(i)?;
    let (i, version) = http_version(i)?;
    let (i, _) = crlf(i)?;

    Ok((i, RequestLine {
        method,
        version,
        path,
    }))
}

fn http_version(i: &[u8]) -> IResult<&[u8], Version> {
    let (i, _) = tag("HTTP/1.")(i)?;
    let (i, minor) = one_of("01")(i)?;

    Ok((i, if minor == '0' {
        Version::V10
    } else {
        Version::V11
    }))
}

fn header_line(i: &[u8]) -> IResult<&[u8], Header> {
    let (i, (name, _, _, value, _)) =
        tuple((token, tag(":"), take_while(is_space), take_while(is_header_value_char), crlf))(i)?;

    Ok((i, Header {
        name,
        value,
    }))
}


fn is_token_char(i: u8) -> bool {
    is_alphanumeric(i) ||
        b"!#$%&'*+-.^_`|~".contains(&i)
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


