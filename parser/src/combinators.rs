use crate::parsers::{IResult, Span};
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_till, take_until, take_until1, take_while};
use nom::character::complete::{char, crlf, newline};
use nom::character::{is_alphanumeric, is_space};
use nom::combinator::{eof, opt, peek, recognize};
use nom::multi::{many0, many0_count};
use nom::sequence::{pair, tuple};

#[cfg(all(not(target_os = "windows")))]
pub(crate) const NEW_LINE: &str = "\n";

#[cfg(target_os = "windows")]
const NEW_LINE: &str = "\r\n";

const SCRIPT_START: &str = "> ";
const SCRIPT_END: &str = "%}";

pub fn request_title(i: Span) -> IResult<Span> {
    let (i, (_, _, _, title, _)) = tuple((
        many0(tag(NEW_LINE)),
        tag("###"),
        many0(tag(" ")),
        take_until(NEW_LINE),
        tag(NEW_LINE),
    ))(i)?;

    return Ok((i, title));
}

pub fn until_new_request_title(i: Span) -> IResult<Span> {
    take_until("###")(i) // TODO: add all line matcher, not only ###
}

pub fn until_script_start(i: Span) -> IResult<Span> {
    take_until(SCRIPT_START)(i)
}

pub fn inline_script(i: Span) -> IResult<Span> {
    let (i, (_, _, script, _, _)) = tuple((
        tag("> "),
        tag("{%"),
        take_until1(SCRIPT_END),
        tag(SCRIPT_END),
        many0(tag(NEW_LINE)),
    ))(i)?;

    Ok((i, script))
}

pub fn header(i: Span) -> IResult<(Span, Span)> {
    let (i, (name, _, _, value, _)) = tuple((
        token,
        tag(":"),
        take_while(is_space_char),
        take_while(is_header_value_char),
        tag(NEW_LINE)
    ))(i)?;

    Ok((i, (name, value)))
}

////////////////////////

pub fn is_header_value_char(i: char) -> bool {
    /*let i = match i.to_digit(10) {
        None => return false,
        Some(x) => x,
    };
    */
    let i = i as u32;

    i == 9 || (i >= 32 && i <= 126)
}

pub fn token(i: Span) -> IResult<Span> {
    take_while(is_token_char)(i)
}

pub fn vchar_1(i: Span) -> IResult<Span> {
    take_while(is_vchar)(i)
}

fn empty_lines(i: Span) -> IResult<Span> {
    alt((tag(NEW_LINE), tag("\n"), tag("\r"), eof))(i)
}

fn is_token_char(i: char) -> bool {
    is_alphanumeric(i as u8) || "!#$%&'*+-.^_`|~".contains(i)
}

fn is_vchar(i: char) -> bool {
    // c.is_alphabetic()
    i as u32 > 32 && i as u32 <= 126
}

pub fn is_space_char(x: char) -> bool {
    x == ' '
}

pub fn sp(i: Span) -> IResult<char> {
    char(' ')(i)
}


