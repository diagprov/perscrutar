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
use std::ops::Not;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while, take_while1, take_until, is_not},
    character::complete::{alphanumeric1 as alphanumeric, char, one_of},
    character::{is_alphabetic, is_alphanumeric},
    combinator::{cut, map, opt, value},
    error::{context, convert_error, ContextError, ErrorKind, ParseError, VerboseError},
    multi::{fold_many0, separated_list0},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
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
  let chars = "-_.,;:/ ^$+*\\\n";

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

/**
Utility function, remove comments entirely
*/
fn eolcomment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
  value(
    (), // Output is thrown away.
    tuple((
        tag("#"),
        take_until("\n"),
        tag("\n")
    ))
  )(i)
}

fn parse_str_with_comments<'a, E: ParseError<&'a str>>(i: &'a str) 
-> IResult<&'a str, String, E> {
  map(separated_list0(eolcomment, parse_str), |result: Vec<&str>| {
    let mut s = String::new();
    for r in result.iter() {
        s.push_str(r)
    }
    s.clone()
  })(i)
}

fn alphabeticlabel_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    alt((terminated(alphabeticlabel, eolcomment),
         alphabeticlabel))(i)
}

/** String_spm finds entries surrounded by 
  "" possibly split over multiple lines
*/
fn string_spm<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, String, E> {
  context(
    "string",
    preceded(char('\"'), cut(terminated(parse_str_with_comments, char('\"')))),
  )(i)
}

/** String_spm finds entries surrounded by 
  {} possibly split over multiple lines
*/
fn string_brc<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, String, E> {
  context(
    "string",
    preceded(char('{'), cut(terminated(parse_str_with_comments, char('}')))),
  )(i)
}

fn key_value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, (&'a str, String), E> {
  separated_pair(
    preceded(sp, alphabeticlabel_comment),
    cut(preceded(sp, char('='))),
    preceded(sp, alt((string_spm, string_brc)))
  )(i)
}

