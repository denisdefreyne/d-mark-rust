//! A parser for [D★Mark](https://ddfreyne.github.io/d-mark/).
//!
//! # Examples
//!
//! ```
//! let contents = "#p[only=web] I %em{love} Rust!";
//!
//! let parsed = d_mark::Parser::call(&contents)
//!   .expect("parsing failed");
//! println!("{:#?}", parsed);
//! ```

mod tests;
mod translator;
mod util;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use util::FilterableResult;

pub use translator::Translator;

#[derive(Debug, PartialEq)]
pub struct ElementNode {
    name: Cow<'static, str>,
    attributes: HashMap<String, String>,
    children: Vec<Node>,
}

#[derive(Debug, PartialEq)]
pub struct StringNode {
    content: Cow<'static, str>,
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Element(ElementNode),
    String(StringNode),
}

#[derive(Debug)]
struct Pos {
    idx: usize,
    col_nr: usize,
    line_nr: usize,
}

impl Pos {
    fn new() -> Pos {
        Pos {
            idx: 0,
            col_nr: 0,
            line_nr: 0,
        }
    }

    fn advance(&mut self, nl: bool) {
        self.idx += 1;

        if nl {
            self.col_nr = 0;
            self.line_nr += 1;
        } else {
            self.col_nr += 1;
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    /// An unexpected end-of-file was encountered.
    UnexpectedEOF,

    /// An unexpected end-of-line was encountered.
    UnexpectedEOL,

    /// TODO
    UnexpectedEscapeSequence,

    /// An unexpected right brace, }, was encountered.
    UnexpectedRightBrace,

    /// TODO
    UnexpectedContentAfterBlockName,

    /// Expected a left brace, {, but none was found.
    ExpectedLeftBrace,

    /// Expected a right brace, }, but none was found.
    ExpectedRightBrace,

    /// Expected a hash (#), but none was found
    ExpectedHash,

    /// Expected a space, but none was found
    ExpectedSpace,

    /// Expected an name char, but none was found.
    InvalidCharInName,
}

#[derive(Debug)]
pub struct ErrorWithContext<'a> {
    error: Error,
    pos: Pos,
    line0: Option<&'a str>,
    line1: Option<&'a str>,
}

impl<'a> fmt::Display for ErrorWithContext<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let color_red = "\u{1B}[31m";
        let color_reset = "\u{1B}[0m";

        write!(
            f,
            "parse error at line {}, column {}: #{:?}\n\n",
            self.pos.line_nr, self.pos.col_nr, self.error,
        );

        if let Some(line) = self.line0 {
            write!(f, "{}\n", line);
        }

        write!(f, "{}\n", self.line1.unwrap_or(""));

        write!(
            f,
            "{}{:>width$}{}",
            color_red,
            "↑",
            color_reset,
            width = self.pos.col_nr
        )
    }
}

#[derive(Debug)]
struct ParserContent {
    chars: Vec<char>,
    pos: Pos,
}

impl ParserContent {
    /// Move on to the next character.
    fn advance(&mut self) {
        if let Some('\n') = self.peek() {
            self.pos.advance(true);
        } else {
            self.pos.advance(false);
        }
    }

