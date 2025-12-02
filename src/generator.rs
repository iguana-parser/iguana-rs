use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::Ident;

use crate::grammar::symbols::Grammar;
use crate::grammar::symbols::Nonterminal;
use crate::grammar::symbols::Seq;
use crate::grammar::symbols::Symbol;
use crate::grammar::symbols::Terminal;
use crate::parser::NonterminalId;
use crate::parser::SlotId;
use crate::parser::TerminalId;

pub fn generate(grammar: &Grammar) -> String {
    let grammar_name = &grammar.name;
    let mut nonterminal_ids = NonterminalIds::default();
    for nonterminal in grammar.nonterminals() {
        nonterminal_ids.insert(&nonterminal.name);
    }
    let mut slot_ids = SlotIds::default();
    let mut terminal_ids = TerminalIds::default();

    let imports = gen_imports();
    let nonterminals_const = gen_nonterminals_const(&nonterminal_ids);
    let execute_method =
        gen_execute_method(grammar, &nonterminal_ids, &mut slot_ids, &mut terminal_ids);
    let first_descriptors =
        gen_add_first_descriptors_method(grammar, &nonterminal_ids, &mut slot_ids);
    let terminal_const = gen_terminal_const(&terminal_ids);
    let slots_const = gen_slots_const(&slot_ids);
    let nonterminal_name_method = gen_nonterminal_name_method();
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
    let input_len_method = gen_input_len_method();
    let stats_method = gen_stats_method();
    let stats_mut_method = gen_stats_mut_method();
    let lookup_nonterminal_node_method = gen_lookup_nonterminal_node_method();
    let lookup_intermediate_node_method = gen_lookup_intermediate_node_method();
    let lookup_terminal_node_method = gen_lookup_terminal_node_method();
    let gss_nodes_method = gen_gss_nodes_method();
    let add_nonterminal_node_child = gen_add_nonterminal_node_child();
    let add_intermediate_node_child = gen_add_intermediate_node_child();
    let intermediate_nodes_children = gen_intermediate_nodes_children_map();
    let nonterminal_nodes_children = gen_nonterminal_nodes_children_map();
    let parser_struct = gen_parser_struct(grammar_name, &nonterminal_ids, &terminal_ids, &slot_ids);
    let parser_impl = gen_parser_impl(grammar_name, &nonterminal_ids, &terminal_ids, &slot_ids);
    let scanner_struct = gen_scanner_struct(grammar_name);
    let scanner_impl = gen_scanner_impl(grammar_name, &terminal_ids);
    let grammar_name_ident =
        Ident::new(&format!("{}{}", grammar_name, "Parser"), Span::call_site());
    let code = quote! {
        #imports
        #nonterminals_const
        #terminal_const
        #slots_const
        impl<'i> Parser<'i> for #grammar_name_ident<'i> {
            #execute_method
            #first_descriptors
            #nonterminal_name_method
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
            #add_intermediate_node_child
            #add_nonterminal_node_child
            #intermediate_nodes_children
            #nonterminal_nodes_children
        }
        #parser_struct
        #parser_impl
        #scanner_struct
        #scanner_impl
    };
    let tokens = code.to_string();
    let syntax = syn::parse_file(&tokens.to_string()).unwrap();
    prettyplease::unparse(&syntax)
}

