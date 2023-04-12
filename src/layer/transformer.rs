use std::borrow::Cow;

use crate::model::{html::*, tree::*};

#[derive(Debug, Clone)]
pub struct Transformer {
    section: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for Transformer {
    fn default() -> Self {
        Self { section: false }
    }
}

impl Transformer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn section(mut self, section: bool) -> Self {
        self.section = section;
        self
    }
}

impl Transformer {
    pub fn transform<'a>(&self, tree: MarkdownTree<'a>) -> DocumentNode<'a> {
        DocumentNode {
            root: self.block_tree(tree.root),
        }
    }

    fn block_tree<'a>(&self, tree: BlockTree<'a>) -> Vec<Node<'a>> {
        tree.root
            .into_iter()
            .map(|item| self.block_item(item))
            .collect()
    }

    fn block_item<'a>(&self, item: BlockItem<'a>) -> Node<'a> {
        match item {
            BlockItem::Paragraph(tree) => self.paragraph(tree),
            BlockItem::Headline(level, tree) => self.headline(level, tree),
            BlockItem::BulletList(tree) => self.bullet_list(tree),
            BlockItem::OrderedList(tree) => self.ordered_list(tree),
            BlockItem::BlockQuote(tree) => self.blockquote(tree),
            BlockItem::Container(_, _) => todo!(),
        }
    }

    fn paragraph<'a>(&self, tree: InlineTree<'a>) -> Node<'a> {
        Node::Element(ElementNode {
            tag: ElementTag::P,
            id: vec![],
            class: vec![],
            children: self.inline_tree(tree),
        })
    }

    fn headline<'a>(&self, level: u8, tree: InlineTree<'a>) -> Node<'a> {
        Node::Element(ElementNode {
            tag: ElementTag::headline(level).unwrap(),
            id: vec![],
            class: vec![],
            children: self.inline_tree(tree),
        })
    }

    fn bullet_list<'a>(&self, tree: ListTree<'a>) -> Node<'a> {
        Node::Element(ElementNode {
            tag: ElementTag::Ul,
            id: vec![],
            class: vec![],
            children: self.list_tree(tree),
        })
    }

    fn ordered_list<'a>(&self, tree: ListTree<'a>) -> Node<'a> {
        Node::Element(ElementNode {
            tag: ElementTag::Ol,
            id: vec![],
            class: vec![],
            children: self.list_tree(tree),
        })
    }

    fn blockquote<'a>(&self, tree: BlockTree<'a>) -> Node<'a> {
        Node::Element(ElementNode {
            tag: ElementTag::Blockquote,
            id: vec![],
            class: vec![],
            children: self.block_tree(tree),
        })
    }

    fn list_tree<'a>(&self, tree: ListTree<'a>) -> Vec<Node<'a>> {
        tree.root
            .into_iter()
            .map(|item| {
                let mut nodes = self.inline_tree(item.name);

                item.children
                    .into_iter()
                    .map(|item| self.block_item(item))
                    .for_each(|node| nodes.push(node));

                Node::Element(ElementNode {
                    tag: ElementTag::Li,
                    id: vec![],
                    class: vec![],
                    children: nodes,
                })
            })
            .collect()
    }

    fn inline_tree<'a>(&self, tree: InlineTree<'a>) -> Vec<Node<'a>> {
        tree.root
            .into_iter()
            .map(|item| self.inline_item(item))
            .collect()
    }

    fn inline_item<'a>(&self, item: InlineItem<'a>) -> Node<'a> {
        match item {
            InlineItem::Text(text) => self.text(text),
            InlineItem::Italic(tree) => self.italic(tree),
            InlineItem::Strong(tree) => self.strong(tree),
            InlineItem::Break => self.r#break(),
        }
    }

    fn text<'a>(&self, text: Cow<'a, str>) -> Node<'a> {
        Node::Text(TextNode { text })
    }

    fn italic<'a>(&self, tree: InlineTree<'a>) -> Node<'a> {
        Node::Element(ElementNode {
            tag: ElementTag::Em,
            id: vec![],
            class: vec![],
            children: self.inline_tree(tree),
        })
    }

    fn strong<'a>(&self, tree: InlineTree<'a>) -> Node<'a> {
        Node::Element(ElementNode {
            tag: ElementTag::Strong,
            id: vec![],
            class: vec![],
            children: self.inline_tree(tree),
        })
    }

    fn r#break<'a>(&self) -> Node<'a> {
        Node::Element(ElementNode {
            tag: ElementTag::Br,
            id: vec![],
            class: vec![],
            children: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform() {
        // # Hello
        // World
        //
        // Hello2 *World2*
        let tree = MarkdownTree {
            root: BlockTree {
                root: vec![
                    BlockItem::Headline(
                        1,
                        InlineTree {
                            root: vec![
                                InlineItem::Text(Cow::Borrowed("Hello")),
                                InlineItem::Break,
                                InlineItem::Text(Cow::Borrowed("World")),
                            ],
                        },
                    ),
                    BlockItem::Paragraph(InlineTree {
                        root: vec![InlineItem::Strong(InlineTree {
                            root: vec![InlineItem::Text(Cow::Borrowed("Hello World2"))],
                        })],
                    }),
                ],
            },
        };

        let transformer = Transformer::new();
        let document = transformer.transform(tree);

        assert_eq!(
            document,
            DocumentNode {
                root: vec![
                    Node::Element(ElementNode {
                        tag: ElementTag::H1,
                        id: vec![],
                        class: vec![],
                        children: vec![
                            Node::Text(TextNode {
                                text: Cow::Borrowed("Hello")
                            }),
                            Node::Element(ElementNode {
                                tag: ElementTag::Br,
                                id: vec![],
                                class: vec![],
                                children: vec![]
                            }),
                            Node::Text(TextNode {
                                text: Cow::Borrowed("World")
                            }),
                        ]
                    }),
                    Node::Element(ElementNode {
                        tag: ElementTag::P,
                        id: vec![],
                        class: vec![],
                        children: vec![Node::Element(ElementNode {
                            tag: ElementTag::Strong,
                            id: vec![],
                            class: vec![],
                            children: vec![Node::Text(TextNode {
                                text: Cow::Borrowed("Hello World2")
                            })],
                        }),]
                    }),
                ]
            }
        );
    }

    #[test]
    fn test_transform2() {
        // - Hello
        // - World
        //   1. Change the **world**
        //   1. OK
        //     Good
        // - Hello2
        let tree = MarkdownTree {
            root: BlockTree {
                root: vec![BlockItem::BulletList(ListTree {
                    root: vec![
                        ListItem {
                            name: InlineTree {
                                root: vec![InlineItem::Text(Cow::Borrowed("Hello"))],
                            },
                            children: vec![],
                        },
                        ListItem {
                            name: InlineTree {
                                root: vec![InlineItem::Text(Cow::Borrowed("World"))],
                            },
                            children: vec![
                                BlockItem::OrderedList(ListTree {
                                    root: vec![
                                        ListItem {
                                            name: InlineTree {
                                                root: vec![InlineItem::Text(Cow::Borrowed(
                                                    "Change the ",
                                                ))],
                                            },
                                            children: vec![],
                                        },
                                        ListItem {
                                            name: InlineTree {
                                                root: vec![InlineItem::Strong(InlineTree {
                                                    root: vec![InlineItem::Text(Cow::Borrowed(
                                                        "world",
                                                    ))],
                                                })],
                                            },
                                            children: vec![],
                                        },
                                        ListItem {
                                            name: InlineTree {
                                                root: vec![
                                                    InlineItem::Text(Cow::Borrowed("OK")),
                                                    InlineItem::Break,
                                                    InlineItem::Text(Cow::Borrowed("Good")),
                                                ],
                                            },
                                            children: vec![],
                                        },
                                    ],
                                }),
                                BlockItem::Paragraph(InlineTree {
                                    root: vec![InlineItem::Text(Cow::Borrowed("OK"))],
                                }),
                            ],
                        },
                        ListItem {
                            name: InlineTree {
                                root: vec![InlineItem::Text(Cow::Borrowed("Hello2"))],
                            },
                            children: vec![],
                        },
                    ],
                })],
            },
        };

        let transformer = Transformer::new();
        let document = transformer.transform(tree);

        assert_eq!(
            document,
            DocumentNode {
                root: vec![Node::Element(ElementNode {
                    tag: ElementTag::Ul,
                    id: vec![],
                    class: vec![],
                    children: vec![
                        Node::Element(ElementNode {
                            tag: ElementTag::Li,
                            id: vec![],
                            class: vec![],
                            children: vec![Node::Text(TextNode {
                                text: Cow::Borrowed("Hello")
                            }),]
                        }),
                        Node::Element(ElementNode {
                            tag: ElementTag::Li,
                            id: vec![],
                            class: vec![],
                            children: vec![
                                Node::Text(TextNode {
                                    text: Cow::Borrowed("World")
                                }),
                                Node::Element(ElementNode {
                                    tag: ElementTag::Ol,
                                    id: vec![],
                                    class: vec![],
                                    children: vec![
                                        Node::Element(ElementNode {
                                            tag: ElementTag::Li,
                                            id: vec![],
                                            class: vec![],
                                            children: vec![Node::Text(TextNode {
                                                text: Cow::Borrowed("Change the ")
                                            }),]
                                        }),
                                        Node::Element(ElementNode {
                                            tag: ElementTag::Li,
                                            id: vec![],
                                            class: vec![],
                                            children: vec![Node::Element(ElementNode {
                                                tag: ElementTag::Strong,
                                                id: vec![],
                                                class: vec![],
                                                children: vec![Node::Text(TextNode {
                                                    text: Cow::Borrowed("world")
                                                }),]
                                            }),]
                                        }),
                                        Node::Element(ElementNode {
                                            tag: ElementTag::Li,
                                            id: vec![],
                                            class: vec![],
                                            children: vec![
                                                Node::Text(TextNode {
                                                    text: Cow::Borrowed("OK")
                                                }),
                                                Node::Element(ElementNode {
                                                    tag: ElementTag::Br,
                                                    id: vec![],
                                                    class: vec![],
                                                    children: vec![]
                                                }),
                                                Node::Text(TextNode {
                                                    text: Cow::Borrowed("Good")
                                                }),
                                            ]
                                        }),
                                    ]
                                }),
                                Node::Element(ElementNode {
                                    tag: ElementTag::P,
                                    id: vec![],
                                    class: vec![],
                                    children: vec![Node::Text(TextNode {
                                        text: Cow::Borrowed("OK")
                                    }),]
                                }),
                            ]
                        }),
                        Node::Element(ElementNode {
                            tag: ElementTag::Li,
                            id: vec![],
                            class: vec![],
                            children: vec![Node::Text(TextNode {
                                text: Cow::Borrowed("Hello2")
                            }),]
                        }),
                    ]
                }),]
            }
        )
    }
}
