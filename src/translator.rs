use super::{ElementNode, Node, StringNode};

pub trait Translator<T> {
    fn translate(&self, node: &Node) -> T {
        match node {
            Node::Element(n) => self.translate_element(n),
            Node::String(n) => self.translate_string(n),
        }
    }

    fn translate_element(&self, node: &ElementNode) -> T;
    fn translate_string(&self, node: &StringNode) -> T;
}

#[cfg(test)]
mod tests {
    use super::Translator;
    use super::{ElementNode, Node, StringNode};
    use std::collections::HashMap;

    struct SampleStringTranslator {}

    impl Translator<String> for SampleStringTranslator {
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

    #[derive(PartialEq, Debug)]
    enum TreeNode {
        Elem(String, Vec<TreeNode>),
        Str(String),
    }

    struct SampleTreeTranslator {}

    impl Translator<TreeNode> for SampleTreeTranslator {
        fn translate_element(&self, node: &ElementNode) -> TreeNode {
            TreeNode::Elem(
                node.name.to_owned().to_string(),
                node.children.iter().map(|c| self.translate(c)).collect(),
            )
        }

        fn translate_string(&self, node: &StringNode) -> TreeNode {
            TreeNode::Str(node.content.to_owned().to_string())
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
            SampleStringTranslator {}.translate(&input),
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
            SampleTreeTranslator {}.translate(&input),
            TreeNode::Elem(
                "root-elem".to_string(),
                vec![TreeNode::Str("child-str".into())]
            )
        );
    }
}
