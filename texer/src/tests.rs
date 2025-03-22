use super::*;

#[test]
fn test_parse_method_name() {
    assert_eq!(parse_method_name("'a").unwrap().1, "'");
    assert_eq!(parse_method_name("'u").unwrap().1, "'");
}

#[test]
fn test_parse_method_with_arg() {
    assert_eq!(
        parse_method_with_arg(r"\some{f}hello"),
        Ok((
            "hello",
            Method {
                method: "some",
                argument: Some("f")
            }
        ))
    );
    assert_eq!(
        parse_method_with_arg(r"\textbf{hello}foo"),
        Ok((
            "foo",
            Method {
                method: "textbf",
                argument: Some("hello")
            }
        ))
    );
    assert_eq!(
        parse_method_with_arg(r"\textbf{\textbf{hello}}"),
        Ok((
            "",
            Method {
                method: "textbf",
                argument: Some(r"\textbf{hello}")
            }
        ))
    );
    assert_eq!(
        parse_method_with_arg(r"\textbf{}test"),
        Ok((
            "test",
            Method {
                method: "textbf",
                argument: None,
            }
        ))
    );
}

#[test]
fn test_parse_method_without_arg() {
    assert_eq!(
        parse_method_without_arg(r"\textbf foo"),
        Ok((
            " foo",
            Method {
                method: "textbf",
                argument: None
            }
        ))
    );
    assert_eq!(
        parse_method_without_arg(r"\o"),
        Ok((
            "",
            Method {
                method: "o",
                argument: None
            }
        ))
    );
    assert_eq!(
        parse_method_without_arg(r"\o "),
        Ok((
            " ",
            Method {
                method: "o",
                argument: None
            }
        ))
    );
}

#[test]
fn test_parse_method() {
    let method = Method {
        method: "foo",
        argument: Some("bar"),
    };
    assert_eq!(parse_method(r"\foo{bar}hello"), Ok(("hello", method)));
    assert_eq!(
        parse_method(r"\foo{}hello"),
        Ok((
            "hello",
            Method {
                method: "foo",
                argument: None
            }
        ))
    );

    assert_eq!(
        parse_method(r"\'"),
        Ok((
            "",
            Method {
                method: "'",
                argument: None
            }
        ))
    );
    assert_eq!(
        parse_method(r"\'u"),
        Ok((
            "",
            Method {
                method: "'",
                argument: Some("u")
            }
        ))
    );
    assert_eq!(
        parse_method(r"\o"),
        Ok((
            "",
            Method {
                method: "o",
                argument: None,
            }
        ))
    );
}

#[test]
fn test_parser() {
    assert_eq!(
        parser("hello world"),
        Ok(("", Cow::Borrowed("hello world")))
    );
    assert_eq!(parser(r"\o"), Ok(("", Cow::Borrowed("ø"))));
    assert_eq!(parser(r"\o{}"), Ok(("", Cow::Borrowed("ø"))));
    assert_eq!(parser(r"\o{}c"), Ok(("c", Cow::Borrowed("ø"))));
}

#[test]
fn test_35() {
    assert_eq!(texer(r"Ci\^encias"), "Cie\u{302}ncias");
    assert_eq!(texer(r"{{Hello World}}"), "Hello World");
    assert_eq!(texer(r"\^e"), "e\u{302}");
    assert_eq!(texer(r"\^en"), "e\u{302}n");
    assert_eq!(texer(r"Ci\^encias"), "Cie\u{302}ncias");
    assert_eq!(
        texer(r"{Ci\^encias Biol\'ogicas}"),
        "Cie\u{302}ncias Biolo\u{301}gicas"
    );
    assert_eq!(
        texer(r"{{Ci\^encias Biol\'ogicas}}"),
        "Cie\u{302}ncias Biolo\u{301}gicas"
    );
    assert_eq!(
        texer(
            r"Contexto Da P\'os-Gradua\c{c}\~ao Stricto Sensu No \^Ambito Da Forma\c{c}\~ao Pedag\'ogica e a Forma\c{c}\~ao Docente Em {{Ci\^encias Biol\'ogicas}}: o Que Revela a Produ\c{c}\~ao Acad\^emica"
        ),
        "Contexto Da Po\u{301}s-Graduac\u{327}a\u{303}o Stricto Sensu No A\u{302}mbito Da Formac\u{327}a\u{303}o Pedago\u{301}gica e a Formac\u{327}a\u{303}o Docente Em Cie\u{302}ncias Biolo\u{301}gicas: o Que Revela a Produc\u{327}a\u{303}o Acade\u{302}mica",
    )
}

#[test]
fn test_46() {
    assert_eq!(texer("\\\"u"), "u\u{308}");
    assert_eq!(texer("\\\"{u}"), "u\u{308}");
    assert_eq!(texer("Sch\\\"{u}tt"), "Schu\u{308}tt");
}

#[test]
fn test_delimited_by_brackets() {
    assert_eq!(delimited_by_brackets("{abc}"), Ok(("", "abc")));
    assert_eq!(
        delimited_by_brackets(r"{\foo{bar}}"),
        Ok(("", r"\foo{bar}"))
    );
}

