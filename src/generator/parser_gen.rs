use proc_macro2::Literal;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::generator::id::EndSlot;
use crate::generator::id::NonterminalIds;
use crate::generator::id::SlotIds;
use crate::generator::id::TerminalIds;
use crate::generator::utils::to_first_uppercase;
use crate::grammar::grammar::Alternative;
use crate::grammar::grammar::Grammar;
use crate::grammar::symbols::Nonterminal;
use crate::grammar::symbols::Symbol;
use crate::grammar::symbols::Terminal;

pub fn generate(
    grammar: &Grammar,
    nonterminal_ids: &mut NonterminalIds,
    slot_ids: &mut SlotIds,
    terminal_ids: &mut TerminalIds,
) -> TokenStream {
    let grammar_name = &grammar.name;
    let imports = gen_imports(grammar);
    let nonterminals_const = gen_nonterminals_const(nonterminal_ids);
    let execute_method = gen_execute_method(grammar, nonterminal_ids, slot_ids, terminal_ids);
    let first_descriptors = gen_add_first_descriptors_method(grammar, nonterminal_ids, slot_ids);
    let terminals_const = gen_terminals_const(terminal_ids);
    let slots_const = gen_slots_const(slot_ids);
    let nonterminal_name_method = gen_nonterminal_name_method();
    let nonterminals_method = gen_nonterminals_method();
    let terminal_name_method = gen_terminal_name_method();
    let slot_name_method = gen_slot_name_method();
    let get_gss_node_method = gen_get_gss_node_method();
    let gen_add_gss_node_method = gen_add_gss_node_method();
    let gen_new_gss_node_method = gen_new_gss_node_method();
    let gss_node_method = gen_gss_node_method();
    let gss_node_mut_method = gen_gss_node_mut_method();
    let sppf_node_method = gen_sppf_node_method();
    let sppf_node_mut_method = gen_sppf_node_mut_method();
    let add_descriptor_method = gen_add_descriptor_method();
    let next_descriptor_method = gen_next_descriptor_method();
    let new_terminal_node_method = gen_add_terminal_node_method();
    let new_nonterminal_node_method = gen_add_nonterminal_node_method();
    let new_intermediate_node_method = gen_add_intermediate_node_method();
    let input_len_method = gen_input_method();
    let stats_method = gen_stats_method();
    let stats_mut_method = gen_stats_mut_method();
    let lookup_nonterminal_node_method = gen_lookup_nonterminal_node_method();
    let lookup_intermediate_node_method = gen_lookup_intermediate_node_method();
    let lookup_terminal_node_method = gen_lookup_terminal_node_method();
    let gss_nodes_method = gen_gss_nodes_method();
    let add_nonterminal_node_child_method = gen_add_nonterminal_node_child_method();
    let add_intermediate_node_child_method = gen_add_intermediate_node_child_method();
    let intermediate_nodes_children_method = gen_intermediate_nodes_children_map_method();
    let nonterminal_nodes_children_method = gen_nonterminal_nodes_children_map_method();
    let add_trace_event_method = gen_add_trace_event_method();
    let start_nonterminal_method = gen_start_nonterminal_method();
    let parser_struct = gen_parser_struct(grammar_name, nonterminal_ids, terminal_ids, slot_ids);
    let parser_impl = gen_parser_impl(grammar_name, nonterminal_ids, terminal_ids, slot_ids);
    let grammar_name_ident = format_ident!("{}Parser", to_first_uppercase(grammar_name));
    quote! {
        #imports
        #nonterminals_const
        #terminals_const
        #slots_const
        impl<'i> Parser<'i> for #grammar_name_ident<'i> {
            #execute_method
            #first_descriptors
            #nonterminal_name_method
            #nonterminals_method
            #terminal_name_method
            #slot_name_method
            #get_gss_node_method
            #gen_add_gss_node_method
            #gen_new_gss_node_method
            #gss_node_method
            #gss_node_mut_method
            #sppf_node_method
            #sppf_node_mut_method
            #add_descriptor_method
            #next_descriptor_method
            #new_terminal_node_method
            #new_nonterminal_node_method
            #new_intermediate_node_method
            #input_len_method
            #stats_method
            #stats_mut_method
            #lookup_nonterminal_node_method
            #lookup_intermediate_node_method
            #lookup_terminal_node_method
            #gss_nodes_method
            #add_intermediate_node_child_method
            #add_nonterminal_node_child_method
            #intermediate_nodes_children_method
            #nonterminal_nodes_children_method
            #add_trace_event_method
            #start_nonterminal_method
        }
        #parser_struct
        #parser_impl
    }
}

