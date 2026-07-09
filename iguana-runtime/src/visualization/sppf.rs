use std::io::{self, Write};

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    parser::Parser,
    sppf::{SPPFNode, SPPFNodeId},
    visualization::dot::{ToDot, escape_label},
};

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct SPPF {
    pub nodes: Vec<SPPFDotNode>,
    pub edges: Vec<SPPFDotEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum NodeKind {
    Nonterminal { ambiguous: bool },
    Intermediate { ambiguous: bool },
    Terminal,
    Packed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SPPFDotNode {
    pub id: SPPFNodeId,
    pub kind: NodeKind,
    pub label: String,
    pub left_extent: u32,
    pub right_extent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SPPFDotEdge {
    pub src: SPPFNodeId,
    pub dest: SPPFNodeId,
}

impl ToDot for SPPF {
    fn write_dot<W: Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(w, "digraph sppf {{")?;
        writeln!(w, "    rankdir=TB;")?;
        for n in &self.nodes {
            let label = escape_label(&n.label);
            match n.kind {
                NodeKind::Nonterminal { .. } => {
                    writeln!(
                        w,
                        "    N{}[label=\"{}\", shape=box, style=rounded];",
                        n.id, label
                    )?;
                }
                NodeKind::Intermediate { .. } | NodeKind::Terminal => {
                    writeln!(w, "    N{}[label=\"{}\", shape=box];", n.id, label)?;
                }
                NodeKind::Packed => {
                    writeln!(w, "    N{}[label=\"{}\", shape=\"\"];", n.id, label)?;
                }
            }
        }
        for e in &self.edges {
            writeln!(w, "    N{} -> N{};", e.src, e.dest)?;
        }
        writeln!(w, "}}")
    }
}

pub fn build_sppf_graph<'i, 'arena>(
    parser: &impl Parser<'i, 'arena>,
    start_node: SPPFNodeId,
) -> SPPF {
    SPPFGraphBuilder::new(parser).build(start_node)
}

struct SPPFGraphBuilder<'p, P> {
    parser: &'p P,
    nodes: Vec<SPPFDotNode>,
    edges: Vec<SPPFDotEdge>,
    visited_nodes: FxHashSet<SPPFNodeId>,
    current_packed_node_id: usize,
}

impl<'p, 'i, 'arena, P: Parser<'i, 'arena>> SPPFGraphBuilder<'p, P> {
    fn new(parser: &'p P) -> Self {
        Self {
            parser,
            nodes: vec![],
            edges: vec![],
            visited_nodes: FxHashSet::default(),
            current_packed_node_id: parser.sppf_nodes().len(),
        }
    }
    fn build(mut self, start_id: SPPFNodeId) -> SPPF {
        self.visit_node(start_id);
        SPPF {
            nodes: self.nodes,
            edges: self.edges,
        }
    }
    fn visit_node(&mut self, id: SPPFNodeId) {
        if self.visited_nodes.contains(&id) {
            return;
        }
        self.visited_nodes.insert(id);
        let node = self.parser.sppf_node(id);
        let label = self.parser.sppf_node_to_string(self.parser.sppf_node(id));
        match node {
            SPPFNode::Terminal(_) => {
                let dot_node = SPPFDotNode {
                    id,
                    label,
                    kind: NodeKind::Terminal,
                    left_extent: node.left_extent(),
                    right_extent: node.right_extent(),
                };
                self.nodes.push(dot_node);
            }
            SPPFNode::Nonterminal(n) => {
                let dot_node = SPPFDotNode {
                    id,
                    kind: NodeKind::Nonterminal {
                        ambiguous: n.ambiguous,
                    },
                    label,
                    left_extent: node.left_extent(),
                    right_extent: node.right_extent(),
                };
                self.nodes.push(dot_node);
                self.add_edge(id, n.child);
                self.visit_node(n.child);
                if n.ambiguous {
                    let children_map = self.parser.nonterminal_nodes_children_map();
                    if let Some(children) = children_map.get(&id) {
                        for (child, _) in children {
                            self.add_edge(id, *child);
                            self.visit_node(*child);
                        }
                    }
                }
            }
            SPPFNode::Intermediate(i) => {
                let dot_node = SPPFDotNode {
                    id,
                    kind: NodeKind::Intermediate {
                        ambiguous: i.ambiguous,
                    },
                    label,
                    left_extent: node.left_extent(),
                    right_extent: node.right_extent(),
                };
                self.nodes.push(dot_node);
                if i.ambiguous {
                    self.add_packed_node_to_intermediate_node(id, i.child.0, i.child.1);
                    let children_map = self.parser.intermediate_nodes_children_map();
                    if let Some(children) = children_map.get(&id) {
                        for (left_child, right_child) in children {
                            self.add_packed_node_to_intermediate_node(
                                id,
                                *left_child,
                                *right_child,
                            );
                        }
                    }
                } else {
                    let (left_child, right_child) = i.child;
                    self.add_edge(id, left_child);
                    self.add_edge(id, right_child);
                    self.visit_node(left_child);
                    self.visit_node(right_child);
                }
            }
        }
    }

    // Add a packed node to group the children of each intermediate node
    // when there is ambiguity.
    fn add_packed_node_to_intermediate_node(
        &mut self,
        id: SPPFNodeId,
        left_child: SPPFNodeId,
        right_child: SPPFNodeId,
    ) {
        let packed_node_id = SPPFNodeId(self.current_packed_node_id as u32);
        self.current_packed_node_id += 1;
        // Packed nodes are virtual nodes for visualization only, they don't have spans
        let packed_node = SPPFDotNode {
            id: packed_node_id,
            kind: NodeKind::Packed,
            label: "".to_owned(),
            left_extent: 0,
            right_extent: 0,
        };
        self.nodes.push(packed_node);
        self.add_edge(id, packed_node_id);
        self.add_edge(packed_node_id, left_child);
        self.add_edge(packed_node_id, right_child);
        self.visit_node(left_child);
        self.visit_node(right_child);
    }

    fn add_edge(&mut self, parent: SPPFNodeId, child: SPPFNodeId) {
        self.edges.push(SPPFDotEdge {
            src: parent,
            dest: child,
        });
    }
}
