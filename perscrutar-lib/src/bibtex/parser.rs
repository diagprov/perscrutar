/**

The goal of this parser is to read in something like this:

@book{Cox-CFT,
    author = {David A. Cox},
    title = {Primes of the form $x^2 + ny^2$: Fermat, Class Field Theory, and Complex Multiplication},
    edition = {2nd ed.},
    publisher = {John Wiley and Sons Inc},
    year = {2013},
    ISBN = {978-1-118-39018-4},
    doi = {10.1002/9781118400722}
}


*/

use std::str;

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while},
    character::complete::{alphanumeric1 as alphanumeric, char, one_of},
    character::{is_alphabetic, is_alphanumeric},
    combinator::{cut, map, opt, value},
    error::{context, convert_error, ContextError, ErrorKind, ParseError, VerboseError},
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair, terminated},
    Err, IResult,
};

use crate::bibtex::data::*;

/**
Space Parser
*/
fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
  let chars = " \t\r\n";

  // nom combinators like `take_while` return a function. That function is the
  // parser,to which we can pass the input
  take_while(move |c| chars.contains(c))(i)
}

fn alphabeticlabel<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
  let chars = "-_";

  take_while(move |c: char| {
    is_alphabetic(c as u8) || chars.contains(c)
  })(i)
}

fn alphanumericplus<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
  let chars = "-_.,;:/";

  take_while(move |c: char| {
    is_alphanumeric(c as u8) || chars.contains(c)
  })(i)
}

/**
Parse alphanumeric strings, allowing escapes and other properties 
that can be inside a label.
*/
fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
  escaped(alphanumericplus, '\\', one_of("\"n\\"))(i)
}


fn string_spm<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, &'a str, E> {
  context(
    "string",
    preceded(char('\"'), cut(terminated(parse_str, char('\"')))),
  )(i)
}

fn string_brc<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, &'a str, E> {
  context(
    "string",
    preceded(char('{'), cut(terminated(parse_str, char('}')))),
  )(i)
}

fn key_value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, (&'a str, JsonValue), E> {
  separated_pair(
    preceded(sp, alphabeticlabel),
    cut(preceded(sp, char('='))),
    ,
  )(i)
}

#[cfg(test)]
mod tests {
    
    use super::*;

}