fn gen_imports(grammar: &Grammar) -> TokenStream {
    let scanner_name = format_ident!("{}Scanner", to_first_uppercase(&grammar.name));
    quote! {
        use std::{cell::OnceCell, sync::LazyLock};
        use iguana::{
            descriptor::Descriptor,
            grammar::symbols::{Nonterminal, NonterminalNodeKind},
            gss::GSSNode,
            ids::{GssNodeId, NonterminalId, SlotId, TerminalId},
            input::Input,
            parser::{Parser, Stats, init_logger},
            record,
            scanner::Scanner,
            sppf::{IntermediateNode, NonterminalNode, SPPFNode, SPPFNodeId, Span, TerminalNode},
            utils::inline_map::InlineMap,
        };
        #[cfg(feature = "debug-trace")]
        use iguana::trace::TraceEvent;
        use crate::scanner::#scanner_name;
        use rustc_hash::FxHashMap;
    }
}

fn gen_add_first_descriptors_method(
    grammar: &Grammar,
    nonterminal_ids: &NonterminalIds,
    slot_ids: &mut SlotIds,
) -> TokenStream {
    let mut nonterminal_quotes = vec![];
    for nonterminal in grammar.nonterminals() {
        let nonterminal_id = nonterminal_ids.get_id(nonterminal);
        let nt_name = &nonterminal.name;
        let mut alternative_quotes = vec![];
        let alternatives = grammar.alternatives(nonterminal);
        if alternatives.is_empty() {
            // todo: handle the empty alternative
            todo!()
        }
        for alternative in alternatives {
            let slot_name = slot_to_string(nt_name, alternative, 0);
            let first_slot = slot_ids.id(&slot_name);
            alternative_quotes.push(quote! {
                #[comment = #slot_name]
                self.add_descriptor(Descriptor {
                    input_index,
                    slot_id: #first_slot,
                    sppf_node_id: None,
                    gss_node_id,
                });
            });
        }
        nonterminal_quotes.push(quote! {
            #[comment = #nt_name]
            #nonterminal_id => {
                #( #alternative_quotes)*
            }
        });
    }
    quote! {
        fn add_first_descriptors(
            &mut self,
            nonterminal_id: NonterminalId,
            input_index: u32,
            gss_node_id: GssNodeId
        ) {
            match nonterminal_id {
                #( #nonterminal_quotes)*
                _ => {
                    panic!("Unknown nonterminal id: {nonterminal_id}");
                }
            }
        }
    }
}