fn gen_imports() -> TokenStream {
    quote! {
        use std::cell::OnceCell;
        use iguana::{
            descriptor::Descriptor,
            parser::{NonterminalId, SlotId, TerminalId},
            gss::GSSNode,
            input::Input,
            parser::{Parser, Stats, init_logger},
            scanner::Scanner,
            sppf::{IntermediateNode, NonterminalNode, SPPFNode, SPPFNodeId, Span, TerminalNode},
            utils::inline_map::InlineMap,
        };
        use log::trace;
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
        let name = &nonterminal.name;
        let nt_slot = nonterminal_ids.get_id(name);
        let nt_name = &nonterminal.name;
        let mut alternative_quotes = vec![];
        if let Some(alternatives) = grammar.alternatives(nonterminal) {
            for alternative in alternatives.iter() {
                let slot_name = slot_to_string(nt_name, alternative, 0);
                let first_slot = slot_ids.id(&slot_name);
                alternative_quotes.push(quote! {
                    #[comment = #slot_name]
                    self.add_descriptor(Descriptor::new(#first_slot, None, gss_node_id));
                });
            }
        } else {
            // todo: handle the empty alternative
        }
        nonterminal_quotes.push(quote! {
            #[comment = #nt_name]
            #nt_slot => {
                #( #alternative_quotes)*
            }
        });
    }
    quote! {
        fn add_first_descriptors(&mut self, nonterminal_id: NonterminalId, gss_node_id: usize) {
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
    let nonterminal_ids_len = nonterminal_ids.len();
    let nonterminal_names = nonterminal_ids.nonterminals.iter().map(|n| quote! { #n });
    quote! {
        const NONTERMINALS: [&str; #nonterminal_ids_len] = [#(#nonterminal_names),*];
    }
}

fn gen_terminal_const(terminal_ids: &TerminalIds) -> TokenStream {
    let terminal_ids_len = terminal_ids.len();
    let terminal_names = terminal_ids.terminals.iter().map(|n| quote! { #n });
    quote! {
        const TERMINALS: [&str; #terminal_ids_len] = [#(#terminal_names),*];
    }
}

fn gen_slots_const(slot_ids: &SlotIds) -> TokenStream {
    let slot_ids_len = slot_ids.len();
    let slot_names = slot_ids.slots.iter().map(|s| quote! { #s });
    quote! {
        const SLOTS: [&str; #slot_ids_len] = [#(#slot_names),*];
    }
}

fn gen_execute_method(
    grammar: &Grammar,
    nonterminal_ids: &NonterminalIds,
    slot_ids: &mut SlotIds,
    terminal_ids: &mut TerminalIds,
) -> TokenStream {
    let mut slot_quotes = vec![];
    for nonterminal in grammar.nonterminals() {
        let nt_name = &nonterminal.name;
        let alternatives = grammar.alternatives(nonterminal);
        if let Some(alternatives) = alternatives {
            for alternative in alternatives.iter() {
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
                let last_index = alternative.symbols.len();
                let last_slot_name = slot_to_string(nt_name, alternative, last_index);
                let last_slot_id = slot_ids.id(&last_slot_name);
                let nonterminal_id = nonterminal_ids.get_id(nt_name);
                let last_slot_quote = quote! {
                    #[comment = #last_slot_name]
                    #last_slot_id => {
                        let Some(result) = result else {
                            unreachable!("result cannot be None here.")
                        };
                        let node = self.sppf_node(result);
                        let left_extent = node.left_extent();
                        let right_extent = node.right_extent();
                        let nonterminal_id = #nonterminal_id;
                        let return_slot = #last_slot_id;
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
    }

    quote! {
        fn execute(&mut self, slot_id: SlotId, result: Option<SPPFNodeId>, gss_node_id: usize) {
            trace!(
                "Processing ({}, {}, {})",
                self.slot_name(slot_id),
                self.gss_to_string(gss_node_id),
                if let Some(result) = result {
                    self.sppf_node_to_string(self.sppf_node(result))
                } else {
                    "$".to_string()
                }
            );
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
    alternative: &Seq,
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

fn gen_terminal_slot(
    terminal: &Terminal,
    position: usize,
    slot_name: &str,
    next_slot_name: &str,
    terminal_ids: &mut TerminalIds,
    slot_ids: &mut SlotIds,
) -> TokenStream {
    let terminal_name = &terminal.name;
    let terminal_id = terminal_ids.id(&terminal.name);
    let slot_id = slot_ids.id(slot_name);
    let next_slot_id = slot_ids.id(next_slot_name);
    // At grammar position 0, i.e., A ::= . alpha, the current SPPF node is None.
    // Therefore, we should get the input index from the current GSS node.
    let input_index = if position == 0 {
        quote! {
            let i = self.gss_node(gss_node_id).index;
        }
    } else {
        quote! {
            let left_child_id = result.expect("Result should not be None.");
            let left_child = self.sppf_node(left_child_id);
            let left_extent = left_child.left_extent();
            let i = left_child.right_extent();
        }
    };
    // At grammar position 0, we do not need to create an intermediate node.
    let new_node = if position == 0 {
        quote! {
            let new_node = right_child_id;
            self.execute(next_slot_id, Some(new_node), gss_node_id);
        }
    } else {
        quote! {
            if let Some(new_node) = self.create_intermediate_node_or_attach_children(
                next_slot_id,
                left_extent,
                j,
                left_child_id,
                right_child_id,
            ) {
                self.execute(next_slot_id, Some(new_node), gss_node_id);
            }
        }
    };
    quote! {
        #[comment = #slot_name]
        #slot_id => {
            #input_index
            trace!("Matching terminal {} at input index {i}", #terminal_name);
            let terminal_id = #terminal_id;
            match self.scanner.match_token(terminal_id, i) {
                Some(j) => {
                    trace!("Terminal match successful, index: {j}");
                    let right_child_id = self.get_or_create_terminal_node(terminal_id, i, j);
                    #[comment = #next_slot_name]
                    let next_slot_id = #next_slot_id;
                    #new_node
                }
                None => trace!("Parse error: failed to match '{}' at index {i}", #terminal_name),
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
    let nonterminal_id = nonterminal_ids.get_id(&nonterminal.name);
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
        fn nonterminal_name(&self, nonterminal_id: NonterminalId) -> &str {
            NONTERMINALS[nonterminal_id.index()]
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
        fn get_gss_node(&self, nonterminal_id: NonterminalId, input_index: u32) -> Option<usize> {
            let gss_nodes = &self.gss_nodes_index[nonterminal_id.index()];
            gss_nodes.iter().find(|(k, _)| *k == input_index).map(|x| x.1)
        }
    }
}

fn gen_add_gss_node_method() -> TokenStream {
    quote! {
        fn add_gss_node(&mut self, nonterminal_id: NonterminalId, input_index: u32, gss_node_id: usize) {
            let gss_nodes = &mut self.gss_nodes_index[nonterminal_id.index()];
            gss_nodes.push((input_index, gss_node_id));
        }
    }
}

fn gen_new_gss_node_method() -> TokenStream {
    quote! {
        fn new_gss_node(&mut self, nonterminal_id: NonterminalId, input_index: u32) -> usize {
            let id = self.gss_nodes.len();
            let gss_node = GSSNode::new(id, nonterminal_id, input_index);
            trace!("GSS node ({},{input_index}) created", self.nonterminal_name(nonterminal_id));
            self.gss_nodes.push(gss_node);
            self.stats.gss_nodes_count += 1;
            self.gss_nodes[id].id
        }
    }
}

fn gen_gss_node_method() -> TokenStream {
    quote! {
        fn gss_node(&self, id: usize) -> &GSSNode {
            &self.gss_nodes[id]
        }
    }
}

fn gen_gss_node_mut_method() -> TokenStream {
    quote! {
        fn gss_node_mut(&mut self, id: usize) -> &mut GSSNode {
            self.gss_nodes.get_mut(id).expect("GSS node id should be valid")
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
            trace!("Descriptor added: {}", self.descriptor_to_string(&descriptor));
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
                .insert(terminal_node.span.clone(), terminal_node_id);
            let node = SPPFNode::Terminal(terminal_node);
            trace!("Terminal node created: {}", self.sppf_node_to_string(&node));
            self.sppf_nodes.push(node);
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
                .insert(nonterminal_node.span.clone(), nonterminal_node_id);
            let node = SPPFNode::Nonterminal(nonterminal_node);
            trace!(
                "Nonterminal node created: {}",
                self.sppf_node_to_string(&node),
            );
            self.sppf_nodes.push(node);
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
                .insert(intermediate_node.span.clone(), intermediate_node_id);
            let node = SPPFNode::Intermediate(intermediate_node);
            trace!(
                "Intermediate node created: {}",
                self.sppf_node_to_string(&node)
            );
            self.sppf_nodes.push(node);
            intermediate_node_id
        }
    }
}

fn gen_input_len_method() -> TokenStream {
    quote! {
        fn input_len(&self) -> u32 {
            self.scanner.input.len() as u32
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

fn gen_add_intermediate_node_child() -> TokenStream {
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

fn gen_intermediate_nodes_children_map() -> TokenStream {
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

fn gen_add_nonterminal_node_child() -> TokenStream {
    quote! {
        fn add_nonterminal_node_child(&mut self, node: SPPFNodeId, child: SPPFNodeId) {
            self.nonterminal_nodes_children.push((node, child));
        }
    }
}

fn gen_nonterminal_nodes_children_map() -> TokenStream {
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

fn gen_parser_struct(
    grammar_name: &str,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds,
) -> TokenStream {
    let nonterminal_ids_len = nonterminal_ids.len();
    let terminal_ids_len = terminal_ids.len();
    let slot_ids_len = slot_ids.len();
    let parser_name_ident =
        syn::Ident::new(&format!("{}{}", grammar_name, "Parser"), Span::call_site());
    let scanner_name_ident =
        syn::Ident::new(&format!("{}{}", grammar_name, "Scanner"), Span::call_site());
    quote! {
        pub struct #parser_name_ident<'i> {
            descriptors: Vec<Descriptor>,
            scanner: #scanner_name_ident<'i>,
            gss_nodes: Vec<GSSNode>,
            #[comment="A vector from nonterminal_ids to a tuple (input_index, gss_node_id)"]
            gss_nodes_index: [Vec<(u32, usize)>; #nonterminal_ids_len],
            sppf_nodes: Vec<SPPFNode>,
            stats: Stats,
            nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; #nonterminal_ids_len],
            intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; #slot_ids_len],
            terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; #terminal_ids_len],
            intermediate_nodes_children: Vec<(SPPFNodeId, (SPPFNodeId, SPPFNodeId))>,
            intermediate_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>>,
            nonterminal_nodes_children: Vec<(SPPFNodeId, SPPFNodeId)>,
            nonterminal_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>>,
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
        pub fn new(input: &'i Input) -> Self {
            init_logger();
            Self {
                #gss_nodes_index_field,
                descriptors: vec![],
                scanner: #name_ident::new(input),
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
            }
        }
    }
}

fn gen_gss_nodes_index_field(nonterminal_ids: &NonterminalIds) -> TokenStream {
    let nonterminal_ids_len = nonterminal_ids.len();
    quote! {
        gss_nodes_index: [const { vec![] }; #nonterminal_ids_len]
    }
}

fn gen_nonterminal_nodes_index_field(nonterminal_ids: &NonterminalIds) -> TokenStream {
    let nonterminal_ids_len = nonterminal_ids.len();
    quote! {
        nonterminal_nodes_index: [const { InlineMap::Empty }; #nonterminal_ids_len]
    }
}

fn gen_intermediate_nodes_index_field(slot_ids: &SlotIds) -> TokenStream {
    let intermediate_ids_len = slot_ids.len();
    quote! {
        intermediate_nodes_index: [const { InlineMap::Empty }; #intermediate_ids_len]
    }
}

fn gen_terminal_nodes_index_field(slot_ids: &TerminalIds) -> TokenStream {
    let terminal_ids_len = slot_ids.len();
    quote! {
        terminal_nodes_index: [const { InlineMap::Empty }; #terminal_ids_len]
    }
}

fn gen_scanner_struct(grammar_name: &str) -> TokenStream {
    let name_ident = syn::Ident::new(&format!("{}{}", grammar_name, "Scanner"), Span::call_site());
    quote! {
        pub struct #name_ident<'i> {
            pub input: &'i Input,
        }

        impl<'i> #name_ident<'i> {
            fn new(input: &'i Input) -> Self {
                Self { input }
            }
        }
    }
}

fn gen_scanner_impl(name: &str, terminal_ids: &TerminalIds) -> TokenStream {
    let match_tokens_method = gen_match_token(terminal_ids);
    let name_ident = syn::Ident::new(&format!("{}{}", name, "Scanner"), Span::call_site());
    quote! {
        impl Scanner for #name_ident<'_> {
            #match_tokens_method
        }
    }
}

fn gen_match_token(terminal_ids: &TerminalIds) -> TokenStream {
    let mut match_terminal_id_quotes = vec![];
    for (id, terminal_name) in terminal_ids.terminals.iter().enumerate() {
        let ch = terminal_name.chars().next().unwrap();
        let id = id as u16;
        match_terminal_id_quotes.push(quote! {
            #[comment= #terminal_name]
            TerminalId(#id) => {
                if let Some(c) = self.input.char_at(input_index) && c == #ch {
                    Some(input_index + 1)
                } else {
                    None
                }
            }
        });
    }
    quote! {
        fn match_token(&self, terminal_id: TerminalId, input_index: u32) -> Option<u32> {
            match terminal_id {
                #(#match_terminal_id_quotes)*
                _ => {
                    unreachable!("Unknown token type: {terminal_id}");
                }
            }
        }
    }
}

/// Creates a string representation of a grammar slot of the form `A : a B . c`.
fn slot_to_string(nt_name: &str, seq: &Seq, pos: usize) -> String {
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

#[derive(Default)]
struct NonterminalIds {
    value: usize,
    nonterminal_to_id: HashMap<String, u16>,
    nonterminals: Vec<String>,
}

impl NonterminalIds {
    fn insert(&mut self, name: &str) {
        let value = self.value;
        self.value += 1;
        self.nonterminal_to_id.insert(name.to_owned(), value as u16);
        self.nonterminals.push(name.to_owned());
    }
    fn get_id(&self, name: &str) -> NonterminalId {
        let id = self.nonterminal_to_id.get(name).unwrap();
        NonterminalId(*id)
    }
    fn len(&self) -> usize {
        self.nonterminals.len()
    }
}

#[derive(Default)]
struct SlotIds {
    value: usize,
    slot_to_id: HashMap<String, usize>,
    slots: Vec<String>,
}

impl SlotIds {
    fn id(&mut self, name: &str) -> SlotId {
        if let Some(id) = self.slot_to_id.get(name) {
            SlotId(*id as u16)
        } else {
            let value = self.value;
            self.value += 1;
            self.slot_to_id.insert(name.to_owned(), value);
            self.slots.push(name.to_owned());
            SlotId(value as u16)
        }
    }
    fn len(&self) -> usize {
        self.slots.len()
    }
}

#[derive(Default)]
struct TerminalIds {
    value: usize,
    terminal_ids: HashMap<String, usize>,
    terminals: Vec<String>,
}

impl TerminalIds {
    fn id(&mut self, name: &str) -> TerminalId {
        if let Some(id) = self.terminal_ids.get(name) {
            TerminalId(*id as u16)
        } else {
            let value = self.value;
            self.value += 1;
            self.terminal_ids.insert(name.to_owned(), value);
            self.terminals.push(name.to_owned());
            TerminalId(value as u16)
        }
    }
    fn len(&self) -> usize {
        self.terminal_ids.len()
    }
}

pub fn gen_cargo_toml_file(name: &str) -> String {
    format!(
        r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[profile.release]
debug = true

[lib]
path = "src/lib.rs"

[dependencies]
iguana = {{ path = "/Users/afroozeh/Workspace/iguana-rs" }}
dot = {{ git = "https://github.com/przygienda/dot-rust.git", rev = "fed06f613a9d72bfde711a12791f96a777b2371e" }}
log = "0.4.28"
rustc-hash = "2.1.1"
dhat = "0.3"

[features]
dhat-heap = []
    "#,
        name
    )
    .trim()
    .to_owned()
}

#[cfg(test)]
mod test {
    use crate::{
        generator::generate,
        grammar::symbols::{Grammar, Nonterminal, Seq, Symbol, Terminal},
    };
    use std::io::Result;

    #[test]
    fn test1() -> Result<()> {
        // A ::= 'a' 'b'
        let grammar = Grammar::builder()
            .name("Test".to_string())
            .add_production(
                Nonterminal::new("A"),
                Seq::builder()
                    .add_symbol(Symbol::Terminal(Terminal::new("a")))
                    .add_symbol(Symbol::Terminal(Terminal::new("b")))
                    .build(),
            )
            .start_symbol(Nonterminal::new("A"))
            .build();
        let output = generate(&grammar);
        println!("{output}");
        Ok(())
    }

    #[test]
    fn test2() -> Result<()> {
        // A ::= B
        // B ::= 'b'
        let grammar = Grammar::builder()
            .name("Test2".to_string())
            .add_production(
                Nonterminal::new("A"),
                Seq::builder()
                    .add_symbol(Symbol::Nonterminal(Nonterminal::new("B")))
                    .build(),
            )
            .add_production(
                Nonterminal::new("B"),
                Seq::builder()
                    .add_symbol(Symbol::Terminal(Terminal::new("b")))
                    .build(),
            )
            .start_symbol(Nonterminal::new("A"))
            .build();
        let output = generate(&grammar);
        println!("{output}");
        Ok(())
    }

    #[test]
    fn test3() -> Result<()> {
        // A ::= A B 'c'
        // B ::= 'b'
        // A ::= 'a'
        let grammar = Grammar::builder()
            .name("Test2".to_string())
            .add_production(
                Nonterminal::new("A"),
                Seq::builder()
                    .add_symbol(Symbol::Nonterminal(Nonterminal::new("A")))
                    .build(),
            )
            .add_production(
                Nonterminal::new("A"),
                Seq::builder()
                    .add_symbol(Symbol::Terminal(Terminal::new("a")))
                    .build(),
            )
            .start_symbol(Nonterminal::new("A"))
            .build();
        let output = generate(&grammar);
        println!("{output}");
        Ok(())
    }

    #[test]
    fn test4() -> Result<()> {
        // E ::= E '+' E
        // E ::= 'a'
        let grammar = Grammar::builder()
            .name("Test2".to_string())
            .add_production(
                Nonterminal::new("E"),
                Seq::builder()
                    .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                    .add_symbol(Symbol::Terminal(Terminal::new("+")))
                    .add_symbol(Symbol::Nonterminal(Nonterminal::new("E")))
                    .build(),
            )
            .add_production(
                Nonterminal::new("E"),
                Seq::builder()
                    .add_symbol(Symbol::Terminal(Terminal::new("a")))
                    .build(),
            )
            .start_symbol(Nonterminal::new("A"))
            .build();
        let output = generate(&grammar);
        println!("{output}");
        Ok(())
    }
}
