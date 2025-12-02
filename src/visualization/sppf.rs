use std::{
    borrow::Cow,
    fs::File,
    io::{self, BufWriter},
    path::Path,
};

use dot::Labeller;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    parser::Parser,
    sppf::{SPPFNode, SPPFNodeId},
};

#[derive(Debug)]
pub struct SPPF {
    // A map from node_id to SPPFDotNode
    // Because not all created SPPF nodes are reachable from the root,
    // and for visualization, we only care about the reachable nodes from the root,
    // we need a hashmap here to only keep track of the reachable nodes.
    pub nodes_map: FxHashMap<SPPFNodeId, SPPFDotNode>,
    pub nodes: Vec<SPPFDotNode>,
    pub edges: Vec<SPPFDotEdge>,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Nonterminal,
    Intermediate,
    Terminal,
    Packed,
}

#[derive(Debug, Clone)]
pub struct SPPFDotNode {
    pub id: SPPFNodeId,
    pub kind: NodeKind,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct SPPFDotEdge {
    pub src: SPPFNodeId,
    pub dest: SPPFNodeId,
}

impl<'a> Labeller<'a, SPPFDotNode, SPPFDotEdge> for SPPF {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("GSS").unwrap()
    }

    fn node_id(&'a self, n: &SPPFDotNode) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n.id)).unwrap()
    }

    fn node_label(&'a self, n: &SPPFDotNode) -> dot::LabelText<'a> {
        dot::LabelText::LabelStr(Cow::Owned(n.label.clone()))
    }

    fn rank_dir(&'a self) -> Option<dot::RankDir> {
        Some(dot::RankDir::TopBottom)
    }

    fn node_shape(&'a self, n: &SPPFDotNode) -> Option<dot::LabelText<'a>> {
        let shape = match n.kind {
            NodeKind::Nonterminal => dot::LabelText::LabelStr(Cow::Borrowed("box")),
            NodeKind::Intermediate => dot::LabelText::LabelStr(Cow::Borrowed("box")),
            NodeKind::Terminal => dot::LabelText::LabelStr(Cow::Borrowed("box")),
            NodeKind::Packed => dot::LabelText::LabelStr(Cow::Borrowed("")),
        };
        Some(shape)
    }

    fn node_style(&'a self, n: &SPPFDotNode) -> dot::Style {
        match n.kind {
            NodeKind::Nonterminal => dot::Style::Rounded,
            NodeKind::Intermediate => dot::Style::None,
            NodeKind::Terminal => dot::Style::None,
            NodeKind::Packed => dot::Style::None,
        }
    }
}

impl<'a> dot::GraphWalk<'a, SPPFDotNode, SPPFDotEdge> for SPPF {
    fn nodes(&'a self) -> dot::Nodes<'a, SPPFDotNode> {
        Cow::Borrowed(&self.nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, SPPFDotEdge> {
        Cow::Borrowed(&self.edges)
    }

    fn source(&'a self, edge: &SPPFDotEdge) -> SPPFDotNode {
        self.nodes_map.get(&edge.src).unwrap().clone()
    }

    fn target(&'a self, edge: &SPPFDotEdge) -> SPPFDotNode {
        self.nodes_map.get(&edge.dest).unwrap().clone()
    }
}

pub fn write_sppf_dot<'i>(
    parser: &impl Parser<'i>,
    start_node: SPPFNodeId,
    path: impl AsRef<Path>,
) -> io::Result<()> {
    let file = File::create(path)?;
    let mut sppf_dot_file = BufWriter::new(file);
    let builder = SPPFGraphBuilder::new(parser);
    let sppf = builder.build(start_node);
    dot::render(&sppf, &mut sppf_dot_file)
}

struct SPPFGraphBuilder<'a, P> {
    parser: &'a P,
    nodes_map: FxHashMap<SPPFNodeId, SPPFDotNode>,
    nodes: Vec<SPPFDotNode>,
    edges: Vec<SPPFDotEdge>,
    visited_nodes: FxHashSet<SPPFNodeId>,
    current_packed_node_id: usize,
}

impl<'a, 'i, P: Parser<'i>> SPPFGraphBuilder<'a, P> {
    fn new(parser: &'a P) -> Self {
        Self {
            parser,
            nodes_map: FxHashMap::default(),
            nodes: vec![],
            edges: vec![],
            visited_nodes: FxHashSet::default(),
            current_packed_node_id: parser.stats().count_all_sppf_nodes(),
        }
    }
    fn build(mut self, start_id: SPPFNodeId) -> SPPF {
        self.visit_node(start_id);
        SPPF {
            nodes_map: self.nodes_map,
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
                let node = SPPFDotNode {
                    id,
                    label,
                    kind: NodeKind::Terminal,
                };
                self.nodes.push(node.clone());
                self.nodes_map.insert(id, node);
            }
            SPPFNode::Nonterminal(n) => {
                let node = SPPFDotNode {
                    id,
                    kind: NodeKind::Nonterminal,
                    label,
                };
                self.nodes.push(node.clone());
                self.nodes_map.insert(id, node);
                self.add_edge(id, n.child);
                self.visit_node(n.child);
                if n.ambiguous {
                    let children_map = self.parser.nonterminal_nodes_children_map();
                    if let Some(children) = children_map.get(&id) {
                        for child in children {
                            self.add_edge(id, *child);
                            self.visit_node(*child);
                        }
                    }
                }
            }
            SPPFNode::Intermediate(i) => {
                let node = SPPFDotNode {
                    id,
                    kind: NodeKind::Intermediate,
                    label,
                };
                self.nodes.push(node.clone());
                self.nodes_map.insert(id, node);
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
        let packed_node = SPPFDotNode {
            id: packed_node_id,
            kind: NodeKind::Packed,
            label: "".to_owned(),
        };
        self.nodes.push(packed_node.clone());
        self.nodes_map.insert(packed_node_id, packed_node);
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
