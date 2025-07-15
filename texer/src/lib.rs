mod utils;

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;

use nom::error::Error;
use nom::error::ErrorKind;
use nom::error::ParseError;
use nom::sequence::Tuple;
use nom::{
    AsChar, IResult, InputTakeAtPosition,
    branch::alt,
    bytes::complete::{tag, take, take_while, take_while1},
    character::complete::char,
    multi::fold_many0,
    sequence::delimited,
};

use crate::utils::combine_unicode;

static SYMBOLS_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    [("o", "ø"), ("O", "Ø"), ("l", "ł"), ("L", "Ł")]
        .into_iter()
        .collect()
});

#[derive(Debug, Default, PartialEq)]
struct Method<'a> {
    method: &'a str,
    argument: Option<&'a str>,
}

fn parse_method(input: &str) -> IResult<&str, Method<'_>> {
    alt((
        parse_method_with_arg,
        parse_method_without_arg,
        parse_nonalpha_method,
    ))(input)
}

fn parse_method_with_arg(input: &str) -> IResult<&str, Method<'_>> {
    let (input, (_, method, argument)) = (
        tag(r"\"),
        take_while1(|c: char| c.is_alphabetic()),
        delimited_by_brackets,
    )
        .parse(input)?;
    let argument = if argument.is_empty() {
        None
    } else {
        Some(argument)
    };

    Ok((input, Method { method, argument }))
}

fn parse_method_name(input: &str) -> IResult<&str, &str> {
    Ok((&input[1..], &input[0..1]))
}

fn parse_nonalpha_method(input: &str) -> IResult<&str, Method<'_>> {
    let (input, (_, method)) = (tag(r"\"), parse_method_name).parse(input)?;
    if input.starts_with(|c: char| c.is_alphabetic()) {
        let (input, argument) = take::<_, _, ()>(1_usize)(input).unwrap();
        Ok((
            input,
            Method {
                method,
                argument: Some(argument),
            },
        ))
    } else if input.starts_with('{') {
        let (input, (_, argument, _)) = (tag("{"), take(1_usize), tag("}")).parse(input)?;
        Ok((
            input,
            Method {
                method,
                argument: Some(argument),
            },
        ))
    } else {
        Ok((
            input,
            Method {
                method,
                argument: None,
            },
        ))
    }
}

fn parse_method_without_arg(input: &str) -> IResult<&str, Method<'_>> {
    let (input, (_, method)) =
        (tag(r"\"), take_while1(|c: char| c.is_alphabetic())).parse(input)?;

    Ok((
        input,
        Method {
            method,
            argument: None,
        },
    ))
}

// Parses `{anything}`.
fn delimited_by_brackets(input: &str) -> IResult<&str, &str> {
    delimited(tag("{"), take_until_unbalanced('{', '}'), tag("}"))(input)
}

fn till_symbol(s: &str) -> IResult<&str, &str> {
    take_while(|c| c != '{' && c != '\\' && c != '$')(s)
}

fn parser(input: &str) -> IResult<&str, Cow<'_, str>> {
    if input.starts_with(r"\") {
        let (input, Method { method, argument }) = parse_method(input)?;
        if let Some(argument) = argument {
            if argument.len() == 1 && method.len() == 1 {
                Ok((input, combine_unicode(method, argument)))
            } else {
                // TODO Check
                Ok((input, Cow::Owned(format!("{method}{argument}"))))
            }
        } else if let Some(replacement) = SYMBOLS_MAP.get(method) {
            Ok((input, Cow::Borrowed(replacement)))
        } else {
            Ok((input, Cow::Borrowed(method)))
        }
    } else if input.starts_with("$") {
        let (input, data) = delimited(tag("$"), take_while(|c| c != '$'), tag("$"))(input)?;
        Ok((input, Cow::Borrowed(data)))
    } else if input.starts_with("{") {
        let (input, mut data) = delimited_by_brackets(input)?;
        let remainder = &mut data;
        let mut acc = Cow::Borrowed("");

        while !remainder.is_empty() {
            let (r, new_data) = parser(remainder)?;
            *remainder = r;
            acc.to_mut().push_str(&new_data);
        }

        Ok((input, acc))
    } else {
        till_symbols_no_ws(input)
    }
}

pub fn texer(og_input: &str) -> Cow<'_, str> {
    let mut res = Cow::Borrowed("");
    let mut i = og_input;
    let mut first = true;

    while !i.is_empty() {
        match parser(i) {
            Ok((input, data)) => {
                if first && input.is_empty() {
                    return data;
                }
                first = false;
                res.to_mut().push_str(&data);
                i = input;
            }
            Err(e) => match e {
                nom::Err::Incomplete(_needed) => todo!(),
                nom::Err::Error(nom::error::Error { input, .. })
                | nom::Err::Failure(nom::error::Error { input, .. }) => {
                    res.to_mut().push_str(input);
                    return res;
                }
            },
        }
    }
    if res.is_empty() {
        Cow::Borrowed(og_input)
    } else {
        res
    }
}