fn kvlist<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, HashMap<String, String>, E> {
    let sep = alt((
            terminated(preceded(sp, tag(",")), preceded(sp, eolcomment)),
            terminated(tag(","), preceded(sp, eolcomment)),
            terminated(preceded(sp, tag(",")), eolcomment),
            preceded(sp, tag(",")),
            tag(","),
        ));
    context(
        "map",
        cut(terminated(
            map(
            separated_list0(sep, key_value),
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

#[derive(Debug)]
struct BibItem<'a>(&'a str, &'a str, HashMap<String, String>);

fn bibentry<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, (&str, &str, HashMap<String, String>), E> {
    context(
        "bibitem",
        preceded(sp,
        preceded(
            char('@'),
            tuple((
                cut(terminated(
                    terminated(alphabeticlabel_comment, sp),
                    char('{'),
                )),
                cut(terminated(
                    preceded(sp, terminated(alphabeticlabel_comment, sp)),
                    char(','),
                )),
                cut(terminated(
                    kvlist,
                    char('}'),
                )),
            )),
        ),
        ),
    )(i)
}

#[cfg(test)]
mod tests {
  
    use nom::Err::Failure;
    use nom::error::ErrorKind;
    use super::*;

    #[test]
    fn test_comment() {
        let r1t = r#"This is valid#This is a comment
Test more also this line # ends with a comment too
#starts with a comment
#gogogo
Ok, no comment."#;
        let r1 = parse_str_with_comments::<(&str, ErrorKind)>(r1t);
        println!("{:?}", r1);
        assert_eq!(r1, Ok(("", String::from("This is validTest more also this line Ok, no comment."))));
        /*let r2 = comment_discarded::<(&str, ErrorKind)>("This is valid # This is a comment");
        println!("{:?}", r2);*/
    }

    #[test]
    fn test_kv_one() {
        
        let r1 = key_value::<(&str, ErrorKind)>(" Author = {Some Author}");
        assert_eq!(r1, Ok(("", ("Author", String::from("Some Author")))));
        //println!("{:?}", r1);

        let r2 = key_value::<(&str, ErrorKind)>("   Author = \"Sömé Àüthör\",");
        assert_eq!(r2, Ok((",", ("Author", String::from("Sömé Àüthör")))));
        //println!("{:?}", r2);
    
        let r3 = key_value::<(&str, ErrorKind)>("   Author = {Sömé Àüthör\",");
        //println!("{:?}", r3);
        assert_eq!(r3, Err(Failure(("\",", ErrorKind::Char))));

        let r4 = key_value::<(&str, ErrorKind)>("{Author Sömé Àüthör");
        assert!(r4.is_err());

        let r5 = key_value::<(&str, ErrorKind)>("title = {Primes of the form $x^2 + ny^2$: Fermat, Class Field Theory, and Complex Multiplication},");
        //println!("{:?}", r5);
        assert!(r5.is_err() == false);

        let r6 = key_value::<(&str, ErrorKind)>("title = {Primes of the form $x^2 + ny^2$: Fermat, Class Field Theory, and Complex Multiplication}, # some comment");
        println!("{:?}", r6);
        
        let r7 = key_value::<(&str, ErrorKind)>("title = {Primes of # some comment\nthe form $x^2 + ny^2$: Fermat, Class Field Theory, and Complex Multiplication}");
        //println!("{:?}", r7);
        assert!(r7.is_err() == false);

        let r8 = key_value::<(&str, ErrorKind)>("ti # tle = {Primes of # some comment");
        println!("{:?}", r8);

        let r9t = r#"
            Author = {Some Author and
                # some
                Sömé Àüthör};
        "#;

        let r9 = key_value::<(&str, ErrorKind)>(r9t);
        //println!("{:?}", r9);
        assert!(r9.is_err() == false);
    }

    #[test]
    fn test_kvpairs() {
        let b1 = r#"
        author = {Some Author},
        title = {Some fancy title},
        isbn = "111-111123212-1111"
           
        "#;

        let r1 = kvlist::<(&str, ErrorKind)>(b1);
        //println!("{:?}", r1);
        assert!(r1.is_err() == false);
    }

    #[test]
    fn test_bibentry() {

        let b1 = r#"
        @book{Ref-Name,
            author = {Some Author},
            title = {Some fancy title} ,
            isbn = {111-111123212-1111}
        }
        "#;

        let b2 = r#"
@book{Cox-CFT,
    author = {David A. Cox},
    title = {Primes of the form $x^2 + ny^2$: Fermat, Class Field Theory, and Complex Multiplication},
    edition = {2nd ed.},
    publisher = {John Wiley and Sons Inc},
    year = {2013},
    ISBN = {978-1-118-39018-4},
    doi = {10.1002/9781118400722}
}
        "#;

        let b2a = r#"
@book { Cox-CFT,
    author = {David A. Cox},
    title = {Primes of the form $x^2 + ny^2$: Fermat, Class Field Theory, and Complex Multiplication},
    edition = {2nd ed.},
    publisher = {John Wiley and Sons Inc},
    year = {2013},
    ISBN = {978-1-118-39018-4},
    doi = {10.1002/9781118400722}
}
        "#;

        let b3 = r#"
@book{Cox-CFT,
    author = {David A. Cox},
    title = {Primes of the form $x^2 + ny^2$: Fermat, 
        Class Field Theory, and Complex Multiplication},
    # edition = {2nd ed.},
    publisher = {John Wiley and Sons Inc},
    # comment:
    year = {2013},
    ISBN = {978-1-118-39018-4}, # comment
    doi = {10.1002/9781118400722}
}
        "#;


        let r1 = bibentry::<(&str, ErrorKind)>(b1);
        println!("{:?}", r1);
        assert!(r1.is_err() == false);

        let r2 = bibentry::<(&str, ErrorKind)>(b2);
        println!("{:?}", r2);
        assert!(r2.is_err() == false);

        let r2a = bibentry::<(&str, ErrorKind)>(b2a);
        println!("{:?}", r2a);
        assert!(r2a.is_err() == false);

        let r3 = bibentry::<(&str, ErrorKind)>(b3);
        println!("{:?}", r3);
        assert!(r3.is_err() == false);

    }
}
