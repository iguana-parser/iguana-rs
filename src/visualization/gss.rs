use std::{borrow::Cow, fs::File, io, io::BufWriter, path::Path};

use dot::Labeller;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{ids::GssNodeId, parser::Parser};

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct GSS {
    pub nodes: Vec<GSSDotNode>,
    pub edges: Vec<GSSDotEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GSSDotNode {
    pub id: GssNodeId,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GSSDotEdge {
    #[serde(skip)]
    #[specta(skip)]
    pub id: usize,
    pub src: GssNodeId,
    pub dest: GssNodeId,
    pub label: String,
}

impl<'a> Labeller<'a, GSSDotNode, GSSDotEdge> for GSS {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("GSS").unwrap()
    }

    fn node_id(&'a self, n: &GSSDotNode) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n.id)).unwrap()
    }

    fn node_label(&'a self, n: &GSSDotNode) -> dot::LabelText<'a> {
        dot::LabelText::LabelStr(Cow::Borrowed(&self.nodes[n.id.index()].label))
    }

    fn edge_label(&'a self, e: &GSSDotEdge) -> dot::LabelText<'a> {
        dot::LabelText::LabelStr(Cow::Borrowed(&self.edges[e.id].label))
    }

    fn rank_dir(&'a self) -> Option<dot::RankDir> {
        Some(dot::RankDir::BottomTop)
    }
}

impl<'a> dot::GraphWalk<'a, GSSDotNode, GSSDotEdge> for GSS {
    fn nodes(&'a self) -> dot::Nodes<'a, GSSDotNode> {
        Cow::Borrowed(&self.nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, GSSDotEdge> {
        Cow::Borrowed(&self.edges)
    }

    fn source(&'a self, edge: &GSSDotEdge) -> GSSDotNode {
        self.nodes[edge.src.index()].clone()
    }

    fn target(&'a self, edge: &GSSDotEdge) -> GSSDotNode {
        self.nodes[edge.dest.index()].clone()
    }
}

pub fn render_gss<'i>(parser: &impl Parser<'i>, path: impl AsRef<Path>) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    let gss: GSS = build_gss_dot_graph(parser);
    dot::render(&gss, &mut writer)
}

pub fn build_gss_dot_graph<'i, P: Parser<'i>>(parser: &P) -> GSS {
    let mut nodes = Vec::with_capacity(parser.stats().gss_nodes_count);
    let mut edges = Vec::with_capacity(parser.stats().gss_edges_count);
    for gss_node in parser.gss_nodes() {
        nodes.push(GSSDotNode {
            id: gss_node.id,
            label: parser.gss_to_string(gss_node.id),
        });
        for (id, gss_edge) in gss_node.edges().iter().enumerate() {
            edges.push(GSSDotEdge {
                id,
                src: gss_node.id,
                dest: gss_edge.dest_id,
                label: P::slot_name(gss_edge.return_slot).into(),
            });
        }
    }
    GSS { nodes, edges }
}
