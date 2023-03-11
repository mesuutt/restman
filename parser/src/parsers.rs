use std::str::SplitN;
use crate::ast::{Header, MessageBody, Method, Request, Version, ScriptHandler};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while, take_until1, is_not, take_while1};
use nom::bytes::streaming::take_till;
use nom::character::complete::{char, line_ending, one_of, multispace0, newline, none_of};
use nom::character::is_alphanumeric;
use nom::combinator::{cond, eof, map, opt, rest};
use nom::multi::{many0, many1, many_till, separated_list0};
use nom::sequence::{terminated, tuple, preceded, delimited, pair};
use nom::Err::{Error, Failure};
use nom_locate::LocatedSpan;
use nom::Err;
use nom::error::{context, ErrorKind, ParseError};

pub type Span<'a> = LocatedSpan<&'a str>;

pub type IResult<'a, O> = nom::IResult<Span<'a>, O>;

#[cfg(all(not(target_os = "windows")))]
const NEW_LINE: &str = "\n";

#[cfg(target_os = "windows")]
const NEW_LINE: &str = "\r\n";

const SCRIPT_START: &str = "> ";
const SCRIPT_END: &str = "%}";

#[derive(PartialEq, Debug)]
pub struct RequestLine<'a> {
    pub method: Span<'a>,
    pub target: Span<'a>,
    pub version: Version,
}


pub fn parse_request_title(i: Span) -> IResult<Option<Span>> {
    let (i, optional) = context(
        "request title",
        opt(tuple((
            tag("###"),
            opt(tag(" ")),
            take_until(NEW_LINE),
            tag(NEW_LINE),
        ))))(i)?;

    if optional.is_none() {
        return Ok((i, None));
    }

    let (_, _, title, _) = optional.unwrap();

    return Ok((i, Some(title)));
}


pub(crate) fn request_line(i: Span) -> IResult<RequestLine> {
    // [method required-whitespace] request-target [required-whitespace http-version]
    let (i, method) = token(i)?;
    let (i, _) = sp(i)?;
    let (i, target) = vchar_1(i)?; // TODO: handle all valid urls, read rfc
    let (i, _) = take_while(is_space)(i)?;
    let (i, version) = http_version(i)?;
    let (i, _) = many0(tag(NEW_LINE))(i)?;

    Ok((
        i,
        RequestLine {
            method,
            version,
            target,
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

pub(crate) fn header_line(i: Span) -> IResult<Header> {
    let (i, (name, _, _, value, _)) = tuple((
        token,
        tag(":"),
        take_while(is_space),
        take_while(is_header_value_char),
        tag(NEW_LINE),
    ))(i)?;

    Ok((i, Header { name, value }))
}

pub(crate) fn parse_headers(i: Span) -> IResult<Vec<Header>> {
    let (i, headers) = many0(header_line)(i)?;
    let (i, _) = opt(line_ending)(i)?;
    Ok((i, headers))
}


fn request_title(i: Span) -> IResult<Span> {
    let (j, v) = tuple((
        tag("###"),
        many0(char(' ')),
    ))(i)?;

    Ok((i, j))
}

fn script_start(i: Span) -> IResult<Span> {
    tag(SCRIPT_START)(i)
}

fn until_new_request_title(i: Span) -> IResult<Span> {
    take_until("###")(i) // TODO: add all line matcher, not only ###
}

fn until_script_start(i: Span) -> IResult<Span> {
    take_until(SCRIPT_START)(i)
}

// consume content until EOF or script start
pub(crate) fn parse_request_body(i: Span) -> IResult<MessageBody> {
    let (i, body) = alt((until_script_start, until_new_request_title, rest))(i)?;
    if body.is_empty() {
        Ok((i, MessageBody::Empty))
    } else {
        return Ok((i, MessageBody::Bytes(body)));
    }
}

pub(crate) fn parse_input_file_ref(i: Span) -> IResult<MessageBody> {
    let (i, (_, _, file_path)) =
        tuple((tag("<"), tag(" "), take_while(|x| x != '\n' && x != '\r')))(i)?;
    Ok((i, MessageBody::File(file_path)))
}

pub(crate) fn parse_script(i: Span) -> IResult<ScriptHandler> {
    alt((
        parse_inline_script,
        parse_external_script,
        // We are returning empty instead returning error
        // because if script is given but has an error
        // how we know the error is relevant to parsing or because there is no script
        // TODO: maybe we can raise specific ParseErr and check it
        |i| Ok((i, ScriptHandler::Empty))
    ))(i)
}

pub(crate) fn parse_inline_script(i: Span) -> IResult<ScriptHandler> {
    // ‘>’ required-whitespace ‘{%’ handler-script ‘%}’
    let (i, (_, _, script, _, _)) = tuple((
        tag("> "),
        tag("{%"),
        take_until1(SCRIPT_END),
        tag(SCRIPT_END),
        many0(tag(NEW_LINE))
    ))(i)?;

    Ok((i, ScriptHandler::Inline(script)))
}

pub(crate) fn parse_external_script(i: Span) -> IResult<ScriptHandler> {
    // ‘>’ required-whitespace file-path
    let (i, (_, path, _)) = tuple((
        tag("> "),
        take_until1(NEW_LINE),
        many0(tag(NEW_LINE))
    ))(i)?;

    return Ok((i, ScriptHandler::File(path)));
}

pub fn parse_request(i: Span) -> IResult<Request> {
    let (i, title) = parse_request_title(i)?;
    let (i, line) = request_line(i)?;
    let (i, headers) = parse_headers(i)?;
    let (i, body) = parse_request_body(i)?;
    let (i, script) = parse_script(i)?;

    Ok((
        i,
        Request {
            method: Method::from(line.method),
            target: line.target.fragment().to_string(),
            version: line.version,
            title,
            headers,
            body,
            script,
        },
    ))
}

pub fn parse_multiple_request(i: Span) -> IResult<Vec<Request>> {
    let (i, (requests, _eof), ) = many_till(parse_request, eof)(i)?;
    Ok((i, requests))
    // we can split content at here and give each part of the span as separate
    // !peek(parse_request_title)(i).is_ok() && !peek(empty_lines)(i).is_ok()
}

fn token(i: Span) -> IResult<Span> {
    take_while(is_token_char)(i)
}

fn vchar_1(i: Span) -> IResult<Span> {
    take_while(is_vchar)(i)
}

fn empty_lines(i: Span) -> IResult<Span> {
    alt((tag(NEW_LINE), tag("\n"), tag("\r"), eof))(i)
}

fn is_line_ending(i: char) -> bool {
    return i == '\n';
}

fn is_token_char(i: char) -> bool {
    is_alphanumeric(i as u8) || "!#$%&'*+-.^_`|~".contains(i)
}

fn is_vchar(i: char) -> bool {
    // c.is_alphabetic()
    i as u32 > 32 && i as u32 <= 126
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
