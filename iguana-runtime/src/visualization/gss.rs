use std::io::{self, Write};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    ids::GssNodeId,
    parser::Parser,
    visualization::dot::{ToDot, escape_label},
};

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
    pub src: GssNodeId,
    pub dest: GssNodeId,
    pub label: String,
}

impl ToDot for GSS {
    fn write_dot<W: Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(w, "digraph gss {{")?;
        writeln!(w, "    rankdir=BT;")?;
        for n in &self.nodes {
            writeln!(w, "    N{}[label=\"{}\"];", n.id, escape_label(&n.label))?;
        }
        for e in &self.edges {
            writeln!(
                w,
                "    N{} -> N{}[label=\"{}\"];",
                e.src,
                e.dest,
                escape_label(&e.label)
            )?;
        }
        writeln!(w, "}}")
    }
}

pub fn build_gss_dot_graph<'i, P: Parser<'i>>(parser: &P) -> GSS {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for gss_node in parser.gss_nodes() {
        nodes.push(GSSDotNode {
            id: gss_node.id,
            label: parser.gss_to_string(gss_node.id),
        });
        for gss_edge in gss_node.edges() {
            edges.push(GSSDotEdge {
                src: gss_node.id,
                dest: gss_edge.dest_id,
                label: P::slot_name(gss_edge.return_slot).into(),
            });
        }
    }
    GSS { nodes, edges }
}
