use super::{ElementNode, Node, StringNode};

pub trait Translator<T, C> {
    fn translate(&self, node: &Node, context: C) -> T {
        match node {
            Node::Element(n) => self.translate_element(n, context),
            Node::String(n) => self.translate_string(n, context),
        }
    }

    fn translate_element(&self, node: &ElementNode, context: C) -> T;
    fn translate_string(&self, node: &StringNode, context: C) -> T;
}

#[cfg(test)]
mod tests {
    use super::Translator;
    use super::{ElementNode, Node, StringNode};
    use std::collections::HashMap;

    struct SampleStringTranslator {}

    impl Translator<String, ()> for SampleStringTranslator {
        fn translate_element(&self, node: &ElementNode, context: ()) -> String {
            format!(
                "elem(name={:?}, attrs={:?}, children=[{}])",
                node.name,
                node.attributes,
                node.children
                    .iter()
                    .map(|c| self.translate(c, context))
                    .collect::<Vec<String>>()
                    .join(","),
            )
        }

        fn translate_string(&self, node: &StringNode, _context: ()) -> String {
            format!("str({:?})", node.content)
        }
    }

    #[derive(PartialEq, Debug)]
    enum TreeNode {
        Elem(String, Vec<TreeNode>),
        Str(String),
    }

    struct SampleTreeTranslator {}

    impl Translator<TreeNode, ()> for SampleTreeTranslator {
        fn translate_element(&self, node: &ElementNode, context: ()) -> TreeNode {
            TreeNode::Elem(
                node.name.to_owned().to_string(),
                node.children
                    .iter()
                    .map(|c| self.translate(c, context))
                    .collect(),
            )
        }

        fn translate_string(&self, node: &StringNode, _context: ()) -> TreeNode {
            TreeNode::Str(node.content.to_owned().to_string())
        }
    }

    struct SampleNestedTranslator {}

    impl Translator<String, u8> for SampleNestedTranslator {
        fn translate_element(&self, node: &ElementNode, context: u8) -> String {
            match node.name.as_ref() {
                "section" => node
                    .children
                    .iter()
                    .map(|c| self.translate(c, context + 1))
                    .collect::<Vec<String>>()
                    .join(""),
                "header" => format!(
                    "<h{}>{}</h{}>",
                    context,
                    node.children
                        .iter()
                        .map(|c| self.translate(c, context + 1))
                        .collect::<Vec<String>>()
                        .join(""),
                    context
                ),
                _ => panic!("unknown element"),
            }
        }

        fn translate_string(&self, node: &StringNode, _context: u8) -> String {
            node.content.to_owned().to_string()
        }
    }

    #[test]
    fn example_string() {
        let mut attrs = HashMap::new();
        attrs.insert("foo".into(), "bar".into());

        let input = Node::Element(ElementNode {
            name: "root-elem".into(),
            attributes: attrs,
            children: vec![Node::String(StringNode {
                content: "child-str".into(),
            })],
        });
        assert_eq!(
            SampleStringTranslator {}.translate(&input, ()),
            "elem(name=\"root-elem\", attrs={\"foo\": \"bar\"}, children=[str(\"child-str\")])"
                .to_string()
        );
    }

    #[test]
    fn example_tree() {
        let mut attrs = HashMap::new();
        attrs.insert("foo".into(), "bar".into());

        let input = Node::Element(ElementNode {
            name: "root-elem".into(),
            attributes: attrs,
            children: vec![Node::String(StringNode {
                content: "child-str".into(),
            })],
        });
        assert_eq!(
            SampleTreeTranslator {}.translate(&input, ()),
            TreeNode::Elem(
                "root-elem".to_string(),
                vec![TreeNode::Str("child-str".into())]
            )
        );
    }

    #[test]
    fn example_context() {
        let input = Node::Element(ElementNode {
            name: "section".into(),
            attributes: HashMap::new(),
            children: vec![
                Node::Element(ElementNode {
                    name: "header".into(),
                    attributes: HashMap::new(),
                    children: vec![Node::String(StringNode {
                        content: "foo".into(),
                    })],
                }),
                Node::Element(ElementNode {
                    name: "section".into(),
                    attributes: HashMap::new(),
                    children: vec![Node::Element(ElementNode {
                        name: "header".into(),
                        attributes: HashMap::new(),
                        children: vec![Node::String(StringNode {
                            content: "bar".into(),
                        })],
                    })],
                }),
            ],
        });

        assert_eq!(
            SampleNestedTranslator {}.translate(&input, 0),
            "<h1>foo</h1><h2>bar</h2>".to_string()
        );
    }
}
