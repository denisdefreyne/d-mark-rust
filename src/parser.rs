use super::util::FilterableResult;
use super::{ElementNode, Node, StringNode};

use std::collections::HashMap;
use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::{ElementNode, Error, Node, Parser, StringNode};
    use std::collections::HashMap;

    #[test]
    fn parse_inline_string() {
        assert_eq!(
            Parser::new(&"#p hai").run().unwrap(),
            vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![Node::String(StringNode {
                    content: "hai".into()
                })]
            })]
        );
    }

    #[test]
    fn parse_inline_string_empty() {
        assert_eq!(
            Parser::new(&"#p ").run().unwrap(),
            vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![]
            })]
        );
    }

    #[test]
    fn parse_inline_element_empty() {
        assert_eq!(
            Parser::new(&"#p %foo{}").run().unwrap(),
            vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![Node::Element(ElementNode {
                    name: "foo".into(),
                    attributes: HashMap::new(),
                    children: vec![]
                })]
            })]
        );
    }

    #[test]
    fn parse_inline_element_str() {
        assert_eq!(
            Parser::new(&"#p %foo{abc}").run().unwrap(),
            vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![Node::Element(ElementNode {
                    name: "foo".into(),
                    attributes: HashMap::new(),
                    children: vec![Node::String(StringNode {
                        content: "abc".into()
                    })]
                })]
            })]
        );
    }

    #[test]
    fn parse_inline_element_wrapped() {
        assert_eq!(
            Parser::new(&"#p alpha %foo{abc} omega").run().unwrap(),
            vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "alpha ".into()
                    }),
                    Node::Element(ElementNode {
                        name: "foo".into(),
                        attributes: HashMap::new(),
                        children: vec![Node::String(StringNode {
                            content: "abc".into()
                        })]
                    }),
                    Node::String(StringNode {
                        content: " omega".into()
                    }),
                ]
            })]
        );
    }

    #[test]
    fn parse_inline_element_nested() {
        assert_eq!(
            Parser::new(&"#p %foo{%bar{}}").run().unwrap(),
            vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![Node::Element(ElementNode {
                    name: "foo".into(),
                    attributes: HashMap::new(),
                    children: vec![Node::Element(ElementNode {
                        name: "bar".into(),
                        attributes: HashMap::new(),
                        children: vec![]
                    })]
                })]
            })]
        );
    }

    #[test]
    fn parse_inline_element_escaped() {
        assert_eq!(
            Parser::new(&"#p a %% b").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "a ".into()
                    }),
                    Node::String(StringNode {
                        content: "%".into()
                    }),
                    Node::String(StringNode {
                        content: " b".into()
                    })
                ]
            })])
        );
    }

    #[test]
    fn parse_inline_element_eof1() {
        assert_eq!(Parser::new(&"#p a %").run(), Err(Error::UnexpectedEOF));
    }

    #[test]
    fn parse_inline_element_eof2() {
        assert_eq!(Parser::new(&"#p a %a").run(), Err(Error::UnexpectedEOF));
    }

    #[test]
    fn parse_inline_element_eof3() {
        assert_eq!(Parser::new(&"#p a %a{").run(), Err(Error::UnexpectedEOF));
    }

    #[test]
    fn parse_inline_element_nl1() {
        assert_eq!(
            Parser::new(&"#p a %\nb{}").run(),
            Err(Error::InvalidCharInName)
        );
    }

    #[test]
    fn parse_inline_element_nl2() {
        assert_eq!(
            Parser::new(&"#p a %a\nb{}").run(),
            Err(Error::ExpectedLeftBrace)
        );
    }

    #[test]
    fn parse_inline_element_nl3() {
        assert_eq!(
            Parser::new(&"#p a %a{\nb}").run(),
            Err(Error::ExpectedRightBrace)
        );
    }

    #[test]
    fn parse_inline_element_nl4() {
        assert_eq!(
            Parser::new(&"#p a %a{b\n}").run(),
            Err(Error::ExpectedRightBrace)
        );
    }

    #[test]
    fn parse_inline_attr_empty() {
        assert_eq!(
            Parser::new(&"#p foo %aaa[]{stuff} bar").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "foo ".into()
                    }),
                    Node::Element(ElementNode {
                        name: "aaa".into(),
                        attributes: HashMap::new(),
                        children: vec![Node::String(StringNode {
                            content: "stuff".into()
                        })],
                    }),
                    Node::String(StringNode {
                        content: " bar".into()
                    })
                ]
            })])
        );
    }

    #[test]
    fn parse_inline_attr_single_pair() {
        let mut attributes = HashMap::new();
        attributes.insert("x".to_string(), "1".to_string());

        assert_eq!(
            Parser::new(&"#p foo %aaa[x=1]{stuff} bar").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "foo ".into()
                    }),
                    Node::Element(ElementNode {
                        name: "aaa".into(),
                        attributes: attributes,
                        children: vec![Node::String(StringNode {
                            content: "stuff".into()
                        })],
                    }),
                    Node::String(StringNode {
                        content: " bar".into()
                    })
                ]
            })])
        );
    }

    #[test]
    fn parse_inline_attr_just_key() {
        let mut attributes = HashMap::new();
        attributes.insert("static".to_string(), "static".to_string());

        assert_eq!(
            Parser::new(&"#p foo %aaa[static]{stuff} bar").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "foo ".into()
                    }),
                    Node::Element(ElementNode {
                        name: "aaa".into(),
                        attributes: attributes,
                        children: vec![Node::String(StringNode {
                            content: "stuff".into()
                        })],
                    }),
                    Node::String(StringNode {
                        content: " bar".into()
                    }),
                ]
            })])
        );
    }

    #[test]
    fn parse_inline_attr_escape_percentage() {
        let mut attributes = HashMap::new();
        attributes.insert("x".to_string(), "a%b".to_string());

        assert_eq!(
            Parser::new(&"#p foo %aaa[x=a%%b]{stuff} bar").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "foo ".into()
                    }),
                    Node::Element(ElementNode {
                        name: "aaa".into(),
                        attributes: attributes,
                        children: vec![Node::String(StringNode {
                            content: "stuff".into()
                        })],
                    }),
                    Node::String(StringNode {
                        content: " bar".into()
                    }),
                ]
            })])
        );
    }

    #[test]
    fn parse_inline_attr_escape_comma() {
        let mut attributes = HashMap::new();
        attributes.insert("x".to_string(), "a,b".to_string());

        assert_eq!(
            Parser::new(&"#p foo %aaa[x=a%,b]{stuff} bar").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "foo ".into()
                    }),
                    Node::Element(ElementNode {
                        name: "aaa".into(),
                        attributes: attributes,
                        children: vec![Node::String(StringNode {
                            content: "stuff".into()
                        })],
                    }),
                    Node::String(StringNode {
                        content: " bar".into()
                    }),
                ]
            })])
        );
    }

    #[test]
    fn parse_inline_attr_escape_other() {
        assert_eq!(
            Parser::new(&"#p foo %aaa[x=a%?b]{stuff} bar").run(),
            Err(Error::UnexpectedEscapeSequence),
        );
    }

    #[test]
    fn parse_inline_attr_escape_rbracket() {
        let mut attributes = HashMap::new();
        attributes.insert("x".to_string(), "a]b".to_string());

        assert_eq!(
            Parser::new(&"#p foo %aaa[x=a%]b]{stuff} bar").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "foo ".into()
                    }),
                    Node::Element(ElementNode {
                        name: "aaa".into(),
                        attributes: attributes,
                        children: vec![Node::String(StringNode {
                            content: "stuff".into()
                        })],
                    }),
                    Node::String(StringNode {
                        content: " bar".into()
                    }),
                ]
            })])
        )
    }

    #[test]
    fn parse_inline_attr_escape_eol() {
        assert_eq!(
            Parser::new(&"#p foo %aaa[x=a%\n]b]{stuff} bar").run(),
            Err(Error::UnexpectedEOL),
        );
    }

    #[test]
    fn parse_inline_attr_escape_eof() {
        assert_eq!(
            Parser::new(&"#p foo %aaa[x=a%").run(),
            Err(Error::UnexpectedEOF),
        );
    }

    #[test]
    fn parse_inline_attr_early_eof() {
        assert_eq!(
            Parser::new(&"#p foo %aaa[x=a").run(),
            Err(Error::UnexpectedEOF),
        );
    }

    #[test]
    fn parse_inline_attr_early_eol() {
        assert_eq!(
            Parser::new(&"#p foo %aaa[x=a\n").run(),
            Err(Error::UnexpectedEOL),
        );
    }

    #[test]
    fn parse_block_empty() {
        assert_eq!(Parser::new(&"").run(), Ok(vec![]),);
    }

    #[test]
    fn parse_block_one_empty_el() {
        assert_eq!(
            Parser::new(&"#p").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![],
            })]),
        );
    }

    #[test]
    fn parse_block_one_empty_el_with_space() {
        assert_eq!(
            Parser::new(&"#p ").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_without_space() {
        assert_eq!(
            Parser::new(&"#p%a{b}").run(),
            Err(Error::UnexpectedContentAfterBlockName),
        );
    }

    #[test]
    fn parse_block_one_el_with_string() {
        assert_eq!(
            Parser::new(&"#p hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_with_string_with_escaped_percent() {
        assert_eq!(
            Parser::new(&"#p hi %%").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "hi ".into()
                    }),
                    Node::String(StringNode {
                        content: "%".into()
                    })
                ],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_with_string_with_escaped_rbrace() {
        assert_eq!(
            Parser::new(&"#p hi %}").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "hi ".into()
                    }),
                    Node::String(StringNode {
                        content: "}".into()
                    })
                ],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_name_with_dash() {
        assert_eq!(
            Parser::new(&"#intro-para hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "intro-para".into(),
                attributes: HashMap::new(),
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_name_with_underscore() {
        assert_eq!(
            Parser::new(&"#intro_para hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "intro_para".into(),
                attributes: HashMap::new(),
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_name_with_uppercase() {
        assert_eq!(
            Parser::new(&"#introPara hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "introPara".into(),
                attributes: HashMap::new(),
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_attr_empty() {
        assert_eq!(
            Parser::new(&"#foo[] hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "foo".into(),
                attributes: HashMap::new(),
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_attr_simple() {
        let mut attributes = HashMap::new();
        attributes.insert("abc".to_string(), "xyz".to_string());

        assert_eq!(
            Parser::new(&"#foo[abc=xyz] hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "foo".into(),
                attributes: attributes,
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_attr_key_with_dash() {
        let mut attributes = HashMap::new();
        attributes.insert("intended-audience".to_string(), "learner".to_string());

        assert_eq!(
            Parser::new(&"#foo[intended-audience=learner] hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "foo".into(),
                attributes: attributes,
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_attr_key_with_underscore() {
        let mut attributes = HashMap::new();
        attributes.insert("intended_audience".to_string(), "learner".to_string());

        assert_eq!(
            Parser::new(&"#foo[intended_audience=learner] hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "foo".into(),
                attributes: attributes,
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_attr_key_with_uppercase() {
        let mut attributes = HashMap::new();
        attributes.insert("intendedAudience".to_string(), "learner".to_string());

        assert_eq!(
            Parser::new(&"#foo[intendedAudience=learner] hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "foo".into(),
                attributes: attributes,
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_attr_key_with_number() {
        let mut attributes = HashMap::new();
        attributes.insert("over-9000".to_string(), "yes".to_string());

        assert_eq!(
            Parser::new(&"#foo[over-9000=yes] hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "foo".into(),
                attributes: attributes,
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_attr_without_value() {
        let mut attributes = HashMap::new();
        attributes.insert("foo".to_string(), "foo".to_string());

        assert_eq!(
            Parser::new(&"#p[foo] hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: attributes,
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_attrs_simple() {
        let mut attributes = HashMap::new();
        attributes.insert("foo".to_string(), "one".to_string());
        attributes.insert("bar".to_string(), "two".to_string());

        assert_eq!(
            Parser::new(&"#p[foo=one,bar=two] hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: attributes,
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_attrs_without_value() {
        let mut attributes = HashMap::new();
        attributes.insert("foo".to_string(), "foo".to_string());
        attributes.insert("bar".to_string(), "bar".to_string());

        assert_eq!(
            Parser::new(&"#p[foo,bar] hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: attributes,
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_attrs_escaped() {
        let mut attributes = HashMap::new();
        attributes.insert("foo".to_string(), "]".to_string());
        attributes.insert("bar".to_string(), "%".to_string());
        attributes.insert("donkey".to_string(), ",".to_string());

        assert_eq!(
            Parser::new(&"#p[foo=%],bar=%%,donkey=%,] hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: attributes,
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_attr_key_starts_with_dash() {
        assert_eq!(
            Parser::new(&"#p[-foo=abc] hi").run(),
            Err(Error::InvalidCharInName),
        );
    }

    #[test]
    fn parse_block_one_el_attr_key_starts_with_underscore() {
        assert_eq!(
            Parser::new(&"#p[_foo=abc] hi").run(),
            Err(Error::InvalidCharInName),
        );
    }

    #[test]
    fn parse_block_one_el_attr_key_starts_with_num() {
        assert_eq!(
            Parser::new(&"#p[1foo=abc] hi").run(),
            Err(Error::InvalidCharInName),
        );
    }

    #[test]
    fn parse_block_one_el_attr_value_has_unescaped_percent() {
        assert_eq!(
            Parser::new(&"#p %ref[url=https://github.com/?q=user%3Ananoc]{eek}").run(),
            Err(Error::UnexpectedEscapeSequence),
        );
    }

    #[test]
    fn parse_block_one_el_attr_early_eof() {
        assert_eq!(
            Parser::new(&"#p %ref[url=hello").run(),
            Err(Error::UnexpectedEOF),
        );
    }

    #[test]
    fn parse_block_one_el_attr_early_eof_escape() {
        assert_eq!(
            Parser::new(&"#p %ref[url=hello%").run(),
            Err(Error::UnexpectedEOF),
        );
    }

    #[test]
    fn parse_block_one_el_early_eof_escape() {
        assert_eq!(Parser::new(&"#p %").run(), Err(Error::UnexpectedEOF),);
    }

    #[test]
    fn parse_block_one_el_unexpected_rbrace() {
        assert_eq!(Parser::new(&"#p }").run(), Err(Error::UnexpectedRightBrace),);
    }

    #[test]
    fn parse_block_one_el_continued_content1() {
        assert_eq!(
            Parser::new(&"#p\n  hi").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![Node::String(StringNode {
                    content: "hi".into()
                })],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_continued_content2() {
        assert_eq!(
            Parser::new(&"#p\n  hi\n  ho").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "hi".into()
                    }),
                    Node::String(StringNode {
                        content: "\n".into()
                    }),
                    Node::String(StringNode {
                        content: "ho".into()
                    })
                ],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_continued_content3() {
        assert_eq!(
            Parser::new(&"#p hi\n  ho").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "hi".into()
                    }),
                    Node::String(StringNode {
                        content: "\n".into()
                    }),
                    Node::String(StringNode {
                        content: "ho".into()
                    })
                ],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_continued_content4() {
        assert_eq!(
            Parser::new(&"#p hi\n    ho").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "hi".into()
                    }),
                    Node::String(StringNode {
                        content: "\n".into()
                    }),
                    Node::String(StringNode {
                        content: "  ho".into()
                    })
                ],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_continued_content5() {
        assert_eq!(
            Parser::new(&"#p hi\n    ho\n  ha").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "hi".into()
                    }),
                    Node::String(StringNode {
                        content: "\n".into()
                    }),
                    Node::String(StringNode {
                        content: "  ho".into()
                    }),
                    Node::String(StringNode {
                        content: "\n".into()
                    }),
                    Node::String(StringNode {
                        content: "ha".into()
                    })
                ],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_continued_content_nested() {
        assert_eq!(
            Parser::new(&"#p hi\n  %#foo").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "hi".into()
                    }),
                    Node::String(StringNode {
                        content: "\n".into()
                    }),
                    Node::String(StringNode {
                        content: "#".into()
                    }),
                    Node::String(StringNode {
                        content: "foo".into()
                    })
                ],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_continued_content_hash_but_no_block() {
        assert_eq!(
            Parser::new(&"#listing\n  calc_foo()\n  # => 123").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "listing".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "calc_foo()".into()
                    }),
                    Node::String(StringNode {
                        content: "\n".into()
                    }),
                    Node::String(StringNode {
                        content: "# => 123".into()
                    })
                ],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_nested1() {
        assert_eq!(
            Parser::new(&"#p hi\n  #x a").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "hi".into()
                    }),
                    Node::Element(ElementNode {
                        name: "x".into(),
                        attributes: HashMap::new(),
                        children: vec![Node::String(StringNode {
                            content: "a".into()
                        })],
                    })
                ],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_nested2() {
        assert_eq!(
            Parser::new(&"#p\n  hi\n  #x a").run(),
            Ok(vec![Node::Element(ElementNode {
                name: "p".into(),
                attributes: HashMap::new(),
                children: vec![
                    Node::String(StringNode {
                        content: "hi".into()
                    }),
                    Node::Element(ElementNode {
                        name: "x".into(),
                        attributes: HashMap::new(),
                        children: vec![Node::String(StringNode {
                            content: "a".into()
                        })],
                    })
                ],
            })]),
        );
    }

    #[test]
    fn parse_block_one_el_garbage_at_eol() {
        assert_eq!(
            Parser::new(&"#p hi}").run(),
            Err(Error::UnexpectedRightBrace),
        );
    }

    #[test]
    fn parse_block_two_els_simple() {
        assert_eq!(
            Parser::new(&"#p hi\n#p ho").run(),
            Ok(vec![
                Node::Element(ElementNode {
                    name: "p".into(),
                    attributes: HashMap::new(),
                    children: vec![Node::String(StringNode {
                        content: "hi".into()
                    })],
                }),
                Node::Element(ElementNode {
                    name: "p".into(),
                    attributes: HashMap::new(),
                    children: vec![Node::String(StringNode {
                        content: "ho".into()
                    })],
                })
            ]),
        );
    }

    #[test]
    fn parse_block_two_els_continued() {
        assert_eq!(
            Parser::new(&"#p hi\n  hi2\n#p ho\n  ho2").run(),
            Ok(vec![
                Node::Element(ElementNode {
                    name: "p".into(),
                    attributes: HashMap::new(),
                    children: vec![
                        Node::String(StringNode {
                            content: "hi".into()
                        }),
                        Node::String(StringNode {
                            content: "\n".into()
                        }),
                        Node::String(StringNode {
                            content: "hi2".into()
                        })
                    ],
                }),
                Node::Element(ElementNode {
                    name: "p".into(),
                    attributes: HashMap::new(),
                    children: vec![
                        Node::String(StringNode {
                            content: "ho".into()
                        }),
                        Node::String(StringNode {
                            content: "\n".into()
                        }),
                        Node::String(StringNode {
                            content: "ho2".into()
                        })
                    ],
                })
            ]),
        );
    }

    /*
    expect(parse('#p %%')).to eq [
    expect(parse('#p %}')).to eq [
    expect(parse('#p %foo{%%}')).to eq [
    expect(parse('#p %foo{%}}')).to eq [
    expect(parse('#p hi %em{ho}')).to eq [
    expect(parse("#p hi %em{ho}\n")).to eq [
    expect(parse("#p hi %em{ho}\n  #p child p")).to eq [
    expect(parse("#p hi %em{ho}\n  #p child p\n    #p subchild p")).to eq [
    expect(parse("#p foo\n \n  #p bar\n  \n\n    #p qux")).to eq [
    expect(parse("#p foo\n#p bar")).to eq [
    expect(parse("#p foo\n  donkey")).to eq [
    expect(parse("#p foo\n    donkey")).to eq [
    expect(parse("#p foo\n    donkey\n  giraffe\n    zebra\n")).to eq [
    expect(parse("#p foo\n\n  donkey\n\n    giraffe\n")).to eq [
    expect(parse("#p\n  donkey\n")).to eq [
    expect(parse("#p\n  %em{donkey}")).to eq [
    expect(parse("#p[foo=bar]\n  hi")).to eq [
    expect(parse("#p\n  this is not a child block.")).to eq [
    expect(parse("#p\n  foo.bar")).to eq [
    expect(parse("#ul\n  #li\n    #p You can.")).to eq [
    expect(parse("#ul\n  #li")).to eq [
    expect(parse("#ul\n  #li[foo]")).to eq [
    expect(parse("  \n \n#p Hi!")).to eq [
    expect(parse("#listing\n  %#h1 Foo\n")).to eq [
    expect(parse("#listing\n  %#h1[donkey] Foo\n")).to eq [
    */
}