    /// Get the current character, without consuming it.
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos.idx).cloned()
    }

    /// Get the next character, without consuming it.
    fn peek2(&self) -> Option<char> {
        self.chars.get(self.pos.idx + 1).cloned()
    }

    /// Get the current character, and move on to the next.
    fn consume(&mut self) -> Result<char, Error> {
        let c = self.peek();
        self.advance();
        c.ok_or(Error::UnexpectedEOF)
    }

    fn try_consume_char(&mut self, expected_c: char) -> bool {
        let c = self.peek();
        match c {
            Some(c) if c == expected_c => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    fn is_eof(&self) -> bool {
        self.peek().is_none()
    }
}

#[derive(Debug)]
pub struct Parser {
    content: ParserContent,
}

impl Parser {
    pub fn new(s: &str) -> Self {
        Parser {
            content: ParserContent {
                chars: s.chars().collect(),
                pos: Pos::new(),
            },
        }
    }

    pub fn call(s: &str) -> Result<Vec<Node>, ErrorWithContext> {
        let mut parser = Parser::new(s);
        let res = parser.run();
        match res {
            Ok(parsed) => Ok(parsed),
            Err(error) => {
                let mut lines = s.lines();
                let mut line0;
                let mut line1;
                if parser.content.pos.line_nr > 0 {
                    let mut lines = lines.skip(parser.content.pos.line_nr - 1);
                    line0 = lines.next();
                    line1 = lines.next();
                } else {
                    line0 = None;
                    line1 = lines.next();
                }

                Err(ErrorWithContext {
                    error: error,
                    pos: parser.content.pos,
                    line0: line0,
                    line1: line1,
                })
            }
        }
    }

    pub fn run(&mut self) -> Result<Vec<Node>, Error> {
        // Skip blank lines
        loop {
            if self.content.is_eof() {
                break;
            }

            let blank_idx = self.try_read_blank_line();
            match blank_idx {
                Some(idx) => {
                    self.content.pos.idx = idx;
                    self.content.pos.line_nr += 1;
                    self.content.pos.col_nr = 0;
                }
                None => break,
            }
        }

        let mut nodes = vec![];
        loop {
            if self.content.is_eof() {
                break;
            }

            nodes.push(self.read_block_with_children(0)?);
        }

        Ok(nodes)
    }

    // Utility functions

    fn is_name_head_char(c: &char) -> bool {
        match c {
            'a'...'z' | 'A'...'Z' => true,
            _ => false,
        }
    }

    fn is_name_tail_char(c: &char) -> bool {
        match c {
            'a'...'z' | 'A'...'Z' | '-' | '_' | '0'...'9' => true,
            _ => false,
        }
    }

    // Utility functions – reading

    fn read_name_head(&mut self) -> Result<char, Error> {
        self.content
            .consume()
            .filter(Parser::is_name_head_char, Error::InvalidCharInName)
    }

    fn read_left_brace(&mut self) -> Result<char, Error> {
        self.content
            .consume()
            .filter(|c| *c == '{', Error::ExpectedLeftBrace)
    }

    fn read_right_brace(&mut self) -> Result<char, Error> {
        self.content
            .consume()
            .filter(|c| *c == '}', Error::ExpectedRightBrace)
    }

    fn read_hash(&mut self) -> Result<char, Error> {
        self.content
            .consume()
            .filter(|c| *c == '#', Error::ExpectedHash)
    }

    fn read_space(&mut self) -> Result<char, Error> {
        self.content
            .consume()
            .filter(|c| *c == ' ', Error::ExpectedSpace)
    }

    // Reading -- nodes

    fn read_block_element_node(&mut self) -> Result<ElementNode, Error> {
        self.read_hash()?;
        let name = self.read_name()?;
        let attributes = self.read_attributes()?;
        let mut children = vec![];

        match self.content.consume() {
            Err(_) => (),
            Ok('\n') => {}
            Ok(' ') => {
                let nodes = self.read_inline_nodes()?;
                self.read_end_of_inline_content()?;
                children.extend(nodes);
            }
            _ => return Err(Error::UnexpectedContentAfterBlockName),
        };

        Ok(ElementNode {
            name: name.into(),
            attributes: attributes,
            children: children,
        })
    }

    fn read_inline_element_node(&mut self) -> Result<Node, Error> {
        let name = self.read_name()?;
        let attributes = self.read_attributes()?;
        self.read_left_brace()?;
        let content = self.read_inline_nodes()?;
        self.read_right_brace()?;

        Ok(Node::Element(ElementNode {
            name: name.into(),
            attributes: attributes,
            children: content,
        }))
    }

    fn read_string_node(&mut self) -> Result<Node, Error> {
        let mut res = String::new();

        loop {
            let c = self.content.peek();
            match c {
                None => break,
                Some('\n') | Some('%') | Some('}') => break,
                Some(ch) => {
                    self.content.advance();
                    res.push(ch);
                }
            }
        }

        Ok(Node::String(StringNode {
            content: res.into(),
        }))
    }

    // Reading -- misc

    fn read_block_with_children(&mut self, indent: usize) -> Result<Node, Error> {
        let mut res = self.read_block_element_node()?;

        let mut pending_blanks = 0;
        while !self.content.is_eof() {
            let blank_idx = self.try_read_blank_line();
            match blank_idx {
                Some(idx) => {
                    self.content.pos.idx = idx;
                    self.content.pos.line_nr += 1;
                    self.content.pos.col_nr = 0;
                    pending_blanks += 1;
                }
                None => {
                    let sub_indentation = self.detect_indentation();
                    if sub_indentation < indent + 1 {
                        break;
                    }

                    self.read_indentation(indent + 1)?;
                    if self.try_read_block_start() {
                        res.children
                            .push(self.read_block_with_children(indent + 1)?)
                    } else {
                        if !res.children.is_empty() {
                            res.children.push(Node::String(StringNode {
                                content: "\n".into(),
                            }))
                        }

                        for _ in 0..pending_blanks {
                            res.children.push(Node::String(StringNode {
                                content: "\n".into(),
                            }));
                        }

                        pending_blanks = 0;
                        res.children.extend(self.read_inline_nodes()?);
                        self.read_end_of_inline_content()?;
                    }
                }
            }
        }

        Ok(Node::Element(res))
    }

    fn try_read_blank_line(&self) -> Option<usize> {
        let mut idx = self.content.pos.idx;

        loop {
            match self.content.chars.get(idx) {
                Some(' ') => idx += 1,
                None => break Some(idx + 1),
                Some('\n') => break Some(idx + 1),
                _ => break None,
            }
        }
    }

    fn read_indentation(&mut self, indent: usize) -> Result<(), Error> {
        for _ in 0..indent {
            self.read_space()?;
            self.read_space()?;
        }

        Ok(())
    }

    fn detect_indentation(&self) -> usize {
        let mut indentation_chars = 0;
        let mut idx = self.content.pos.idx;

        loop {
            match self.content.chars.get(idx) {
                Some(' ') => {
                    idx += 1;
                    indentation_chars += 1;
                }
                _ => break,
            }
        }

        indentation_chars / 2
    }

    fn try_read_block_start(&self) -> bool {
        match self.content.peek() {
            Some('#') => match self.content.peek2() {
                Some(c) if Parser::is_name_head_char(&c) => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn read_end_of_inline_content(&mut self) -> Result<(), Error> {
        match self.content.consume() {
            Err(_) | Ok('\n') => Ok(()),
            Ok('}') => Err(Error::UnexpectedRightBrace),
            _ => panic!("internal error: unexpected content after inline content"),
        }
    }

    fn read_inline_nodes(&mut self) -> Result<Vec<Node>, Error> {
        let mut res: Vec<Node> = vec![];

        while let Some(c) = self.content.peek() {
            match c {
                '\n' => break,
                '}' => break,
                '%' => res.push(self.read_percent_body()?),
                _ => res.push(self.read_string_node()?),
            }
        }

        Ok(res)
    }

    fn read_percent_body(&mut self) -> Result<Node, Error> {
        // Skip char that triggered this read
        self.content.advance();

        let c = self.content.peek().ok_or(Error::UnexpectedEOF)?;
        match c {
            '%' | '}' | '#' => self.read_escaped_char(),
            _ => self.read_inline_element_node(),
        }
    }

    fn read_escaped_char(&mut self) -> Result<Node, Error> {
        let c = self.content.peek().ok_or(Error::UnexpectedEOF)?;
        self.content.advance();
        Ok(Node::String(StringNode {
            content: c.to_string().into(),
        }))
    }

    fn read_name_tail_char(&mut self) -> Option<char> {
        let c = self.content.peek().filter(Parser::is_name_tail_char);
        if c.is_some() {
            self.content.advance();
        }
        c
    }

    fn read_name(&mut self) -> Result<String, Error> {
        let mut res = String::new();

        res.push(self.read_name_head()?);
        while let Some(c) = self.read_name_tail_char() {
            res.push(c);
        }
        Ok(res)
    }

    fn read_attribute_key(&mut self) -> Result<String, Error> {
        self.read_name()
    }

    fn read_attribute_value(&mut self) -> Result<String, Error> {
        let mut res = String::new();

        loop {
            let c = self.content.peek().ok_or(Error::UnexpectedEOF)?;
            match c {
                '%' => {
                    self.content.advance();
                    let c2 = self.content.peek().ok_or(Error::UnexpectedEOF)?;
                    match c2 {
                        '%' | ']' | ',' => {
                            self.content.advance();
                            res.push(c2);
                        }
                        '\n' => return Err(Error::UnexpectedEOL),
                        _ => return Err(Error::UnexpectedEscapeSequence),
                    }
                }

                ']' | ',' => break,

                '\n' => return Err(Error::UnexpectedEOL),

                _ => {
                    self.content.advance();
                    res.push(c);
                }
            }
        }

        Ok(res)
    }

    fn read_attributes(&mut self) -> Result<HashMap<String, String>, Error> {
        let mut attributes = HashMap::new();

        if !self.content.try_consume_char('[') {
            return Ok(attributes);
        }

        if self.content.try_consume_char(']') {
            return Ok(attributes);
        }

        loop {
            let key = self.read_attribute_key()?;

            if self.content.try_consume_char('=') {
                attributes.insert(key, self.read_attribute_value()?);
            } else {
                attributes.insert(key.clone(), key);
            }

            match self.content.consume()? {
                ']' => break,
                ',' => (),
                _ => panic!("internal error: unexpected content after attribute value"),
            }
        }

        Ok(attributes)
    }
}
