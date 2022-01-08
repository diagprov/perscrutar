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
use std::collections::HashMap;

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

use nom_unicode::is_alphanumeric as is_alphanumeric_unicode;
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
  let chars = "-_.,;:/ ";

  take_while(move |c: char| {
    is_alphanumeric_unicode(c as char) || chars.contains(c)
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
) -> IResult<&'a str, (&'a str, &'a str), E> {
  separated_pair(
    preceded(sp, alphabeticlabel),
    cut(preceded(sp, char('='))),
    preceded(sp, alt((string_spm, string_brc)))
  )(i)
}

fn kvlist<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, HashMap<String, String>, E> {
    context(
        "map",
        cut(terminated(
            map(
            separated_list0(preceded(sp, char(',')), key_value),
                |tuple_vec| {
                    tuple_vec
                    .into_iter()
                    .map(|(k, v)| (String::from(k), String::from(v)))
                .collect()
                },
            ),
            sp,
        )),
    )(i)
}

#[cfg(test)]
mod tests {
  
    use nom::Err::Failure;
    use nom::error::ErrorKind;
    use super::*;

    #[test]
    fn test_kv() {
        
        let r1 = key_value::<(&str, ErrorKind)>(" Author = {Antony Vennard}");
        assert_eq!(r1, Ok(("", ("Author", "Antony Vennard"))));
        println!("{:?}", r1);

        let r2 = key_value::<(&str, ErrorKind)>("   Author = \"Antöny Vénnärd\",");
        assert_eq!(r2, Ok((",", ("Author", "Antöny Vénnärd"))));
        println!("{:?}", r2);
    
        let r3 = key_value::<(&str, ErrorKind)>("   Author = {Antöny Vénnärd\",");
        println!("{:?}", r3);
        assert_eq!(r3, Err(Failure(("\",", ErrorKind::Char))));

        let r4 = key_value::<(&str, ErrorKind)>("{Author Antöny Vénnärd");
        assert!(r4.is_err());
    }

    #[test]
    fn test_kvpairs() {
        let b1 = r#"
        author = {Antony Vennard},
        title = {Some fancy title},
        isbn = "111-111123212-1111"
           
        "#;

        let r1 = kvlist::<(&str, ErrorKind)>(b1);
        println!("{:?}", r1);
        assert!(r1.is_err() == false);
    }

    #[test]
    fn test_bibentry() {

        let _b1 = r#"
        @book{
        }
        "#;

        let _b2 = r#"
        @book{
            author = {Antony Vennard},
            title = {Some fancy title},
            isbn = {111-111123212-1111}
        }
        "#;
    }
}
