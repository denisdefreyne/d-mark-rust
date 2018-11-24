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

mod parser;
mod translator;
mod util;

use std::borrow::Cow;
use std::collections::HashMap;

pub use parser::Parser;
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