#[test]
fn test_texer() {
    assert_eq!(texer("hello world"), "hello world");
    if let Cow::Owned(_) = texer("hello world") {
        panic!();
    }
    assert_eq!(texer(r"hello w\o{}rld"), "hello wørld",);
    assert_eq!(texer(r"hello w\o{}"), "hello wø",);
    assert_eq!(texer(r"hello w\o"), "hello wø",);
    assert_eq!(texer(r"hello w\o{}rld asdf"), "hello wørld asdf");

    assert_eq!(texer(r"hello w{o}rld"), "hello world");
    assert_eq!(texer(r"hello {}{}wo{}rld"), "hello world");

    assert_eq!(texer("{ab}"), "ab");
    assert_eq!(texer("{foo bar}"), "foo bar");

    assert_eq!(texer(r"\'u"), "u\u{301}");
    assert_eq!(texer(r"\c{c}"), "c\u{0327}");
    assert_eq!(texer(r"\v{c}"), "c\u{30C}");
    assert_eq!(texer(r"{\o}"), "ø");
    assert_eq!(texer(r"{\o}a"), "øa");
    assert_eq!(texer(r"{\o}"), "ø");
    assert_eq!(texer(r#"{\"o}"#), "o\u{308}");
    assert_eq!(texer(r"a {some}"), "a some");
    assert_eq!(texer(r"a {\some}"), "a some");

    assert_eq!(texer(r"\^e"), "e\u{302}");

    assert_eq!(texer(r"a  b"), r"a b");
    assert_eq!(
        texer(
            r"
              "
        ),
        " "
    );
    assert_eq!(texer(r"{a  b}"), r"a b");
    assert_eq!(texer(r"a {$so\ ^\ast asda \me$}"), r"a so\ ^\ast asda \me");

    assert_eq!(texer(r"$ab$"), r"ab");
    assert_eq!(texer(r"a {$some$}"), "a some");
    // TODO Figure this
    // assert_eq!(texer(r"$a  b$"), r"a b");
}

#[test]
fn test_multispacer() {
    assert_eq!(multispacer("\n a"), Ok(("a", " ")));
    assert_eq!(multispacer("\r  \n\r \n\na"), Ok(("a", " ")));
    assert_eq!(multispacer("\n  "), Ok(("", " ")));
    assert!(multispacer("a").is_err());
}

#[test]
fn test_till_symbols_no_ws_once() {
    assert_eq!(till_symbols_no_ws_once("hello"), Ok(("", "hello")));
    assert_eq!(till_symbols_no_ws_once(" hello"), Ok(("hello", " ")));
    assert_eq!(till_symbols_no_ws_once("  hello"), Ok(("hello", " ")));
    assert_eq!(till_symbols_no_ws_once(" \n hello"), Ok(("hello", " ")));

    assert_eq!(till_symbols_no_ws_once("a"), Ok(("", "a")));
    assert_eq!(till_symbols_no_ws_once("a  b"), Ok(("  b", "a")));
    assert_eq!(till_symbols_no_ws_once("a  "), Ok(("  ", "a")));
    assert_eq!(till_symbols_no_ws_once("  b"), Ok(("b", " ")));
    assert_eq!(till_symbols_no_ws_once("a"), Ok(("", "a")));

    assert_eq!(till_symbols_no_ws_once("a b"), Ok((" b", "a")));
    assert_eq!(till_symbols_no_ws_once(" b"), Ok(("b", " ")));
}

#[test]
fn test_till_symbols_no_ws() {
    assert_eq!(
        till_symbols_no_ws(" \n hello"),
        Ok(("", Cow::Owned(" hello".into())))
    );
    if let Ok((input, Cow::Owned(_data))) = till_symbols_no_ws("hello world") {
        assert_eq!(input, "hello world");
        panic!();
    }

    if let Ok((_, Cow::Owned(_))) = till_symbols_no_ws("a b") {
        panic!();
    }
    assert_eq!(
        till_symbols_no_ws("a  b"),
        Ok(("", Cow::Owned("a b".to_string())))
    );

    assert_eq!(
        till_symbols_no_ws("hello"),
        Ok(("", Cow::Borrowed("hello")))
    );
    assert_eq!(
        till_symbols_no_ws("hello world"),
        Ok(("", Cow::Borrowed("hello world")))
    );
    assert_eq!(
        till_symbols_no_ws("hello  world"),
        Ok(("", Cow::Owned("hello world".into())))
    );
}

#[test]
fn test_tag_consecutive_spaces() {
    assert_eq!(tag_consecutive_spaces(""), Ok(("", "")));
    assert_eq!(tag_consecutive_spaces("a b"), Ok(("", "a b")));
    assert_eq!(tag_consecutive_spaces("a b c"), Ok(("", "a b c")));
    assert_eq!(
        tag_consecutive_spaces("hello world"),
        Ok(("", "hello world"))
    );

    assert_eq!(tag_consecutive_spaces("a  c"), Ok(("  c", "a")));
    assert_eq!(tag_consecutive_spaces("aa  c"), Ok(("  c", "aa")));
    assert_eq!(tag_consecutive_spaces("aa  c  d"), Ok(("  c  d", "aa")));
    assert_eq!(tag_consecutive_spaces("  "), Ok(("  ", "")));
    assert_eq!(tag_consecutive_spaces(" \n "), Ok((" \n ", "")));
}
