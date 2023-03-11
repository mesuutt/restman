use crate::ast::{Header, MessageBody, Method, Request, ScriptHandler, Version};
use nom::branch::alt;
use nom::bytes::complete::{escaped, is_not, tag, take_until, take_until1, take_while};

use nom::character::complete::{crlf, line_ending, newline, one_of};

use nom::combinator::{eof, opt, peek, recognize, rest};
use nom::multi::{many0, many_till};
use nom::sequence::{pair, terminated, tuple};

use nom_locate::LocatedSpan;

use crate::combinators::*;
use nom::error::context;

pub type Span<'a> = LocatedSpan<&'a str, &'a str>;

pub type IResult<'a, O> = nom::IResult<Span<'a>, O>;

#[derive(PartialEq, Debug)]
pub struct RequestLine<'a> {
    pub method: Span<'a>,
    pub target: Span<'a>,
    pub version: Version,
}

pub fn parse_request_title(i: Span) -> IResult<Option<Span>> {
    let (i, title) = context("request title", opt(request_title))(i)?;

    return Ok((i, title));
}

pub(crate) fn request_line(i: Span) -> IResult<RequestLine> {
    // [method required-whitespace] request-target [required-whitespace http-version]
    let (i, method) = token(i)?;
    let (i, _) = sp(i)?;
    let (i, target) = vchar_1(i)?; // TODO: handle all valid urls, read rfc
    let (i, _) = take_while(is_space_char)(i)?;
    let (i, version) = http_version(i)?;
    // let (i, _) = many0(tag(NEW_LINE))(i)?;

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

// parse one header
pub(crate) fn parse_header(i: Span) -> IResult<Header> {
    let (i, _) = many0(newline)(i)?;
    let (i, (name, value)) = header(i)?;
    Ok((i, Header { name, value }))
}

// parse multiple headers
pub(crate) fn parse_headers(i: Span) -> IResult<Vec<Header>> {
    let (i, headers) = many0(parse_header)(i)?;
    // let (i, _) = opt(line_ending)(i)?;
    Ok((i, headers))
}

// consume content until script, new request or eof
pub(crate) fn parse_request_body(i: Span) -> IResult<MessageBody> {
    let (_, until_script) = peek(alt((take_until("> "), rest)))(i)?;
    let (_, until_title) = peek(alt((take_until("###"), rest)))(i)?;

    // We should consume until to [script start | new title | eof].
    // alt runs first parser until get error.
    // if first parser not return error it returns parsed from first parser.
    // I could not find good way to consume until one of [script start | next title | eof]
    // TODO: refactor this logic
    let mut body = Span::new_extra("", "");
    let mut j = Span::new_extra("", "");

    if until_script.fragment().len() > until_title.fragment().len() {
        (j, body) = alt((until_new_request_title, rest))(i)?;
    } else {
        (j, body) = alt((until_script_start, rest))(i)?;
    }

    // clean new lines from beginning of body
    let (body, _) = many0(newline)(body)?;

    if body.is_empty() {
        Ok((j, MessageBody::Empty))
    } else {
        Ok((j, MessageBody::Bytes(body)))
    }
}

// parse input file ref that will use for body
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
        |i| Ok((i, ScriptHandler::Empty)),
    ))(i)
}

pub(crate) fn parse_inline_script(i: Span) -> IResult<ScriptHandler> {
    // ‘>’ required-whitespace ‘{%’ handler-script ‘%}’
    let (i, script) = inline_script(i)?;
    Ok((i, ScriptHandler::Inline(script)))
}

pub(crate) fn parse_external_script(i: Span) -> IResult<ScriptHandler> {
    // ‘>’ required-whitespace file-path
    let (i, (_, path)) = tuple((tag("> "), alt((take_until(NEW_LINE), rest))))(i)?;

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
    let (i, (requests, _)) = many_till(parse_request, eof)(i)?;
    Ok((i, requests))
    // we can split content at here and give each part of the span as separate
    // !peek(parse_request_title)(i).is_ok() && !peek(empty_lines)(i).is_ok()
}

pub fn parse<'a>(filename: &'a str, i: &'a str) -> Vec<Request<'a>> {
    // TODO: error handling
    let (_, requests) = parse_multiple_request(Span::new_extra(i, filename)).map_err(|e| format!("request parse failed: {:?}", e)).unwrap();

    requests
}