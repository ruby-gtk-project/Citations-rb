use anyhow::{bail, Context, Result};
use std::cell::RefCell;
use std::iter::Iterator;
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum GrammarItem {
    Text(String),
    /// A command with an optional argument
    Command(String),
    Brace,        // {
    Math(String), // $
}

#[derive(Debug, Clone)]
pub struct ParseNode {
    pub children: Vec<ParseNode>,
    pub entry: GrammarItem,
}

impl ParseNode {
    fn new(entry: GrammarItem) -> Self {
        Self {
            children: Vec::new(),
            entry,
        }
    }
}

struct Lexer {
    inner: Vec<LexItem>,
    position: RefCell<usize>,
}

impl Lexer {
    fn next(&self) -> Option<&LexItem> {
        let item = self.inner.get(*self.position.borrow());
        self.bump();
        item
    }

    fn next_if(&self, func: impl FnOnce(&LexItem) -> bool) -> Option<&LexItem> {
        let item = self.peek()?;
        if func(item) {
            self.bump();
            Some(item)
        } else {
            None
        }
    }

    fn peek(&self) -> Option<&LexItem> {
        self.inner.get(*self.position.borrow())
    }

    fn bump(&self) {
        let mut position = self.position.borrow_mut();
        *position += 1;
    }

    fn is_finished(&self) -> bool {
        *self.position.borrow() == self.inner.len()
    }

    fn position(&self) -> usize {
        *self.position.borrow()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexItem {
    OpBrace,
    EdBrace,
    BackSlash,
    Char(char),
    Space,
    Dollar,
}

impl LexItem {
    fn is_char(&self) -> bool {
        if let LexItem::Char(_) = self {
            true
        } else {
            false
        }
    }

    fn is_dollar(&self) -> bool {
        if let LexItem::Dollar = self {
            true
        } else {
            false
        }
    }
}

impl Lexer {
    fn new(input: &str) -> Self {
        let inner = input.chars().map(lex_char).collect();
        Self {
            inner,
            position: RefCell::new(0),
        }
    }
}

fn lex_char(c: char) -> LexItem {
    match c {
        '{' => LexItem::OpBrace,
        '}' => LexItem::EdBrace,
        '\\' => LexItem::BackSlash,
        '$' => LexItem::Dollar,
        ' ' => LexItem::Space,
        _ => LexItem::Char(c),
    }
}

impl From<&LexItem> for char {
    fn from(i: &LexItem) -> Self {
        match i {
            LexItem::OpBrace => '{',
            LexItem::EdBrace => '}',
            LexItem::BackSlash => '\\',
            LexItem::Char(c) => *c,
            LexItem::Space => ' ',
            LexItem::Dollar => '$',
        }
    }
}

pub fn parse(input: &str) -> Result<ParseNode> {
    let tokens = Lexer::new(input);
    let node = parse_expr(&tokens)?;
    if !tokens.is_finished() {
        anyhow::bail!(
            "Expected end of input, found {:?} at {}",
            tokens.next(),
            tokens.position(),
        )
    }
    Ok(node)
}

fn parse_expr(tokens: &Lexer) -> Result<ParseNode> {
    let node_summand = parse_term(tokens)?;
    match tokens.peek().context("Nothing at postion {pos}")? {
        LexItem::BackSlash => {
            tokens.bump();
            let (command, argument) = parse_command(tokens)?;
            // let mut node = ParseNode::new(GrammarItem::Command(command, argument));
            todo!()
            // node.children.push(node_summand);
            // Ok(node)
        }
        LexItem::Dollar => {
            let math = parse_math(tokens)?;
            let mut node = ParseNode::new(GrammarItem::Math(math));
            node.children.push(node_summand);
            Ok(node)
        }
        LexItem::Space => {
            tokens.bump();
            parse_expr(tokens)
        }
        _ => Ok(node_summand),
    }
}

fn parse_term(tokens: &Lexer) -> Result<ParseNode> {
    let c: &LexItem = tokens
        .peek()
        .context("Unexpected end of input, expected paren or number")?;
    match c {
        LexItem::Char(_c) => {
            let string = parse_words(tokens);
            let node = ParseNode::new(GrammarItem::Text(string));
            Ok(node)
        }
        LexItem::OpBrace => {
            tokens.bump();
            let node = parse_expr(tokens)?;
            if let Some(&LexItem::EdBrace) = tokens.peek() {
                tokens.bump();
                let mut paren = ParseNode::new(GrammarItem::Brace);
                paren.children.push(node);
                Ok(paren)
            } else {
                anyhow::bail!("Expected to find }} at pos {}", tokens.position());
            }
        }
        _ => anyhow::bail!("Unexpected token {c:?}, expected paren or number"),
    }
}

fn parse_command(tokens: &Lexer) -> Result<(String, Option<String>)> {
    let mut result = "".to_string();
    while let Some(LexItem::Char(c)) = tokens.peek() {
        tokens.bump();
        result.push(*c);
    }
    if let Some(LexItem::OpBrace) = tokens.peek() {
        let mut argument = "".to_string();
        tokens.bump();
        while let Some(LexItem::Char(c)) = tokens.peek() {
            argument.push(*c);
            tokens.bump();
        }
        if !matches!(tokens.peek(), Some(LexItem::EdBrace)) {
            anyhow::bail!("Expecting ending brace");
        }
        Ok((result, Some(argument)))
    } else if let Some(LexItem::Space) = tokens.peek() {
        todo!();
    } else {
        Ok((result, None))
    }
}

fn parse_words(tokens: &Lexer) -> String {
    let mut result = "".to_string();
    while let Some(lex_item) = tokens
        .next_if(|item| !matches!(item, LexItem::Dollar | LexItem::OpBrace | LexItem::EdBrace))
    {
        result.push(lex_item.into());
    }
    tokens.bump();
    result
}

fn parse_math(tokens: &Lexer) -> Result<String> {
    if !tokens.next().context("TODO")?.is_dollar() {
        bail!("Does not start with dollar");
    }
    let mut result = "".to_string();
    while let Some(lex_item) = tokens.next_if(|x| !x.is_dollar()) {
        result.push(lex_item.into());
    }
    // Consume the dollar.
    if tokens.is_finished() {
        bail!("Could not find matching dollar");
    } else {
        tokens.bump();
    }

    Ok(result)
}

pub fn print(tree: &ParseNode) -> String {
    match &tree.entry {
        GrammarItem::Brace => {
            format!(
                r"{{{}}}",
                print(tree.children.get(0).expect("parens need one child"))
            )
        }
        GrammarItem::Text(text) => {
            format!("{}", text)
        }
        _ => todo!(),
        // GrammarItem::Command => {
        //     let command = print(tree.children.get(0).expect("products need two children"));
        //     let argument = print(tree.children.get(1).expect("products need two children"));
        //     format!(r"\{command}{argument}", command, argument)
        // }
        // GrammarItem::Number(n) => format!("{}", n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_math() {
        let text = "$abc$";
        let lexer = Lexer::new(text);
        assert_eq!(parse_math(&lexer).unwrap(), "abc");
        assert_eq!(lexer.peek(), None);
        assert!(lexer.is_finished());

        let text = "$abc";
        let lexer = Lexer::new(text);
        assert!(parse_math(&lexer).is_err());
    }
}