fn take_until_unbalanced(
    opening_bracket: char,
    closing_bracket: char,
) -> impl Fn(&str) -> IResult<&str, &str> {
    move |i: &str| {
        let mut index = 0;
        let mut bracket_counter = 0;
        while let Some(n) = &i[index..].find(&[opening_bracket, closing_bracket][..]) {
            index += n;
            let mut it = i[index..].chars();
            match it.next() {
                Some(c) if c == opening_bracket => {
                    bracket_counter += 1;
                    index += opening_bracket.len_utf8();
                }
                Some(c) if c == closing_bracket => {
                    // Closing bracket.
                    bracket_counter -= 1;
                    index += closing_bracket.len_utf8();
                }
                // Can not happen.
                _ => unreachable!(),
            };
            // We found the unmatched closing bracket.
            if bracket_counter == -1 {
                // We do not consume it.
                index -= closing_bracket.len_utf8();
                return Ok((&i[index..], &i[0..index]));
            };
        }

        if bracket_counter == 0 {
            Ok(("", i))
        } else {
            Err(nom::Err::Error(Error::from_error_kind(
                i,
                ErrorKind::TakeUntil,
            )))
        }
    }
}

fn till_symbol1(s: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c != '{' && c != '\\' && c != '$' && !c.is_whitespace())(s)
}

// Turns multiple consecutive spaces into a single space. Taken from nom
// https://docs.rs/nom/7.1.3/src/nom/character/complete.rs.html#695
fn multispacer(input: &str) -> IResult<&str, &str> {
    let (input, _) = input.split_at_position1_complete(
        |item| {
            let c = item.as_char();
            // The original check was: !(c == ' ' || c == '\t' || c == '\r' || c == '\n')
            !c.is_whitespace()
        },
        ErrorKind::MultiSpace,
    )?;
    Ok((input, " "))
}

fn weird_whitespaces(input: &str) -> IResult<&str, &str> {
    let (input, _) = char('\u{2009}')(input)?;
    Ok((input, " "))
}

fn till_symbols_no_ws_once(input: &str) -> IResult<&str, &str> {
    alt((multispacer, till_symbol1, weird_whitespaces))(input)
}

fn till_symbols_no_ws(input: &str) -> IResult<&str, Cow<'_, str>> {
    let (input, data) = tag_consecutive_spaces(input)?;
    if !input.is_empty() {
        fold_many0(
            till_symbols_no_ws_once,
            || Cow::Borrowed(data),
            |mut acc, item| {
                acc.to_mut().push_str(item);
                acc
            },
        )(input)
    } else {
        let (input, data) = till_symbol(data)?;
        Ok((input, Cow::Borrowed(data)))
    }
}

/// Parses until two or more consecutive spaces are found.
fn tag_consecutive_spaces(input: &str) -> IResult<&str, &str> {
    let mut iter = input
        .char_indices()
        .map(|(pos, c)| (pos, c.is_whitespace()))
        .peekable();
    while let (Some((pos, is_whitespace)), Some((_, next_is_whitespace))) =
        (iter.next(), iter.peek())
    {
        if is_whitespace && *next_is_whitespace {
            return Ok((&input[pos..], &input[..pos]));
        }
    }
    Ok(("", input))
}

#[cfg(test)]
mod tests;
