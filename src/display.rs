use std::fmt::Write;

use crate::{node::Node, noderef::NodeRef};

pub struct TreeDisplay;

impl TreeDisplay {
    pub fn format<R, F>(
        node: &R,
        f: &mut std::fmt::Formatter<'_>,
        data_format: F,
    ) -> std::fmt::Result
    where
        R: NodeRef,
        F: Fn(
            <<R as NodeRef>::Inner as Node>::DataRef<'_>,
            &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result,
    {
        f.write_str("\n")?;

        let mut iter = node.clone().into_iter().peekable();

        let mut root_children = false;

        let column_width = 2;

        loop {
            if let Some(node) = iter.next() {
                // Peek at the next node to see if there are siblings
                let has_siblings = if let Some(next_node) = iter.peek() {
                    node.depth() == next_node.depth()
                } else {
                    false
                };

                let has_children = node.node().children().is_some();

                if node.depth() == 0 {
                    root_children = has_children
                }

                // The position of the first character of the payload from the previous row
                let pos = node.depth() * column_width;

                if node.depth() == 0 {
                    if has_children || has_siblings {
                        f.write_char('┏')?;
                    } else {
                        f.write_char('━')?;
                    }
                } else {
                    for i in 0..pos {
                        if i % column_width == 0 {
                            f.write_char('┃')?;
                        } else {
                            f.write_char(' ')?;
                        }
                    }

                    if has_children || has_siblings {
                        f.write_char('┣')?;
                    } else {
                        f.write_char('┗')?;
                    }
                }

                data_format(node.node().data(), f)?;

                f.write_char('\n')?;

                //f.write_fmt(format_args!(" {}\n", node.node().data()))?;
            } else {
                // Finished node iteration
                if root_children {
                    f.write_str("┗")?;
                }
                return Ok(());
            }
        }
    }
}
