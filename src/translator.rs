use super::{ElementNode, Node, StringNode};

pub trait Translator {
    fn translate(&self, node: &Node) -> String {
        match node {
            Node::Element(n) => self.translate_element(n),
            Node::String(n) => self.translate_string(n),
        }
    }

    fn translate_element(&self, node: &ElementNode) -> String;
    fn translate_string(&self, node: &StringNode) -> String;
}

#[cfg(test)]
mod tests {
    use super::Translator;
    use super::{ElementNode, Node, StringNode};
    use std::collections::HashMap;

    struct SimpleTranslator {}

    impl Translator for SimpleTranslator {
        fn translate_element(&self, node: &ElementNode) -> String {
            format!(
                "elem(name={:?}, attrs={:?}, children=[{}])",
                node.name,
                node.attributes,
                node.children
                    .iter()
                    .map(|c| self.translate(c))
                    .collect::<Vec<String>>()
                    .join(","),
            )
        }

        fn translate_string(&self, node: &StringNode) -> String {
            format!("str({:?})", node.content)
        }
    }

    #[test]
    fn simple() {
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
            SimpleTranslator {}.translate(&input),
            "elem(name=\"root-elem\", attrs={\"foo\": \"bar\"}, children=[str(\"child-str\")])"
                .to_string()
        );
    }
}