fn gen_nonterminals_const(nonterminal_ids: &NonterminalIds) -> TokenStream {
    let nonterminals_len = Literal::usize_unsuffixed(nonterminal_ids.len());
    let nonterminal_names = nonterminal_ids.nonterminals().map(|n| {
        let nonterminal_name = &n.name;
        let kind = &n.kind;
        quote! { Nonterminal::with_kind(#nonterminal_name, #kind) }
    });
    quote! {
        static NONTERMINALS: LazyLock<[Nonterminal; #nonterminals_len]> = LazyLock::new(|| [#(#nonterminal_names),*]);
    }
}

fn gen_terminals_const(terminal_ids: &TerminalIds) -> TokenStream {
    let terminals_len = Literal::usize_unsuffixed(terminal_ids.len());
    let terminal_names = terminal_ids.terminals().map(|t| {
        let terminal_name = &t.name;
        quote! { #terminal_name }
    });
    quote! {
        const TERMINALS: [&str; #terminals_len] = [#(#terminal_names),*];
    }
}

fn gen_slots_const(slot_ids: &SlotIds) -> TokenStream {
    let slots_len = Literal::usize_unsuffixed(slot_ids.len());
    let slot_names = slot_ids.slots().map(|s| quote! { #s });
    quote! {
        const SLOTS: [&str; #slots_len] = [#(#slot_names),*];
    }
}

fn gen_execute_method(
    grammar: &Grammar,
    nonterminal_ids: &mut NonterminalIds,
    slot_ids: &mut SlotIds,
    terminal_ids: &mut TerminalIds,
) -> TokenStream {
    let mut slot_quotes = vec![];
    for nonterminal in grammar.nonterminals() {
        let nt_name = &nonterminal.name;
        let alternatives = grammar.alternatives(nonterminal);
        for (index, alternative) in alternatives.iter().enumerate() {
            for (position, symbol) in alternative.symbols.iter().enumerate() {
                slot_quotes.push(gen_slot_code(
                    position,
                    symbol,
                    nt_name,
                    alternative,
                    nonterminal_ids,
                    terminal_ids,
                    slot_ids,
                ));
            }
            // Handle the last grammar slot
            let last_symbol_index = alternative.symbols.len();
            let end_slot_name = slot_to_string(nt_name, alternative, last_symbol_index);
            let end_slot_id = slot_ids.id(&end_slot_name);
            let nonterminal_id = nonterminal_ids
                .get_id(nonterminal)
                .expect("nonterminal not found");
            let alternative = EndSlot {
                index,
                slot_id: end_slot_id,
            };
            nonterminal_ids.add_end_slot(nonterminal_id, alternative);
            let last_slot_quote = quote! {
                #[comment = #end_slot_name]
                #end_slot_id => {
                    let Some(result) = result else {
                        unreachable!("result cannot be None here.")
                    };
                    let node = self.sppf_node(result);
                    let left_extent = node.left_extent();
                    let right_extent = node.right_extent();
                    let nonterminal_id = #nonterminal_id;
                    let return_slot = #end_slot_id;
                    if let Some(nonterminal_node_id) = self.create_nonterminal_node_or_attach_children(
                        nonterminal_id,
                        return_slot,
                        left_extent,
                        right_extent,
                        result,
                    ) {
                        self.pop(gss_node_id, nonterminal_node_id);
                    }
                }
            };
            slot_quotes.push(last_slot_quote);
        }
    }

    quote! {
        fn execute(
            &mut self,
            input_index: u32,
            slot_id: SlotId,
            result: Option<SPPFNodeId>,
            gss_node_id: GssNodeId
        ) {
            record!(self, ProcessingDescriptor, input_index, slot_id, result, gss_node_id);
            match slot_id {
                #(#slot_quotes)*
                _ => {
                    panic!("Unknown grammar slot id: {slot_id}");
                }
            }
        }
    }
}

fn gen_slot_code(
    position: usize,
    symbol: &Symbol,
    nt_name: &str,
    alternative: &Alternative,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &mut TerminalIds,
    slot_ids: &mut SlotIds,
) -> TokenStream {
    let slot_name = slot_to_string(nt_name, alternative, position);
    match symbol {
        Symbol::Terminal(terminal) => {
            let next_slot_name = slot_to_string(nt_name, alternative, position + 1);
            gen_terminal_slot(
                terminal,
                position,
                &slot_name,
                &next_slot_name,
                terminal_ids,
                slot_ids,
            )
        }
        Symbol::Nonterminal(nonterminal) => {
            let next_slot_name = slot_to_string(nt_name, alternative, position + 1);
            gen_nonterminal_slot(
                nonterminal,
                &slot_name,
                &next_slot_name,
                nonterminal_ids,
                slot_ids,
            )
        }
        _ => panic!("At runtime only terminal and nonterminals are supported."),
    }
}

/// Generates code for the grammar slots before a terminal.
fn gen_terminal_slot(
    terminal: &Terminal,
    position: usize,
    slot_name: &str,
    next_slot_name: &str,
    terminal_ids: &mut TerminalIds,
    slot_ids: &mut SlotIds,
) -> TokenStream {
    let terminal_id = terminal_ids
        .get_id(terminal)
        .unwrap_or_else(|| panic!("cannot not find the lexical definition {}", terminal.name));
    let slot_id = slot_ids.id(slot_name);
    let next_slot_id = slot_ids.id(next_slot_name);
    // At grammar position 0, we do not need to create an intermediate node.
    let new_node = if position == 0 {
        quote! {
            let new_node = right_child_id;
            self.execute(j, next_slot_id, Some(new_node), gss_node_id);
        }
    } else {
        quote! {
            let left_child_id = result.expect("Result should not be None.");
            let left_child = self.sppf_node(left_child_id);
            let left_extent = left_child.left_extent();
            if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                next_slot_id,
                left_extent,
                j,
                left_child_id,
                right_child_id,
            ) {
                self.execute(j, next_slot_id, Some(new_node), gss_node_id);
            }
        }
    };
    let terminal_name = &terminal.name;
    quote! {
        #[comment = #slot_name]
        #slot_id => {
            record!(self, MatchingLeadingLayout, input_index);
            let (i, leading_layout) = self.scanner.match_leading_layout(input_index);
            record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
            record!(self, MatchingTerminal, #terminal_name, i);
            match self.scanner.match_token(#terminal_id, i) {
                Some(j) => {
                    record!(self, MatchSuccess, #terminal_name, i, j);
                    record!(self, MatchingTrailingLayout, i);
                    let (i, trailing_layout) = self.scanner.match_trailing_layout(i);
                    record!(self, MatchedLayout, leading_layout.is_empty().then_some(i));
                    let right_child_id = self.get_or_create_terminal_node(
                        #terminal_id,
                        i,
                        j,
                        leading_layout,
                        trailing_layout
                    );
                    #[comment = #next_slot_name]
                    let next_slot_id = #next_slot_id;
                    #new_node
                }
                None => {
                    record!(self, MatchFailed, #terminal_name, i);
                }
            }
        }
    }
}

fn gen_nonterminal_slot(
    nonterminal: &Nonterminal,
    slot_name: &str,
    next_slot_name: &str,
    nonterminal_ids: &NonterminalIds,
    slot_ids: &mut SlotIds,
) -> TokenStream {
    let nonterminal_id = nonterminal_ids
        .get_id(nonterminal)
        .unwrap_or_else(|| panic!("nonterminal {} is not defined", nonterminal.name));
    let slot_id = slot_ids.id(slot_name);
    let return_slot_id = slot_ids.id(next_slot_name);
    quote! {
        #[comment = #slot_name]
        #slot_id => {
            self.create(#nonterminal_id, result, gss_node_id, #return_slot_id);
        }
    }
}

fn gen_nonterminal_name_method() -> TokenStream {
    quote! {
        fn nonterminal(&self, nonterminal_id: NonterminalId) -> &Nonterminal {
            &NONTERMINALS[nonterminal_id.index()]
        }
    }
}

fn gen_nonterminals_method() -> TokenStream {
    quote! {
        fn nonterminals() -> impl Iterator<Item = &'static Nonterminal> {
            NONTERMINALS.iter()
        }
    }
}

fn gen_terminal_name_method() -> TokenStream {
    quote! {
        fn terminal_name(&self, terminal_id: TerminalId) -> &str {
            TERMINALS[terminal_id.index()]
        }
    }
}

fn gen_slot_name_method() -> TokenStream {
    quote! {
        fn slot_name(&self, slot_id: SlotId) -> &str {
            SLOTS[slot_id.index()]
        }
    }
}

fn gen_get_gss_node_method() -> TokenStream {
    quote! {
        fn get_gss_node(&self, nonterminal_id: NonterminalId, input_index: u32) -> Option<GssNodeId> {
            let gss_nodes = &self.gss_nodes_index[nonterminal_id.index()];
            gss_nodes.iter().find(|(k, _)| *k == input_index).map(|x| x.1)
        }
    }
}

fn gen_add_gss_node_method() -> TokenStream {
    quote! {
        fn add_gss_node(&mut self, nonterminal_id: NonterminalId, input_index: u32, gss_node_id: GssNodeId) {
            let gss_nodes = &mut self.gss_nodes_index[nonterminal_id.index()];
            gss_nodes.push((input_index, gss_node_id));
        }
    }
}

fn gen_new_gss_node_method() -> TokenStream {
    quote! {
        fn new_gss_node(&mut self, nonterminal_id: NonterminalId, input_index: u32) -> GssNodeId {
            let gss_node_id = GssNodeId(self.gss_nodes.len() as u32);
            let gss_node = GSSNode::new(gss_node_id, nonterminal_id, input_index);
            record!(self, GSSNodeCreated, nonterminal_id, input_index);
            self.gss_nodes.push(gss_node);
            self.stats.gss_nodes_count += 1;
            gss_node_id
        }
    }
}

fn gen_gss_node_method() -> TokenStream {
    quote! {
        fn gss_node(&self, id: GssNodeId) -> &GSSNode {
            &self.gss_nodes[id.index()]
        }
    }
}

fn gen_gss_node_mut_method() -> TokenStream {
    quote! {
        fn gss_node_mut(&mut self, id: GssNodeId) -> &mut GSSNode {
            self.gss_nodes.get_mut(id.index()).expect("GSS node id should be valid")
        }
    }
}

fn gen_sppf_node_method() -> TokenStream {
    quote! {
        fn sppf_node(&self, id: SPPFNodeId) -> &SPPFNode {
            &self.sppf_nodes[id.index()]
        }
    }
}

fn gen_sppf_node_mut_method() -> TokenStream {
    quote! {
        fn sppf_node_mut(&mut self, id: SPPFNodeId) -> &mut SPPFNode {
            &mut self.sppf_nodes[id.index()]
        }
    }
}

fn gen_add_descriptor_method() -> TokenStream {
    quote! {
        fn add_descriptor(&mut self, descriptor: Descriptor) {
            record!(
                self,
                DescriptorAdded,
                descriptor.input_index,
                descriptor.slot_id,
                descriptor.sppf_node_id,
                descriptor.gss_node_id
            );
            self.stats_mut().descriptors_count += 1;
            self.descriptors.push(descriptor);
        }
    }
}

fn gen_next_descriptor_method() -> TokenStream {
    quote! {
        fn next_descriptor(&mut self) -> Option<Descriptor> {
            self.descriptors.pop()
        }
    }
}

fn gen_add_terminal_node_method() -> TokenStream {
    quote! {
        fn add_terminal_node(&mut self, terminal_node: TerminalNode) -> SPPFNodeId {
            let terminal_node_id = SPPFNodeId(self.sppf_nodes.len() as u32);
            self.stats.terminal_nodes_count += 1;
            self.terminal_nodes_index[terminal_node.terminal_id.index()]
                .insert(terminal_node.span, terminal_node_id);
            record!(self, TerminalNodeCreated, terminal_node.terminal_id, terminal_node.span);
            self.sppf_nodes.push(SPPFNode::Terminal(terminal_node));
            terminal_node_id
        }
    }
}

fn gen_add_nonterminal_node_method() -> TokenStream {
    quote! {
        fn add_nonterminal_node(&mut self, nonterminal_node: NonterminalNode) -> SPPFNodeId {
            let nonterminal_node_id = SPPFNodeId(self.sppf_nodes.len() as u32);
            self.stats.nonterminal_nodes_count += 1;
            self.nonterminal_nodes_index[nonterminal_node.nonterminal_id.index()]
                .insert(nonterminal_node.span, nonterminal_node_id);
            record!(self, NonterminalNodeCreated, nonterminal_node.nonterminal_id, nonterminal_node.span);
            self.sppf_nodes.push(SPPFNode::Nonterminal(nonterminal_node));
            nonterminal_node_id
        }
    }
}

fn gen_add_intermediate_node_method() -> TokenStream {
    quote! {
        fn add_intermediate_node(&mut self, intermediate_node: IntermediateNode) -> SPPFNodeId {
            let intermediate_node_id = SPPFNodeId(self.sppf_nodes.len() as u32);
            self.stats.intermediate_nodes_count += 1;
            self.intermediate_nodes_index[intermediate_node.slot_id.index()]
                .insert(intermediate_node.span, intermediate_node_id);
            record!(self, IntermediateNodeCreated, intermediate_node.slot_id, intermediate_node.span);
            self.sppf_nodes.push(SPPFNode::Intermediate(intermediate_node));
            intermediate_node_id
        }
    }
}

fn gen_input_method() -> TokenStream {
    quote! {
        fn input(&self) -> &'i Input {
            self.scanner.input
        }
    }
}

fn gen_stats_method() -> TokenStream {
    quote! {
        fn stats(&self) -> &Stats {
            &self.stats
        }
    }
}

fn gen_stats_mut_method() -> TokenStream {
    quote! {
        fn stats_mut(&mut self) -> &mut Stats {
            &mut self.stats
        }
    }
}

fn gen_lookup_nonterminal_node_method() -> TokenStream {
    quote! {
        fn lookup_nonterminal_node(
            &self,
            nonterminal_id: NonterminalId,
            left_extent: u32,
            right_extent: u32,
        ) -> Option<SPPFNodeId> {
            let map = &self.nonterminal_nodes_index[nonterminal_id.index()];
            map.get(&Span::new(left_extent, right_extent)).copied()
        }
    }
}

fn gen_lookup_intermediate_node_method() -> TokenStream {
    quote! {
        fn lookup_intermediate_node(
            &self,
            slot_id: SlotId,
            left_extent: u32,
            right_extent: u32,
        ) -> Option<SPPFNodeId> {
            let map = &self.intermediate_nodes_index[slot_id.index()];
            map.get(&Span::new(left_extent, right_extent)).copied()
        }
    }
}

fn gen_lookup_terminal_node_method() -> TokenStream {
    quote! {
        fn lookup_terminal_node(
            &self,
            terminal_id: TerminalId,
            left_extent: u32,
            right_extent: u32,
        ) -> Option<SPPFNodeId> {
            let map = &self.terminal_nodes_index[terminal_id.index()];
            map.get(&Span::new(left_extent, right_extent)).copied()
        }
    }
}

fn gen_gss_nodes_method() -> TokenStream {
    quote! {
        fn gss_nodes(&self) -> impl Iterator<Item = &GSSNode> {
            self.gss_nodes.iter()
        }
    }
}

fn gen_add_intermediate_node_child_method() -> TokenStream {
    quote! {
        fn add_intermediate_node_child(
            &mut self,
            node: SPPFNodeId,
            child1: SPPFNodeId,
            child2: SPPFNodeId,
        ) {
            self.intermediate_nodes_children
                .push((node, (child1, child2)));
        }
    }
}

fn gen_intermediate_nodes_children_map_method() -> TokenStream {
    quote! {
        fn intermediate_nodes_children_map(&self) -> &FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>> {
            self.intermediate_nodes_children_map.get_or_init(|| {
                let mut map: FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>> =
                    FxHashMap::default();
                for (k, v) in &self.intermediate_nodes_children {
                    map.entry(*k).or_default().push(*v);
                }
                map
            })
        }
    }
}

fn gen_add_nonterminal_node_child_method() -> TokenStream {
    quote! {
        fn add_nonterminal_node_child(&mut self, node: SPPFNodeId, child: SPPFNodeId) {
            self.nonterminal_nodes_children.push((node, child));
        }
    }
}

fn gen_nonterminal_nodes_children_map_method() -> TokenStream {
    quote! {
        fn nonterminal_nodes_children_map(&self) -> &FxHashMap<SPPFNodeId, Vec<SPPFNodeId>> {
            self.nonterminal_nodes_children_map.get_or_init(|| {
                let mut map: FxHashMap<SPPFNodeId, Vec<SPPFNodeId>> =
                    FxHashMap::default();
                for (k, v) in &self.nonterminal_nodes_children {
                    map.entry(*k).or_default().push(*v);
                }
                map
            })
        }
    }
}

fn gen_add_trace_event_method() -> TokenStream {
    quote! {
        #[cfg(feature = "debug-trace")]
        fn add_trace_event(&mut self, event: TraceEvent) {
            if let Some(trace_events) = &mut self.trace_events {
                trace_events.push(event);
            }
        }
    }
}

fn gen_parser_struct(
    grammar_name: &str,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds,
) -> TokenStream {
    let nonterminal_ids_len = Literal::usize_unsuffixed(nonterminal_ids.len());
    let terminal_ids_len = Literal::usize_unsuffixed(terminal_ids.len());
    let slot_ids_len = Literal::usize_unsuffixed(slot_ids.len());
    let parser_name_ident =
        syn::Ident::new(&format!("{}{}", grammar_name, "Parser"), Span::call_site());
    let scanner_name_ident =
        syn::Ident::new(&format!("{}{}", grammar_name, "Scanner"), Span::call_site());
    quote! {
        pub struct #parser_name_ident<'i> {
            start_nonterminal: NonterminalId,
            scanner: #scanner_name_ident<'i>,
            descriptors: Vec<Descriptor>,
            gss_nodes: Vec<GSSNode>,
            #[comment="A vector from nonterminal_ids to a tuple (input_index, gss_node_id)"]
            gss_nodes_index: [Vec<(u32, GssNodeId)>; #nonterminal_ids_len],
            sppf_nodes: Vec<SPPFNode>,
            stats: Stats,
            nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; #nonterminal_ids_len],
            intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; #slot_ids_len],
            terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; #terminal_ids_len],
            intermediate_nodes_children: Vec<(SPPFNodeId, (SPPFNodeId, SPPFNodeId))>,
            intermediate_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>>,
            nonterminal_nodes_children: Vec<(SPPFNodeId, SPPFNodeId)>,
            nonterminal_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>>,
            #[cfg(feature = "debug-trace")]
            pub trace_events: Option<Vec<TraceEvent>>,
        }
    }
}

fn gen_parser_impl(
    grammar_name: &str,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds,
) -> TokenStream {
    let new_method = gen_new_method(grammar_name, nonterminal_ids, terminal_ids, slot_ids);
    let name_ident = syn::Ident::new(&format!("{}{}", grammar_name, "Parser"), Span::call_site());
    quote! {
        impl<'i> #name_ident<'i> {
            #new_method
        }
    }
}

fn gen_new_method(
    grammar_name: &str,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds,
) -> TokenStream {
    let name_ident = syn::Ident::new(&format!("{}{}", grammar_name, "Scanner"), Span::call_site());
    let gss_nodes_index_field = gen_gss_nodes_index_field(nonterminal_ids);
    let nonterminal_nodes_index_field = gen_nonterminal_nodes_index_field(nonterminal_ids);
    let intermediate_nodes_index_field = gen_intermediate_nodes_index_field(slot_ids);
    let terminal_nodes_index_field = gen_terminal_nodes_index_field(terminal_ids);
    quote! {
        pub fn new(input: &'i Input, start_nonterminal: NonterminalId) -> Self {
            init_logger();
            Self {
                start_nonterminal,
                scanner: #name_ident::new(input),
                #gss_nodes_index_field,
                descriptors: vec![],
                gss_nodes: vec![],
                sppf_nodes: vec![],
                #nonterminal_nodes_index_field,
                #intermediate_nodes_index_field,
                #terminal_nodes_index_field,
                stats: Stats::default(),
                intermediate_nodes_children: vec![],
                intermediate_nodes_children_map: OnceCell::new(),
                nonterminal_nodes_children: vec![],
                nonterminal_nodes_children_map: OnceCell::new(),
                #[cfg(feature = "debug-trace")]
                trace_events: None,
            }
        }
    }
}

fn gen_gss_nodes_index_field(nonterminal_ids: &NonterminalIds) -> TokenStream {
    let nonterminal_ids_len = Literal::usize_unsuffixed(nonterminal_ids.len());
    quote! {
        gss_nodes_index: [const { vec![] }; #nonterminal_ids_len]
    }
}

fn gen_nonterminal_nodes_index_field(nonterminal_ids: &NonterminalIds) -> TokenStream {
    let nonterminal_ids_len = Literal::usize_unsuffixed(nonterminal_ids.len());
    quote! {
        nonterminal_nodes_index: [const { InlineMap::Empty }; #nonterminal_ids_len]
    }
}

fn gen_intermediate_nodes_index_field(slot_ids: &SlotIds) -> TokenStream {
    let intermediate_ids_len = Literal::usize_unsuffixed(slot_ids.len());
    quote! {
        intermediate_nodes_index: [const { InlineMap::Empty }; #intermediate_ids_len]
    }
}

fn gen_terminal_nodes_index_field(slot_ids: &TerminalIds) -> TokenStream {
    let terminal_ids_len = Literal::usize_unsuffixed(slot_ids.len());
    quote! {
        terminal_nodes_index: [const { InlineMap::Empty }; #terminal_ids_len]
    }
}

fn gen_start_nonterminal_method() -> TokenStream {
    quote! {
        fn start_nonterminal(&self) -> NonterminalId {
            self.start_nonterminal
        }
    }
}    

/// Creates a string representation of a grammar slot of the form `A : a B . c`.
fn slot_to_string(nt_name: &str, seq: &Alternative, pos: usize) -> String {
    let mut s = String::new();
    s.push_str(nt_name);
    s.push_str(" : ");
    for (i, symbol) in seq.symbols.iter().enumerate() {
        if i == pos {
            s.push_str(". ");
        }
        s.push_str(&symbol.to_string());
        if i < seq.len() - 1 {
            s.push(' ');
        }
    }
    // Handle the case where slot is the last grammar position
    if seq.symbols.len() == pos {
        s.push('.');
    }
    s
}
