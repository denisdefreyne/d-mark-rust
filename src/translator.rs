use super::{ElementNode, Node, StringNode};

pub trait Translator {
    fn translate(&self, nodes: Vec<Node>) -> String {
        let mut res = String::new();
        for node in nodes {
            res.push_str(&match node {
                Node::Element(n) => self.translate_element(n),
                Node::String(n) => self.translate_string(n),
            })
        }
        res
    }

    fn translate_element(&self, node: ElementNode) -> String;
    fn translate_string(&self, node: StringNode) -> String;
}

#[cfg(test)]
mod tests {
    use super::Translator;
    use super::{ElementNode, Node, StringNode};
    use std::collections::HashMap;

    struct SimpleTranslator {}

    impl Translator for SimpleTranslator {
        fn translate_element(&self, node: ElementNode) -> String {
            format!(
                "elem(name={:?}, attrs={:?}, children=[{}])",
                node.name,
                node.attributes,
                self.translate(node.children),
            )
        }

        fn translate_string(&self, node: StringNode) -> String {
            format!("str({:?})", node.content)
        }
    }

    #[test]
    fn simple() {
        let mut attrs = HashMap::new();
        attrs.insert("foo".into(), "bar".into());

        let input = vec![
            Node::Element(ElementNode {
                name: "root-elem".into(),
                attributes: attrs,
                children: vec![Node::String(StringNode {
                    content: "child-str".into(),
                })],
            }),
            Node::String(StringNode {
                content: "root-str".into(),
            }),
        ];
        assert_eq!(
            SimpleTranslator {}.translate(input),
            "elem(name=\"root-elem\", attrs={\"foo\": \"bar\"}, children=[str(\"child-str\")])str(\"root-str\")".to_string()
        );
    }
}
