#![cfg(test)]

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
