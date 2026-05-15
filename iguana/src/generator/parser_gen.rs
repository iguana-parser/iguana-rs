use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::generator::GenConfig;
use crate::generator::id::NonterminalIds;
use crate::generator::id::SlotIds;
use crate::generator::id::TerminalIds;
use crate::grammar::def::Alternative;
use crate::grammar::def::Grammar;
use crate::grammar::first_follow::FirstFollowSets;
use crate::grammar::slot::Slot;
use crate::grammar::symbols::CondOp;
use crate::grammar::symbols::Definition;
use crate::grammar::symbols::Expr;
use crate::grammar::symbols::Identifier;
use crate::grammar::symbols::Nonterminal;
use crate::grammar::symbols::Parameter;
use crate::grammar::symbols::Symbol;
use crate::grammar::symbols::Terminal;
use crate::ids::NonterminalId;
use crate::ids::SlotId;
use crate::utils::to_first_uppercase;
use crate::utils::to_snake_case;

pub struct ParserGen<'a> {
    grammar: &'a Grammar,
    nonterminal_ids: &'a NonterminalIds,
    terminal_ids: &'a TerminalIds,
    slot_ids: &'a SlotIds<'a>,
    ff: FirstFollowSets<'a>,
    config: GenConfig,
}

impl<'a> ParserGen<'a> {
    pub fn new(
        grammar: &'a Grammar,
        nonterminal_ids: &'a NonterminalIds,
        terminal_ids: &'a TerminalIds,
        slot_ids: &'a SlotIds<'a>,
        config: GenConfig,
    ) -> Self {
        Self {
            grammar,
            nonterminal_ids,
            terminal_ids,
            slot_ids,
            ff: FirstFollowSets::new(grammar),
            config,
        }
    }

    fn is_layout(&self, nonterminal: &Nonterminal) -> bool {
        self.grammar
            .layout
            .as_ref()
            .and_then(|s| s.as_identifier())
            .map(|i| i.name == nonterminal.name)
            .unwrap_or(false)
    }

    pub fn generate(&mut self) -> TokenStream {
        let grammar_name = &self.grammar.name;
        let imports = self.gen_imports();
        let execute_method = self.gen_execute_method();
        let first_descriptors = self.gen_add_first_descriptors_method();
        let nonterminal_display_name_method = Self::gen_nonterminal_display_name_method();
        let terminal_name_method = Self::gen_terminal_name_method();
        let slot_name_method = Self::gen_slot_name_method();
        let epsilon_method = Self::gen_epsilon_method();
        let eof_method = Self::gen_eof_method();
        let get_gss_node_method = Self::gen_get_gss_node_method();
        let gen_add_gss_node_method = Self::gen_add_gss_node_method();
        let gen_new_gss_node_method = Self::gen_new_gss_node_method();
        let gss_node_method = Self::gen_gss_node_method();
        let gss_node_mut_method = Self::gen_gss_node_mut_method();
        let sppf_node_method = Self::gen_sppf_node_method();
        let sppf_node_mut_method = Self::gen_sppf_node_mut_method();
        let add_descriptor_method = Self::gen_add_descriptor_method();
        let next_descriptor_method = Self::gen_next_descriptor_method();
        let new_terminal_node_method = Self::gen_add_terminal_node_method();
        let new_nonterminal_node_method = Self::gen_add_nonterminal_node_method();
        let new_intermediate_node_method = Self::gen_add_intermediate_node_method();
        let input_len_method = Self::gen_input_method();
        let sppf_nodes_method = Self::gen_sppf_nodes_method();
        let increment_descriptor_count_method = Self::gen_increment_descriptor_count_method();
        let count_methods = Self::gen_count_methods();
        let lookup_nonterminal_node_method = Self::gen_lookup_nonterminal_node_method();
        let lookup_intermediate_node_method = Self::gen_lookup_intermediate_node_method();
        let lookup_terminal_node_method = Self::gen_lookup_terminal_node_method();
        let gss_nodes_method = Self::gen_gss_nodes_method();
        let add_nonterminal_node_child_method = Self::gen_add_nonterminal_node_child_method();
        let add_intermediate_node_child_method = Self::gen_add_intermediate_node_child_method();
        let intermediate_nodes_children_method = Self::gen_intermediate_nodes_children_map_method();
        let nonterminal_nodes_children_method = Self::gen_nonterminal_nodes_children_map_method();
        let add_trace_event_method = Self::gen_add_trace_event_method();
        let start_nonterminal_method = Self::gen_start_nonterminal_method();
        let start_env_method = self.gen_start_env_method();
        let lookup_start_nonterminal_node_method = self.gen_lookup_start_nonterminal_node_method();
        let add_start_gss_node_method = self.gen_add_start_gss_node_method();
        let new_env_method = Self::gen_new_env_method();
        let lookup_method = Self::gen_lookup_method();
        let clone_env_method = Self::gen_clone_env();
        let envs_method = Self::gen_envs_method();
        let record_stats_method = Self::gen_record_stats_method();
        let post_conditions_method = self.gen_post_conditions_method();
        let follow_set_check_method = self.gen_follow_set_check_method();
        let follow_set_terminals_method = self.gen_follow_set_terminals_method();
        let parser_struct = self.gen_parser_struct();
        let parser_impl = self.gen_parser_impl();
        let grammar_name_ident = format_ident!("{}Parser", to_first_uppercase(grammar_name));
        quote! {
            #imports
            impl<'i> Parser<'i> for #grammar_name_ident<'i> {
                #nonterminal_display_name_method
                #terminal_name_method
                #slot_name_method
                #epsilon_method
                #eof_method
                #execute_method
                #first_descriptors
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
                #sppf_nodes_method
                #increment_descriptor_count_method
                #count_methods
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
                #start_env_method
                #lookup_start_nonterminal_node_method
                #add_start_gss_node_method
                #new_env_method
                #lookup_method
                #clone_env_method
                #envs_method
                #record_stats_method
                #post_conditions_method
                #follow_set_check_method
                #follow_set_terminals_method

                fn parse_error(&self) -> Option<&ParseError> {
                    self.parse_errors.first()
                }

                fn add_parse_error(
                    &mut self,
                    input_index: u32,
                    slot_id: SlotId,
                    gss_node_id: Option<GssNodeId>,
                    kind: impl FnOnce() -> ParseErrorKind,
                ) {
                    let level = self.parse_errors.first().map_or(0, |e| e.input_index);
                    if input_index < level {
                        record!(self, ParseError, input_index, slot_id, gss_node_id, kind());
                        return;
                    }
                    let kind = kind();
                    record!(self, ParseError, input_index, slot_id, gss_node_id, kind.clone());
                    if input_index > level {
                        self.parse_errors.clear();
                    }
                    self.parse_errors.push(ParseError {
                        input_index,
                        slot_id,
                        gss_node_id,
                        kind,
                    });
                }

                fn match_token(&mut self, terminal_id: TerminalId, input_index: u32) -> Option<u32> {
                    self.scanner.match_token(terminal_id, input_index)
                }
            }
            #parser_struct
            #parser_impl
        }
    }

    fn gen_imports(&self) -> TokenStream {
        let scanner_name = format_ident!("{}Scanner", to_first_uppercase(&self.grammar.name));
        quote! {
            use std::cell::OnceCell;
            use crate::{grammar_data::*, scanner::#scanner_name};
            use iguana_runtime::{
                descriptor::Descriptor,
                env::{Env, EnvId},
                gss::GSSNode,
                ids::{GssNodeId, NonterminalId, SlotId, TerminalId},
                input::Input,
                parser::{Parser, ParseError, ParseErrorKind, init_logger, GSS_CAPACITY_MULTIPLIER, SPPF_CAPACITY_MULTIPLIER},
                record,
                scanner::Scanner,
                sppf::{IntermediateNode, NonterminalNode, SPPFNode, SPPFNodeId, Span, TerminalNode},
                utils::{inline_map::InlineMap, inline_vec::InlineVec}
            };
            #[cfg(feature = "debug-trace")]
            use iguana_runtime::trace::TraceEvent;
            use rustc_hash::FxHashMap;
        }
    }

    fn gen_execute_method(&self) -> TokenStream {
        let mut slot_quotes = vec![];
        for nonterminal in self.grammar.nonterminals() {
            let alternatives = self.grammar.alternatives(nonterminal);
            for alternative in alternatives.iter() {
                for pos in 0..alternative.symbols.len() {
                    let slot = Slot::new(nonterminal, alternative, pos);
                    slot_quotes.push(self.gen_slot_code(slot));
                }
                // Handle the last grammar slot
                let last_symbol_index = alternative.symbols.len();
                let end_slot = Slot::new(nonterminal, alternative, last_symbol_index);
                let end_slot_name = end_slot.name();
                let end_slot_id = self.slot_ids.get_id(&end_slot);
                let nonterminal_id = self.nonterminal_ids.get_id(nonterminal);
                // Handles the case for an empty alternative
                let last_slot_quote = if last_symbol_index == 0 {
                    quote! {
                        #[comment = #end_slot_name]
                        #end_slot_id => {
                            let epsilon_node_id = self.get_or_create_epsilon_node(input_index);
                            let nonterminal_node_id = self.get_or_create_nonterminal_node(
                                #nonterminal_id,
                                #end_slot_id,
                                input_index,
                                input_index,
                                epsilon_node_id,
                                true,
                            );
                            self.pop(gss_node_id, #end_slot_id, nonterminal_node_id, None);
                        }
                    }
                } else {
                    let last_symbol = alternative.symbols.last().unwrap();
                    let slot_body = if let Symbol::Return(expr) = last_symbol {
                        let expr = Self::gen_expr(expr);
                        let create_method = format_ident!(
                            "create_nonterminal_node_or_attach_children_{}",
                            to_snake_case(&nonterminal.name)
                        );
                        quote! {
                            let Some(result) = result else {
                                unreachable!("result cannot be None here.")
                            };
                            let node = self.sppf_node(result);
                            let return_value = #expr;
                            let nonterminal_node_id = self.#create_method(
                                #nonterminal_id,
                                #end_slot_id,
                                node.left_extent(),
                                node.right_extent(),
                                result,
                                return_value,
                            );
                            self.pop(gss_node_id, #end_slot_id, nonterminal_node_id, Some(return_value));
                        }
                    } else {
                        quote! {
                            let nonterminal_node_id = self.create_nonterminal_node(
                                result, #nonterminal_id, #end_slot_id,
                            );
                            self.pop(gss_node_id, #end_slot_id, nonterminal_node_id, None);
                        }
                    };
                    quote! {
                        #[comment = #end_slot_name]
                        #end_slot_id => {
                            #slot_body
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
                gss_node_id: GssNodeId,
                env: Option<EnvId>,
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

    fn gen_add_first_descriptors_method(&self) -> TokenStream {
        let mut nonterminal_quotes = vec![];
        for nonterminal in self.grammar.nonterminals() {
            let nonterminal_id = self.nonterminal_ids.get_id(nonterminal);
            let alternatives = self.grammar.alternatives(nonterminal);
            if alternatives.len() == 1 {
                let first_slot = Slot::new(nonterminal, &alternatives[0], 0);
                let first_slot_name = first_slot.name();
                let first_slot_id = self.slot_ids.get_id(&first_slot);
                nonterminal_quotes.push(quote! {
                    #[comment = #first_slot_name]
                    #nonterminal_id => {
                        self.add_first_descriptor(#first_slot_id, input_index, gss_node_id, env);
                    }
                });
            } else {
                nonterminal_quotes.push(self.gen_multi_alt_first_dispatch(nonterminal));
            }
        }
        quote! {
            fn add_first_descriptors(
                &mut self,
                nonterminal_id: NonterminalId,
                input_index: u32,
                gss_node_id: GssNodeId,
                env: Option<EnvId>,
            ) {
                match nonterminal_id {
                    #(#nonterminal_quotes)*
                    _ => {
                        panic!("Unknown nonterminal id: {nonterminal_id}");
                    }
                }
            }
        }
    }

    /// Emits the GLL dispatch for a multi-alternative nonterminal A.
    ///
    /// Each alternative `A → α` tests `FIRST(α)`; nullable alts also fall
    /// through to `FOLLOW(A)`. The disjunction short-circuits, so FOLLOW is
    /// scanned only when FIRST misses. Repeat FOLLOW scans across multiple
    /// nullable alts hit the scanner's `(position, terminal)` memo and are
    /// near-free. If no alternative spawned, record a parse error.
    fn gen_multi_alt_first_dispatch(&self, nonterminal: &'a Nonterminal) -> TokenStream {
        let nt_name = &nonterminal.name;
        let nonterminal_id = self.nonterminal_ids.get_id(nonterminal);
        let nt_const_suffix = to_snake_case(&nonterminal.name).to_uppercase();
        let alternatives = self.grammar.alternatives(nonterminal);
        let follow_name = format_ident!("FOLLOW_SET_{}", nt_const_suffix);

        let alt_arms: Vec<_> = alternatives
            .iter()
            .enumerate()
            .map(|(alt_index, alt)| {
                let first_alt_name =
                    format_ident!("FIRST_SET_{}_ALT{}", nt_const_suffix, alt_index);
                let slot = Slot::new(nonterminal, alt, 0);
                let slot_id = self.slot_ids.get_id(&slot);
                let slot_name = slot.name();
                let trigger = if self.ff.is_alt_nullable(alt) {
                    quote! {
                        self.scanner.match_any(#first_alt_name, input_index)
                            || self.scanner.match_any(#follow_name, input_index)
                    }
                } else {
                    quote! { self.scanner.match_any(#first_alt_name, input_index) }
                };
                quote! {
                    #[comment = #slot_name]
                    if #trigger {
                        matched = true;
                        self.add_first_descriptor(#slot_id, input_index, gss_node_id, env);
                    }
                }
            })
            .collect();

        let first_set_name = format_ident!("FIRST_SET_{}", nt_const_suffix);
        let first_slot_id = self
            .slot_ids
            .get_id(&Slot::new(nonterminal, &alternatives[0], 0));

        // Nullable NTs accept FOLLOW tokens as valid continuations via the ε arm.
        let expected = if self.ff.is_nonterminal_nullable(nonterminal) {
            quote! {
                {
                    let mut expected = #first_set_name.to_vec();
                    expected.extend_from_slice(#follow_name);
                    expected
                }
            }
        } else {
            quote! { #first_set_name.to_vec() }
        };

        quote! {
            #[comment = #nt_name]
            #nonterminal_id => {
                let mut matched = false;
                #(#alt_arms)*
                if !matched {
                    self.add_parse_error(input_index, #first_slot_id, Some(gss_node_id), || {
                        ParseErrorKind::UnexpectedToken { expected: #expected }
                    });
                }
            }
        }
    }

    fn gen_slot_code(&self, slot: Slot<'a>) -> TokenStream {
        match slot.symbol() {
            Some(Symbol::Condition(expr)) => {
                return self.gen_condition_code(expr, &slot);
            }
            Some(Symbol::Return(_)) => {
                return self.gen_return_code(&slot);
            }
            Some(Symbol::Except { symbol, except }) => {
                return self.gen_except_code(symbol, except.as_slice(), &slot);
            }
            Some(Symbol::FollowRestriction { symbol, .. }) => {
                return self.gen_follow_restriction_code(symbol, &slot);
            }
            Some(Symbol::PrecedeRestriction {
                symbol,
                restriction,
            }) => {
                return self.gen_precede_restriction_code(symbol, restriction, &slot);
            }
            Some(
                Symbol::Labeled { .. }
                | Symbol::Identifier(_)
                | Symbol::Literal(_)
                | Symbol::Group(_)
                | Symbol::Opt(_)
                | Symbol::Alt(_)
                | Symbol::Star(_, _)
                | Symbol::Plus(_, _)
                | Symbol::Call { .. }
                | Symbol::Binding { .. },
            )
            | None => {}
            Some(Symbol::Exclude { .. }) => {
                unreachable!("Exclude should be desugared before code generation")
            }
        }
        let symbol = slot.symbol().unwrap();
        if let Some(identifier) = symbol.as_identifier() {
            let def_id = identifier.resolve();
            let def = self.grammar.definition(def_id);
            match def {
                Definition::Terminal(terminal) => self.gen_terminal_slot(terminal, slot, &[], &[]),
                Definition::Nonterminal(nonterminal) => {
                    let arguments = match symbol.unwrap() {
                        Symbol::Call { arguments, .. } => arguments.clone(),
                        Symbol::Labeled { .. }
                        | Symbol::Identifier(_)
                        | Symbol::Literal(_)
                        | Symbol::Group(_)
                        | Symbol::Opt(_)
                        | Symbol::Alt(_)
                        | Symbol::Star(_, _)
                        | Symbol::Plus(_, _)
                        | Symbol::Except { .. }
                        | Symbol::FollowRestriction { .. }
                        | Symbol::PrecedeRestriction { .. }
                        | Symbol::Condition(_)
                        | Symbol::Return(_)
                        | Symbol::Binding { .. } => vec![],
                        Symbol::Exclude { .. } => {
                            unreachable!("Exclude should be desugared before code generation")
                        }
                    };
                    self.gen_nonterminal_slot(nonterminal, &arguments, slot, &[], &[])
                }
            }
        } else {
            quote! {}
        }
    }

    /// Generates code for the grammar slots before a terminal.
    fn gen_terminal_slot(
        &self,
        terminal: &Terminal,
        slot: Slot<'a>,
        pre_conditions: &[TokenStream],
        post_conditions: &[TokenStream],
    ) -> TokenStream {
        let terminal_id = &self.terminal_ids.get_id(terminal);
        let slot_id = self.slot_ids.get_id(&slot);
        let current_slot_name = slot.name();
        let next_slot = slot.next();
        let next_slot_id = self.slot_ids.get_id(&next_slot);
        let next_slot_name = next_slot.name();
        // At grammar position 0, we do not need to create an intermediate node.
        let new_node = if slot.is_first() {
            quote! {
                #[comment = #next_slot_name]
                self.execute(j, #next_slot_id, Some(right_child), gss_node_id, env);
            }
        } else {
            quote! {
                if let Some((j, new_node)) = self.create_intermediate_node(
                    result, right_child, #next_slot_id,
                ) {
                    #[comment = #next_slot_name]
                    self.execute(j, #next_slot_id, Some(new_node), gss_node_id, env);
                }
            }
        };
        let new_node = if post_conditions.is_empty() {
            new_node
        } else {
            quote! {
                if let Some(error_kind) = self.post_conditions(#next_slot_id, input_index, j) {
                    self.add_parse_error(j, #next_slot_id, Some(gss_node_id), || error_kind);
                } else {
                    #new_node
                }
            }
        };
        let pre_condition_check = if pre_conditions.is_empty() {
            quote! {}
        } else {
            quote! {
                if !(#(#pre_conditions)&&*) { return; }
            }
        };
        let terminal_name = &terminal.name;
        let uses_j = slot.is_first() || !post_conditions.is_empty();
        let destructure = if uses_j {
            quote! { (j, right_child) }
        } else {
            quote! { (_, right_child) }
        };
        quote! {
            #[comment = #current_slot_name]
            #slot_id => {
                #pre_condition_check
                if let Some(#destructure) = self.match_terminal(#terminal_id, input_index, #slot_id, Some(gss_node_id), #terminal_name) {
                    #new_node
                }
            }
        }
    }

    fn gen_except_code(
        &self,
        symbol: &Symbol,
        except: &[Identifier],
        slot: &Slot<'a>,
    ) -> TokenStream {
        let Some(identifier) = symbol.as_identifier() else {
            return quote! {};
        };
        let def_id = identifier.resolve();
        let def = &self.grammar.definition(def_id);
        let except_terminal_ids: Vec<_> = except
            .iter()
            .map(|e| {
                let except_def_id = e.resolve();
                let Definition::Terminal(except_terminal) = &self.grammar.definition(except_def_id)
                else {
                    panic!("Except identifier must resolve to a terminal");
                };
                self.terminal_ids.get_id(except_terminal)
            })
            .collect();
        match def {
            Definition::Terminal(terminal) => {
                let post_conditions: Vec<_> = except_terminal_ids
                    .iter()
                    .map(|id| {
                        quote! {
                            self.scanner.match_token(#id, input_index) != Some(j)
                        }
                    })
                    .collect();
                self.gen_terminal_slot(terminal, slot.clone(), &[], &post_conditions)
            }
            Definition::Nonterminal(nonterminal) => {
                let arguments = match symbol.unwrap() {
                    Symbol::Call { arguments, .. } => arguments.clone(),
                    _ => vec![],
                };
                let post_conditions: Vec<_> = except_terminal_ids
                    .iter()
                    .map(|id| {
                        quote! {
                            self.scanner.match_token(#id, input_index) != Some(j)
                        }
                    })
                    .collect();
                self.gen_nonterminal_slot(
                    nonterminal,
                    &arguments,
                    slot.clone(),
                    &[],
                    &post_conditions,
                )
            }
        }
    }

    fn gen_follow_restriction_code(&self, symbol: &Symbol, slot: &Slot<'a>) -> TokenStream {
        let Some(identifier) = symbol.as_identifier() else {
            return quote! {};
        };
        let def_id = identifier.resolve();
        let def = &self.grammar.definition(def_id);
        // The actual restriction check is in `post_conditions`. Here we just
        // signal that post-conditions exist so the slot codegen wraps the
        // continuation with a `post_conditions` call.
        let has_post_conditions = &[quote! {}];
        match def {
            Definition::Terminal(terminal) => {
                self.gen_terminal_slot(terminal, slot.clone(), &[], has_post_conditions)
            }
            Definition::Nonterminal(nonterminal) => {
                let arguments = match symbol.unwrap() {
                    Symbol::Call { arguments, .. } => arguments.clone(),
                    _ => vec![],
                };
                self.gen_nonterminal_slot(
                    nonterminal,
                    &arguments,
                    slot.clone(),
                    &[],
                    has_post_conditions,
                )
            }
        }
    }

    fn gen_precede_restriction_code(
        &self,
        symbol: &Symbol,
        restriction: &Identifier,
        slot: &Slot<'a>,
    ) -> TokenStream {
        let Some(identifier) = symbol.as_identifier() else {
            return quote! {};
        };
        let def_id = identifier.resolve();
        let def = &self.grammar.definition(def_id);
        let restriction_def_id = restriction.resolve();
        let Definition::Terminal(restriction_terminal) =
            self.grammar.definition(restriction_def_id)
        else {
            panic!("Precede restriction identifier must resolve to a terminal");
        };
        let restriction_terminal_id = self.terminal_ids.get_id(restriction_terminal);
        match def {
            Definition::Terminal(terminal) => {
                let pre_conditions = vec![quote! {
                    input_index == 0 || self.scanner.match_token(#restriction_terminal_id, input_index - 1).is_none()
                }];
                self.gen_terminal_slot(terminal, slot.clone(), &pre_conditions, &[])
            }
            Definition::Nonterminal(nonterminal) => {
                let arguments = match symbol.unwrap() {
                    Symbol::Call { arguments, .. } => arguments.clone(),
                    _ => vec![],
                };
                let pre_conditions = vec![quote! {
                    input_index == 0 || self.scanner.match_token(#restriction_terminal_id, input_index - 1).is_none()
                }];
                self.gen_nonterminal_slot(
                    nonterminal,
                    &arguments,
                    slot.clone(),
                    &pre_conditions,
                    &[],
                )
            }
        }
    }

    fn gen_post_conditions_method(&mut self) -> TokenStream {
        let mut arms = vec![];
        for nonterminal in self.grammar.nonterminals() {
            for (alt_index, alternative) in
                self.grammar.alternatives(nonterminal).iter().enumerate()
            {
                for pos in 0..alternative.symbols.len() {
                    let symbol = &alternative.symbols[pos];
                    if let Symbol::Except { symbol, except } = symbol {
                        let Some(identifier) = symbol.as_identifier() else {
                            continue;
                        };
                        let except_ids: Vec<_> = except
                            .iter()
                            .map(|e| {
                                let except_def_id = e.resolve();
                                let Definition::Terminal(except_terminal) =
                                    self.grammar.definition(except_def_id)
                                else {
                                    panic!("Except identifier must resolve to a terminal");
                                };
                                self.terminal_ids.get_id(except_terminal)
                            })
                            .collect();
                        let checks: Vec<_> = except_ids
                            .iter()
                            .map(|id| {
                                quote! {
                                    self.scanner.match_token(#id, left_extent) == Some(right_extent)
                                }
                            })
                            .collect();
                        let slot = Slot::new(nonterminal, alternative, pos);
                        let next_slot = slot.next();
                        let slot_id = self.slot_ids.get_id(&next_slot);
                        arms.push(quote! {
                            #slot_id => {
                                if #(#checks)||* {
                                    Some(ParseErrorKind::ExcludedMatch {
                                        excluded_by: vec![#(#except_ids),*],
                                    })
                                } else {
                                    None
                                }
                            }
                        });
                    }
                    if let Symbol::FollowRestriction { symbol, .. } = symbol {
                        let Some(_) = symbol.as_identifier() else {
                            continue;
                        };
                        let static_name = format_ident!(
                            "FOLLOW_RESTRICTION_{}_ALT{}_POS{}",
                            to_snake_case(&nonterminal.name).to_uppercase(),
                            alt_index,
                            pos
                        );
                        let slot = Slot::new(nonterminal, alternative, pos);
                        let next_slot = slot.next();
                        let slot_id = self.slot_ids.get_id(&next_slot);
                        arms.push(quote! {
                            #slot_id => {
                                if self.scanner.match_any(#static_name, right_extent) {
                                    Some(ParseErrorKind::ForbiddenFollow {
                                        forbidden: #static_name.to_vec(),
                                    })
                                } else {
                                    None
                                }
                            }
                        });
                    }
                }
            }
        }
        quote! {
            fn post_conditions(&mut self, slot: SlotId, left_extent: u32, right_extent: u32) -> Option<ParseErrorKind> {
                match slot {
                    #(#arms)*
                    _ => None,
                }
            }
        }
    }

    fn gen_follow_set_check_method(&self) -> TokenStream {
        let mut arms = vec![];
        for nonterminal in self.grammar.nonterminals() {
            let nonterminal_id = self.nonterminal_ids.get_id(nonterminal);
            let condition = self.gen_match_any(
                &format!(
                    "FOLLOW_SET_{}",
                    to_snake_case(&nonterminal.name).to_uppercase()
                ),
                quote! { input_index },
            );
            arms.push(quote! {
                #nonterminal_id => { #condition }
            });
        }
        quote! {
            fn follow_set_check(&mut self, nonterminal_id: NonterminalId, input_index: u32) -> bool {
                match nonterminal_id {
                    #(#arms)*
                    _ => true,
                }
            }
        }
    }

    fn gen_follow_set_terminals_method(&self) -> TokenStream {
        let mut arms = vec![];
        for nonterminal in self.grammar.nonterminals() {
            let nonterminal_id = self.nonterminal_ids.get_id(nonterminal);
            let follow_name = format_ident!(
                "FOLLOW_SET_{}",
                to_snake_case(&nonterminal.name).to_uppercase(),
            );
            arms.push(quote! {
                #nonterminal_id => { #follow_name.to_vec() }
            });
        }
        quote! {
            fn follow_set_terminals(&self, nonterminal_id: NonterminalId) -> Vec<TerminalId> {
                match nonterminal_id {
                    #(#arms)*
                    _ => vec![],
                }
            }
        }
    }

    fn gen_nonterminal_slot(
        &self,
        nonterminal: &'a Nonterminal,
        arguments: &[Expr],
        slot: Slot<'a>,
        pre_conditions: &[TokenStream],
        post_conditions: &[TokenStream],
    ) -> TokenStream {
        let slot_id = self.slot_ids.get_id(&slot);
        let slot_name = slot.name();
        let next_slot = slot.next();
        let next_slot_id = self.slot_ids.get_id(&next_slot);
        let pre_condition_check = if pre_conditions.is_empty() {
            quote! {}
        } else {
            quote! { if !(#(#pre_conditions)&&*) { return; } }
        };
        if self.config.ll1_optimization && self.ff.is_ll1(nonterminal) {
            let next_slot_name = next_slot.name();
            let post_condition_check = if post_conditions.is_empty() {
                quote! {}
            } else {
                quote! {
                    if let Some(error_kind) = self.post_conditions(#next_slot_id, input_index, j) {
                        self.add_parse_error(j, #next_slot_id, Some(gss_node_id), || error_kind);
                        return;
                    }
                }
            };
            let method_name = format_ident!("parse_{}_ll1", to_snake_case(&nonterminal.name));
            if slot.is_first() {
                quote! {
                    #[comment = #slot_name]
                    #slot_id => {
                        #pre_condition_check
                        if let Some(right_child) = self.#method_name(input_index) {
                            let j = self.sppf_node(right_child).right_extent();
                            #post_condition_check
                            #[comment = #next_slot_name]
                            self.execute(j, #next_slot_id, Some(right_child), gss_node_id, env);
                        }
                    }
                }
            } else {
                let compute_j = if post_conditions.is_empty() {
                    quote! {}
                } else {
                    quote! { let j = self.sppf_node(right_child).right_extent(); }
                };
                quote! {
                    #[comment = #slot_name]
                    #slot_id => {
                        #pre_condition_check
                        if let Some(right_child) = self.#method_name(input_index) {
                            #compute_j
                            #post_condition_check
                            if let Some((j, new_node)) = self.create_intermediate_node(
                                result, right_child, #next_slot_id,
                            ) {
                                #[comment = #next_slot_name]
                                self.execute(j, #next_slot_id, Some(new_node), gss_node_id, env);
                            }
                        }
                    }
                }
            }
        } else if nonterminal.parameters.is_empty() {
            let nonterminal_id = self.nonterminal_ids.get_id(nonterminal);
            quote! {
                #[comment = #slot_name]
                #slot_id => {
                    #pre_condition_check
                    self.create(#nonterminal_id, result, gss_node_id, #next_slot_id, env);
                }
            }
        } else {
            let method_name = format_ident!("create_{}", to_snake_case(&nonterminal.name));
            let arguments: Vec<_> = arguments.iter().map(Self::gen_expr).collect();
            let bindings = if let Some(Symbol::Binding { name, .. }) = slot.symbol() {
                quote! { Some(#name) }
            } else {
                quote! { None }
            };
            let arguments =
                quote! { result, gss_node_id, #next_slot_id, env, #bindings, #(#arguments),* };
            quote! {
                #[comment = #slot_name]
                #slot_id => {
                    #pre_condition_check
                    self.#method_name(#arguments);
                }
            }
        }
    }

    fn gen_condition_code(&self, expr: &Expr, slot: &Slot<'a>) -> TokenStream {
        let slot_id = self.slot_ids.get_id(slot);
        let slot_name = slot.name();
        let next_slot = slot.next();
        let next_slot_id = self.slot_ids.get_id(&next_slot);
        let condition_expr = Self::gen_expr(expr);
        quote! {
            #[comment = #slot_name]
            #slot_id => {
                if #condition_expr {
                    self.execute(input_index, #next_slot_id, result, gss_node_id, env);
                }
            }
        }
    }

    fn gen_return_code(&self, slot: &Slot<'a>) -> TokenStream {
        let slot_id = self.slot_ids.get_id(slot);
        let slot_name = slot.name();
        let next_slot = slot.next();
        let next_slot_id = self.slot_ids.get_id(&next_slot);
        quote! {
            #[comment = #slot_name]
            #slot_id => {
                self.execute(input_index, #next_slot_id, result, gss_node_id, env);
            }
        }
    }

    fn gen_parser_impl(&mut self) -> TokenStream {
        let grammar_name = &self.grammar.name;
        let new_method = self.gen_new_method();
        let name_ident = format_ident!("{}{}", grammar_name, "Parser");
        let create_methods: Vec<_> = self
            .nonterminal_ids
            .nonterminals()
            .enumerate()
            .filter(|(_, n)| !n.parameters.is_empty())
            .map(|(i, n)| Self::gen_create_method(n, i))
            .collect();
        let ll1_parse_methods: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|nt| self.config.ll1_optimization && self.ff.is_ll1(nt))
            .map(|nt| self.gen_parse_method_ll1(nt, self.is_layout(nt)))
            .collect();
        let get_gss_node_methods: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(Self::gen_get_gss_node_method_with_parameters)
            .collect();
        let add_gss_node_methods: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(Self::gen_add_gss_node_method_with_parameters)
            .collect();
        let specialized_lookup_nonterminal_node_methods: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(Self::gen_specialized_lookup_nonterminal_node_method)
            .collect();
        let specialized_add_nonterminal_node_methods: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(Self::gen_specialized_add_nonterminal_node_method)
            .collect();
        let create_nonterminal_node_or_attach_children_methods: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(Self::gen_create_nonterminal_node_or_attach_children)
            .collect();
        let get_or_create_epsilon_node_method = self.gen_get_or_create_epsilon_node_method();
        quote! {
            impl<'i> #name_ident<'i> {
                #new_method
                #(#create_methods)*
                #(#ll1_parse_methods)*
                #(#get_gss_node_methods)*
                #(#add_gss_node_methods)*
                #(#specialized_lookup_nonterminal_node_methods)*
                #(#specialized_add_nonterminal_node_methods)*
                #(#create_nonterminal_node_or_attach_children_methods)*
                #get_or_create_epsilon_node_method
            }
        }
    }

    fn gen_get_or_create_epsilon_node_method(&self) -> TokenStream {
        let epsilon_id = Literal::usize_unsuffixed(self.terminal_ids.len());
        quote! {
            fn get_or_create_epsilon_node(&mut self, i: u32) -> SPPFNodeId {
                let existing = self.epsilon_nodes[i as usize];
                if existing != SPPFNodeId::NONE {
                    record!(self, TerminalNodeFound, existing);
                    return existing;
                }
                let span = Span::new(i, i);
                let terminal_id = TerminalId(#epsilon_id);
                let node_id = SPPFNodeId(self.sppf_nodes.len() as u32);
                record!(self, TerminalNodeCreated, terminal_id, span);
                self.sppf_nodes.push(SPPFNode::Terminal(TerminalNode { terminal_id, span }));
                self.epsilon_nodes[i as usize] = node_id;
                node_id
            }
        }
    }

    fn gen_new_method(&self) -> TokenStream {
        let grammar_name = &self.grammar.name;
        let name_ident = format_ident!("{}{}", grammar_name, "Scanner");
        let gss_nodes_index_field = self.gen_gss_nodes_index_field();
        let gss_nodes_index_fields: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(Self::gen_gss_nodes_index_field_init)
            .collect();
        let return_value_fields: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(Self::gen_specialized_nonterminal_nodes_index_field_init)
            .collect();
        let nonterminal_nodes_index_field = self.gen_nonterminal_nodes_index_field();
        let intermediate_nodes_index_field = self.gen_intermediate_nodes_index_field();
        let terminal_nodes_index_field = self.gen_terminal_nodes_index_field();
        let layout_memo_init = if self
            .grammar
            .nonterminals()
            .any(|nt| self.is_layout(nt) && self.ff.is_ll1(nt))
        {
            quote! { layout_memo: vec![None; input.len() as usize + 1], }
        } else {
            quote! {}
        };
        quote! {
            pub fn new(input: &'i Input, start_nonterminal: NonterminalId) -> Self {
                init_logger();
                Self {
                    start_nonterminal,
                    scanner: #name_ident::new(input),
                    #gss_nodes_index_field,
                    #(#gss_nodes_index_fields,)*
                    descriptors: Vec::with_capacity(1024),
                    gss_nodes: Vec::with_capacity(input.len() as usize * GSS_CAPACITY_MULTIPLIER),
                    sppf_nodes: Vec::with_capacity(input.len() as usize * SPPF_CAPACITY_MULTIPLIER),
                    #nonterminal_nodes_index_field,
                    #intermediate_nodes_index_field,
                    #terminal_nodes_index_field,
                    epsilon_nodes: vec![SPPFNodeId::NONE; input.len() as usize + 1],
                    #[cfg(feature = "instrument")]
                    descriptors_count: 0,
                    intermediate_nodes_children: vec![],
                    intermediate_nodes_children_map: OnceCell::new(),
                    nonterminal_nodes_children: vec![],
                    nonterminal_nodes_children_map: OnceCell::new(),
                    #(#return_value_fields,)*
                    envs: vec![],
                    parse_errors: InlineVec::Empty,
                    #layout_memo_init
                    #[cfg(feature = "debug-trace")]
                    trace_events: None,
                }
            }
        }
    }

    fn gen_parser_struct(&self) -> TokenStream {
        let grammar_name = &self.grammar.name;
        let nonterminal_ids_len = Literal::usize_unsuffixed(self.nonterminal_ids.len());
        let terminal_ids_len = Literal::usize_unsuffixed(self.terminal_ids.len() + 2);
        let gss_nodes_index_fields: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(Self::gen_gss_nodes_index_field_for_data_dependent_nt)
            .collect();
        let specialized_nonterminal_nodes_index_fields: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(Self::gen_specialized_nonterminal_nodes_index_field)
            .collect();
        let slot_ids_len = Literal::usize_unsuffixed(self.slot_ids.len());
        let parser_name_ident = format_ident!("{}{}", grammar_name, "Parser");
        let scanner_name_ident = format_ident!("{}{}", grammar_name, "Scanner");
        let layout_memo_field = if self
            .grammar
            .nonterminals()
            .any(|nt| self.is_layout(nt) && self.ff.is_ll1(nt))
        {
            quote! { layout_memo: Vec<Option<SPPFNodeId>>, }
        } else {
            quote! {}
        };
        quote! {
            pub struct #parser_name_ident<'i> {
                start_nonterminal: NonterminalId,
                scanner: #scanner_name_ident<'i>,
                descriptors: Vec<Descriptor>,
                gss_nodes: Vec<GSSNode>,
                #[comment = "A vector from nonterminal_ids to a tuple (input_index, gss_node_id)"]
                gss_nodes_index: [Vec<(u32, GssNodeId)>; #nonterminal_ids_len],
                #(#gss_nodes_index_fields,)*
                sppf_nodes: Vec<SPPFNode>,
                #[cfg(feature = "instrument")]
                descriptors_count: usize,
                nonterminal_nodes_index: [InlineMap<Span, SPPFNodeId>; #nonterminal_ids_len],
                intermediate_nodes_index: [InlineMap<Span, SPPFNodeId>; #slot_ids_len],
                terminal_nodes_index: [InlineMap<Span, SPPFNodeId>; #terminal_ids_len],
                #[comment = "Epsilon nodes keyed by input position; SPPFNodeId::NONE marks an empty slot."]
                epsilon_nodes: Vec<SPPFNodeId>,
                intermediate_nodes_children: Vec<(SPPFNodeId, (SPPFNodeId, SPPFNodeId))>,
                intermediate_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<(SPPFNodeId, SPPFNodeId)>>>,
                nonterminal_nodes_children: Vec<(SPPFNodeId, SPPFNodeId)>,
                nonterminal_nodes_children_map: OnceCell<FxHashMap<SPPFNodeId, Vec<SPPFNodeId>>>,
                #(#specialized_nonterminal_nodes_index_fields,)*
                envs: Vec<Env>,
                parse_errors: InlineVec<ParseError, 8>,
                #layout_memo_field
                #[cfg(feature = "debug-trace")]
                pub trace_events: Option<Vec<TraceEvent>>,
            }
        }
    }

    fn gen_gss_nodes_index_field(&self) -> TokenStream {
        let nonterminal_ids_len = Literal::usize_unsuffixed(self.nonterminal_ids.len());
        quote! {
            gss_nodes_index: [const { vec![] }; #nonterminal_ids_len]
        }
    }

    fn gen_nonterminal_nodes_index_field(&self) -> TokenStream {
        let nonterminal_ids_len = Literal::usize_unsuffixed(self.nonterminal_ids.len());
        quote! {
            nonterminal_nodes_index: [const { InlineMap::Empty }; #nonterminal_ids_len]
        }
    }

    fn gen_intermediate_nodes_index_field(&self) -> TokenStream {
        let intermediate_ids_len = Literal::usize_unsuffixed(self.slot_ids.len());
        quote! {
            intermediate_nodes_index: [const { InlineMap::Empty }; #intermediate_ids_len]
        }
    }

    fn gen_terminal_nodes_index_field(&self) -> TokenStream {
        let terminal_ids_len = Literal::usize_unsuffixed(self.terminal_ids.len() + 2);
        quote! {
            terminal_nodes_index: [const { InlineMap::Empty }; #terminal_ids_len]
        }
    }

    fn gen_match_any(&self, static_name: &str, input_index: TokenStream) -> TokenStream {
        let name = format_ident!("{}", static_name);
        quote! { self.scanner.match_any(#name, #input_index) }
    }

    fn gen_parse_method_ll1(&self, nonterminal: &'a Nonterminal, memoize: bool) -> TokenStream {
        // For Plus, generate a loop rather than matching the left-recursive
        // desugared alternatives, which would cause infinite recursion.
        // Star is desugared as `(A+)?`, so Opt and Star are handled by the
        // standard alternative-matching below.
        if matches!(&nonterminal.origin, Some(Symbol::Plus(_, _))) {
            return self.gen_plus_loop_ll1(nonterminal);
        }
        let method_name = format_ident!("parse_{}_ll1", to_snake_case(&nonterminal.name));
        let body_tokens = self.gen_parse_body_ll1(nonterminal);
        if memoize {
            quote! {
                fn #method_name(&mut self, i: u32) -> Option<SPPFNodeId> {
                    if let Some(memo) = self.layout_memo[i as usize] {
                        return Some(memo);
                    }
                    let result: Option<SPPFNodeId> = (|| -> Option<SPPFNodeId> {
                        #body_tokens
                    })();
                    if let Some(node) = result {
                        self.layout_memo[i as usize] = Some(node);
                    }
                    result
                }
            }
        } else {
            quote! {
                fn #method_name(&mut self, i: u32) -> Option<SPPFNodeId> {
                    #body_tokens
                }
            }
        }
    }

    fn gen_parse_body_ll1(&self, nonterminal: &'a Nonterminal) -> TokenStream {
        let nonterminal_id = self.nonterminal_ids.get_id(nonterminal);
        let alternatives = self.grammar.alternatives(nonterminal);

        let nullable_alt = alternatives.iter().find(|alt| self.ff.is_alt_nullable(alt));
        let non_nullable_alts: Vec<&Alternative> = alternatives
            .iter()
            .filter(|alt| !self.ff.is_alt_nullable(alt))
            .collect();

        // LL(1) disjointness gives at most one nullable alt: when one
        // exists, missing on FIRST means the input is a FOLLOW token, so
        // the nullable body is the correct fall-through. `longest_match`
        // disambiguates prefix-overlapping terminals (e.g. `<` vs `<=`).
        let first_set_name = format_ident!(
            "FIRST_SET_{}",
            to_snake_case(&nonterminal.name).to_uppercase(),
        );
        match (nullable_alt, non_nullable_alts.is_empty()) {
            (Some(nullable), true) => self.gen_alt_body_ll1(nonterminal, nullable, nonterminal_id),
            (None, _) => {
                let arms = self.gen_ll1_dispatch_arms(
                    nonterminal,
                    &alternatives.iter().collect::<Vec<_>>(),
                    nonterminal_id,
                );
                quote! {
                    let matched = self.scanner.longest_match(#first_set_name, i)?;
                    match matched {
                        #(#arms)*
                        _ => unreachable!("LL(1) dispatch covers every terminal in FIRST_SET"),
                    }
                }
            }
            (Some(nullable), false) => {
                let arms = self.gen_ll1_dispatch_arms(
                    nonterminal,
                    &alternatives.iter().collect::<Vec<_>>(),
                    nonterminal_id,
                );
                let nullable_body = self.gen_alt_body_ll1(nonterminal, nullable, nonterminal_id);
                quote! {
                    let Some(matched) = self.scanner.longest_match(#first_set_name, i) else {
                        #nullable_body
                    };
                    match matched {
                        #(#arms)*
                        _ => unreachable!("LL(1) dispatch covers every terminal in FIRST_SET"),
                    }
                }
            }
        }
    }

    fn gen_alt_body_ll1(
        &self,
        nonterminal: &'a Nonterminal,
        alternative: &'a Alternative,
        nonterminal_id: NonterminalId,
    ) -> TokenStream {
        let end_slot = Slot::new(nonterminal, alternative, alternative.symbols.len());
        let end_slot_id = self.slot_ids.get_id(&end_slot);
        if alternative.symbols.is_empty() {
            quote! {
                let epsilon_node_id = self.get_or_create_epsilon_node(i);
                return Some(self.get_or_create_nonterminal_node(
                    #nonterminal_id, #end_slot_id, i, i, epsilon_node_id, false,
                ));
            }
        } else {
            self.gen_parse_alternative_ll1(nonterminal, alternative, nonterminal_id, end_slot_id)
        }
    }

    fn gen_ll1_dispatch_arms(
        &self,
        nonterminal: &'a Nonterminal,
        alternatives: &[&'a Alternative],
        nonterminal_id: NonterminalId,
    ) -> Vec<TokenStream> {
        let eof_id = Literal::u16_unsuffixed(self.terminal_ids.len() as u16 + 1);
        alternatives
            .iter()
            .filter_map(|alternative| {
                let first = self.ff.first_set(alternative);
                // Empty FIRST = explicit-ε arm: reached via fall-through, not dispatch.
                if first.is_empty() {
                    return None;
                }
                let body = self.gen_alt_body_ll1(nonterminal, alternative, nonterminal_id);
                let pred_patterns: Vec<_> = first
                    .iter()
                    .map(|t| {
                        if t.name == "EOF" {
                            quote! { TerminalId(#eof_id) }
                        } else {
                            let id = self.terminal_ids.get_id(t);
                            quote! { #id }
                        }
                    })
                    .collect();
                Some(quote! {
                    #(#pred_patterns)|* => { #body }
                })
            })
            .collect()
    }

    /// Generates an LL(1) parse method for a Plus nonterminal as a loop.
    ///
    /// `A+` desugars to `APlus = APlus A | A`. This is left-recursive,
    /// so a recursive-descent parser would loop infinitely. The standard
    /// solution is to parse it as a loop: parse one `A`, then repeat
    /// while more `A`s follow.
    ///
    /// Layout and separators are handled naturally: they appear as symbols
    /// in the recursive alternative. For example, `{A ","}+` with layout
    /// desugars to `APlus = APlus Layout "," Layout A | A`. The loop
    /// parses Layout, `","`, Layout, A in sequence each iteration.
    ///
    /// The SPPF tree is built left-associative, matching GLL's output:
    ///
    /// Example: `S = A*` where `A = 'x'`, input `xxx`.
    /// After desugaring: `APlus = APlus A | A`
    ///
    /// ```text
    /// Iteration 0 (base):  APlus[0,1] -> "x"[0,1]
    /// Iteration 1 (loop):  APlus[0,2] -> Intermediate[0,2] -> (APlus[0,1], "x"[1,2])
    /// Iteration 2 (loop):  APlus[0,3] -> Intermediate[0,3] -> (APlus[0,2], "x"[2,3])
    /// ```
    fn gen_plus_loop_ll1(&self, nonterminal: &'a Nonterminal) -> TokenStream {
        let method_name = format_ident!("parse_{}_ll1", to_snake_case(&nonterminal.name));
        let nonterminal_id = self.nonterminal_ids.get_id(nonterminal);
        let alternatives = self.grammar.alternatives(nonterminal);
        assert_eq!(alternatives.len(), 2);

        let base_alt = &alternatives[1];
        let recursive_alt = &alternatives[0];

        let end_slots: Vec<_> = self.nonterminal_ids.end_slots(nonterminal_id).collect();
        let recursive_end_slot_id = end_slots[0].slot_id;
        let base_end_slot_id = end_slots[1].slot_id;

        // Base case: parse the single symbol in alt 1
        let base_symbol = base_alt.symbols.last().unwrap();
        let base_slot = Slot::new(nonterminal, base_alt, 0);
        let base_slot_id = self.slot_ids.get_id(&base_slot);
        let base_parse = self.gen_match_symbol_ll1(base_symbol, quote! { j }, base_slot_id);

        // Loop: try to parse symbols 1..n of the recursive alt (skip the
        // self-reference). Chain positions forward without updating j.
        // If all succeed, build intermediate nodes and advance j.
        let symbols: Vec<_> = recursive_alt.symbols.iter().skip(1).collect();

        // Phase 1: try each symbol, break if any fails.
        let mut parses = vec![];
        let mut current_pos = quote! { j };
        for (idx, symbol) in symbols.iter().enumerate() {
            let node_var = format_ident!("node_{}", idx);
            let pos_var = format_ident!("pos_{}", idx);
            let slot = Slot::new(nonterminal, recursive_alt, idx + 1);
            let slot_id = self.slot_ids.get_id(&slot);
            let parse = self.gen_match_symbol_ll1(symbol, current_pos.clone(), slot_id);
            parses.push(quote! {
                let Some((#node_var, #pos_var)) = #parse else { break; };
            });
            current_pos = quote! { #pos_var };
        }

        // Phase 2: all succeeded — update j, build intermediate nodes
        let last_pos = format_ident!("pos_{}", symbols.len() - 1);
        let mut build_nodes = vec![quote! { j = #last_pos; }];
        for (idx, _) in symbols.iter().enumerate() {
            let node_var = format_ident!("node_{}", idx);
            let pos_var = format_ident!("pos_{}", idx);
            let next_slot = Slot::new(nonterminal, recursive_alt, idx + 2);
            let next_slot_id = self.slot_ids.get_id(&next_slot);
            build_nodes.push(quote! {
                current = self.get_or_create_intermediate_node(
                    #next_slot_id, left_extent, #pos_var, current, #node_var, true,
                ).unwrap();
            });
        }

        quote! {
            fn #method_name(&mut self, i: u32) -> Option<SPPFNodeId> {
                let mut j = i;
                let (body_node, body_end) = (#base_parse)?;
                j = body_end;
                let left_extent = i;
                let mut current = self.get_or_create_nonterminal_node(
                    #nonterminal_id, #base_end_slot_id, left_extent, j, body_node, false,
                );
                loop {
                    #(#parses)*
                    #(#build_nodes)*
                    current = self.get_or_create_nonterminal_node(
                        #nonterminal_id, #recursive_end_slot_id, left_extent, j, current, false,
                    );
                }
                Some(current)
            }
        }
    }

    /// Generates parse code for a symbol at position `pos`. Returns a
    /// `TokenStream` that evaluates to `Option<(SPPFNodeId, u32)>` — the
    /// node and end position. Records a parse error on terminal match
    /// failure, consistent with GLL.
    fn gen_match_symbol_ll1(
        &self,
        symbol: &Symbol,
        pos: TokenStream,
        slot_id: SlotId,
    ) -> TokenStream {
        let identifier = symbol.as_identifier().unwrap();
        let def = self.grammar.definition(identifier.resolve());
        match def {
            Definition::Terminal(terminal) => {
                let terminal_id = self.terminal_ids.get_id(terminal);
                let terminal_name = &terminal.name;
                quote! {
                    self.match_terminal(#terminal_id, #pos, #slot_id, None, #terminal_name).map(|(end, node)| (node, end))
                }
            }
            Definition::Nonterminal(nt) => {
                let nt_method = format_ident!("parse_{}_ll1", to_snake_case(&nt.name));
                quote! {
                    self.#nt_method(#pos).map(|node| {
                        let end = self.sppf_node(node).right_extent();
                        (node, end)
                    })
                }
            }
        }
    }

    fn gen_parse_alternative_ll1(
        &self,
        nonterminal: &'a Nonterminal,
        alternative: &'a Alternative,
        nonterminal_id: NonterminalId,
        end_slot_id: SlotId,
    ) -> TokenStream {
        let mut body = vec![];
        body.push(quote! { let mut j = i; });

        for (pos, symbol) in alternative.symbols.iter().enumerate() {
            let slot = Slot::new(nonterminal, alternative, pos);
            let is_first = slot.is_first();
            let next_slot = slot.next();
            let next_slot_id = self.slot_ids.get_id(&next_slot);

            // Collect pre/post conditions from restriction symbols
            let mut pre_conditions: Vec<TokenStream> = vec![];
            let mut post_conditions: Vec<TokenStream> = vec![];
            match symbol {
                Symbol::Except { except, .. } => {
                    for e in except {
                        let Definition::Terminal(t) = self.grammar.definition(e.resolve()) else {
                            panic!("Except identifier must resolve to a terminal");
                        };
                        let id = self.terminal_ids.get_id(t);
                        post_conditions
                            .push(quote! { self.scanner.match_token(#id, start) != Some(end) });
                    }
                }
                Symbol::FollowRestriction { restrictions, .. } => {
                    for r in restrictions {
                        let Definition::Terminal(t) = self.grammar.definition(r.resolve()) else {
                            panic!("follow restriction must resolve to a terminal");
                        };
                        let id = self.terminal_ids.get_id(t);
                        post_conditions
                            .push(quote! { self.scanner.match_token(#id, end).is_none() });
                    }
                }
                Symbol::PrecedeRestriction { restriction, .. } => {
                    let Definition::Terminal(t) = self.grammar.definition(restriction.resolve())
                    else {
                        panic!("Precede restriction identifier must resolve to a terminal");
                    };
                    let id = self.terminal_ids.get_id(t);
                    pre_conditions
                        .push(quote! { j == 0 || self.scanner.match_token(#id, j - 1).is_none() });
                }
                _ => {}
            }

            let Some(identifier) = symbol.as_identifier() else {
                continue;
            };
            let def_id = identifier.resolve();
            let def = self.grammar.definition(def_id);

            let pre_check = if pre_conditions.is_empty() {
                quote! {}
            } else {
                quote! { if !(#(#pre_conditions)&&*) { return None; } }
            };

            let post_check = if post_conditions.is_empty() {
                quote! {}
            } else {
                quote! {
                    if let Some(error_kind) = self.post_conditions(#next_slot_id, start, end) {
                        self.add_parse_error(end, #next_slot_id, None, || error_kind);
                        return None;
                    }
                }
            };

            match def {
                Definition::Terminal(terminal) => {
                    let terminal_id = self.terminal_ids.get_id(terminal);
                    let terminal_name = &terminal.name;
                    body.push(quote! {
                        #pre_check
                        let right_child = {
                            let start = j;
                            let (end, node) = self.match_terminal(#terminal_id, start, #next_slot_id, None, #terminal_name)?;
                            #post_check
                            j = end;
                            node
                        };
                    });
                }
                Definition::Nonterminal(nt) => {
                    let nt_method = format_ident!("parse_{}_ll1", to_snake_case(&nt.name));
                    body.push(quote! {
                        #pre_check
                        let right_child = {
                            let start = j;
                            let node = self.#nt_method(start)?;
                            let end = self.sppf_node(node).right_extent();
                            j = end;
                            #post_check
                            node
                        };
                    });
                }
            }

            if is_first {
                body.push(quote! {
                    let left_extent = self.sppf_node(right_child).left_extent();
                    let mut current = right_child;
                });
            } else {
                body.push(quote! {
                    current = self.get_or_create_intermediate_node(
                        #next_slot_id, left_extent, j, current, right_child, true,
                    ).unwrap();
                });
            }
        }

        body.push(quote! {
            return Some(self.get_or_create_nonterminal_node(
                #nonterminal_id, #end_slot_id, left_extent, j, current, false,
            ));
        });

        quote! { #(#body)* }
    }

    fn gen_nonterminal_display_name_method() -> TokenStream {
        quote! {
            fn nonterminal_display_name(nonterminal_id: NonterminalId) -> &'static str {
                NONTERMINALS[nonterminal_id.index()].display
            }
        }
    }

    fn gen_terminal_name_method() -> TokenStream {
        quote! {
            fn terminal_name(terminal_id: TerminalId) -> &'static str {
                TERMINALS[terminal_id.index()].name
            }
        }
    }

    fn gen_slot_name_method() -> TokenStream {
        quote! {
            fn slot_name(slot_id: SlotId) -> &'static str {
                SLOTS[slot_id.index()].display_name
            }
        }
    }

    fn gen_epsilon_method() -> TokenStream {
        quote! {
            fn epsilon() -> TerminalId {
                TerminalId((TERMINALS.len() - 2) as u16)
            }
        }
    }

    fn gen_eof_method() -> TokenStream {
        quote! {
            fn eof() -> TerminalId {
                TerminalId((TERMINALS.len() - 1) as u16)
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

    fn gen_get_gss_node_method_with_parameters(nt: &Nonterminal) -> TokenStream {
        let method_name = format_ident!("get_gss_node_{}", to_snake_case(&nt.name));
        let parameters: Vec<_> = nt
            .parameters
            .iter()
            .map(|Parameter { name, ty }| {
                let name = format_ident!("{}", name);
                quote! { #name: #ty }
            })
            .collect();
        let field_name = format_ident!("gss_nodes_index_{}", to_snake_case(&nt.name));
        let args: Vec<_> = (0..parameters.len())
            .map(|i| format_ident!("a{i}"))
            .collect();
        let comparisons: Vec<_> = nt
            .parameters
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let lhs = format_ident!("a{}", i);
                let rhs = format_ident!("{}", p.name);
                quote! { *#lhs == #rhs }
            })
            .collect();
        let index = Literal::usize_unsuffixed(1 + nt.parameters.len());
        quote! {
            fn #method_name(&self, input_index: u32, #(#parameters),*) -> Option<GssNodeId> {
                self.#field_name
                    .iter()
                    .find(|(i, #(#args,)* _)| *i == input_index && #(#comparisons)&&*)
                    .map(|x| x.#index)
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

    fn gen_add_gss_node_method_with_parameters(nt: &Nonterminal) -> TokenStream {
        let method_name = format_ident!("add_gss_node_{}", to_snake_case(&nt.name));
        let parameters: Vec<_> = nt
            .parameters
            .iter()
            .map(|Parameter { name, ty }| {
                let name = format_ident!("{}", name);
                quote! { #name: #ty }
            })
            .collect();
        let field_name = format_ident!("gss_nodes_index_{}", to_snake_case(&nt.name));
        let parameter_names: Vec<_> = nt
            .parameters
            .iter()
            .map(|p| format_ident!("{}", p.name))
            .collect();
        quote! {
            fn #method_name(&mut self, input_index: u32, #(#parameters,)* gss_node_id: GssNodeId) {
                self.#field_name.push((input_index, #(#parameter_names,)* gss_node_id));
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
                #[cfg(feature = "instrument")]
                self.increment_descriptor_count();
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
                self.nonterminal_nodes_index[nonterminal_node.nonterminal_id.index()]
                    .insert(nonterminal_node.span, nonterminal_node_id);
                record!(
                    self,
                    NonterminalNodeCreated,
                    nonterminal_node.nonterminal_id,
                    nonterminal_node.span,
                    nonterminal_node.child
                );
                self.sppf_nodes.push(SPPFNode::Nonterminal(nonterminal_node));
                nonterminal_node_id
            }
        }
    }

    fn gen_add_intermediate_node_method() -> TokenStream {
        quote! {
            fn add_intermediate_node(
                &mut self,
                intermediate_node: IntermediateNode,
                add_to_index: bool,
            ) -> SPPFNodeId {
                let intermediate_node_id = SPPFNodeId(self.sppf_nodes.len() as u32);
                if add_to_index {
                    self.intermediate_nodes_index[intermediate_node.slot_id.index()]
                        .insert(intermediate_node.span, intermediate_node_id);
                }
                record!(
                    self,
                    IntermediateNodeCreated,
                    intermediate_node.slot_id,
                    intermediate_node.span,
                    intermediate_node.child.0,
                    intermediate_node.child.1
                );
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

    fn gen_sppf_nodes_method() -> TokenStream {
        quote! {
            fn sppf_nodes(&self) -> &[SPPFNode] {
                &self.sppf_nodes
            }
        }
    }

    fn gen_increment_descriptor_count_method() -> TokenStream {
        quote! {
            #[cfg(feature = "instrument")]
            fn increment_descriptor_count(&mut self) {
                self.descriptors_count += 1;
            }
        }
    }

    fn gen_count_methods() -> TokenStream {
        quote! {
            #[cfg(feature = "instrument")]
            fn count_descriptors(&self) -> usize {
                self.descriptors_count
            }

            #[cfg(feature = "instrument")]
            fn count_gss_nodes(&self) -> usize {
                self.gss_nodes.len()
            }

            #[cfg(feature = "instrument")]
            fn count_gss_edges(&self) -> usize {
                self.gss_nodes.iter().map(|n| n.edges().len()).sum()
            }

            #[cfg(feature = "instrument")]
            fn count_nonterminal_nodes(&self) -> usize {
                self.sppf_nodes.iter().filter(|n| matches!(n, SPPFNode::Nonterminal(_))).count()
            }

            #[cfg(feature = "instrument")]
            fn count_intermediate_nodes(&self) -> usize {
                self.sppf_nodes.iter().filter(|n| matches!(n, SPPFNode::Intermediate(_))).count()
            }

            #[cfg(feature = "instrument")]
            fn count_terminal_nodes(&self) -> usize {
                self.sppf_nodes.iter().filter(|n| matches!(n, SPPFNode::Terminal(_))).count()
            }

            #[cfg(feature = "instrument")]
            fn count_ambiguous_nodes(&self) -> usize {
                self.sppf_nodes.iter().filter(|n| match n {
                    SPPFNode::Nonterminal(nn) => nn.ambiguous,
                    SPPFNode::Intermediate(in_) => in_.ambiguous,
                    SPPFNode::Terminal(_) => false,
                }).count()
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

    fn gen_create_method(nt: &Nonterminal, id: usize) -> TokenStream {
        let create_method_name = format_ident!("create_{}", to_snake_case(&nt.name));
        let id = Literal::usize_unsuffixed(id);
        let get_gss_node_method_name = format_ident!("get_gss_node_{}", to_snake_case(&nt.name));
        let add_gss_node_method_name = format_ident!("add_gss_node_{}", to_snake_case(&nt.name));
        let parameters: Vec<_> = nt
            .parameters
            .iter()
            .map(|Parameter { name, ty }| {
                let name = format_ident!("{}", name);
                quote! { #name: #ty }
            })
            .collect();
        let bindings: Vec<_> = nt
            .parameters
            .iter()
            .map(|p| {
                let key = &p.name;
                let value = format_ident!("{}", p.name);
                quote! {
                    env.bind(#key, #value);
                }
            })
            .collect();
        let param_names: Vec<_> = nt
            .parameters
            .iter()
            .map(|p| format_ident!("{}", p.name))
            .collect();
        quote! {
            fn #create_method_name(
                &mut self,
                sppf_node_id: Option<SPPFNodeId>,
                gss_node_id: GssNodeId,
                return_slot: SlotId,
                env: Option<EnvId>,
                binding: Option<&'static str>,
                #(#parameters,)*
            ) {
                record!(self, Call, sppf_node_id, gss_node_id, return_slot);
                let left_child = sppf_node_id.map(|id| {
                    let node = self.sppf_node(id);
                    (id, node.left_extent())
                });
                let gss_node = self.gss_node(gss_node_id);
                let i = match left_child {
                    Some((id, _)) => self.sppf_node(id).right_extent(),
                    None => gss_node.index,
                };
                #[comment = "If there is already a GSS node for this call, add an edge."]
                if let Some(existing_gss_node_id) = self.#get_gss_node_method_name(i, #(#param_names),*) {
                    record!(self, GSSNodeFound, NonterminalId(#id), i);
                    self.add_edge_to_existing_gss_node(existing_gss_node_id, gss_node_id, left_child, return_slot, env, binding);
                } else {
                    record!(self, GSSNodeNotFound, NonterminalId(#id), i);
                    let new_gss_node_id = self.new_gss_node(NonterminalId(#id), i);
                    self.add_gss_edge(new_gss_node_id, gss_node_id, sppf_node_id, return_slot, env, binding);
                    // Create a new environment to bind the parameter.
                    let (env_id, env) = self.new_env();
                    #(#bindings)*
                    self.add_first_descriptors(NonterminalId(#id), i, new_gss_node_id, Some(env_id));
                    self.#add_gss_node_method_name(i, #(#param_names,)* new_gss_node_id);
                }
            }
        }
    }

    fn gen_specialized_lookup_nonterminal_node_method(nt: &Nonterminal) -> TokenStream {
        let create_method_name =
            format_ident!("lookup_nonterminal_node_{}", to_snake_case(&nt.name));
        let field_name = format_ident!("nonterminal_nodes_index_{}", to_snake_case(&nt.name));
        quote! {
            fn #create_method_name(
                &self,
                left_extent: u32,
                right_extent: u32,
                return_value: i32
            ) -> Option<SPPFNodeId> {
                let span = Span::new(left_extent, right_extent);
                self.#field_name
                    .get(&span)
                    .and_then(|entries| {
                        entries.iter()
                            .find(|(rv, _)| *rv == return_value)
                            .map(|(_, id)| *id)
                    })
            }
        }
    }

    fn gen_specialized_add_nonterminal_node_method(nt: &Nonterminal) -> TokenStream {
        let method_name = format_ident!("add_nonterminal_node_{}", to_snake_case(&nt.name));
        let field_name = format_ident!("nonterminal_nodes_index_{}", to_snake_case(&nt.name));
        quote! {
            fn #method_name(&mut self, nonterminal_node: NonterminalNode, return_value: i32) -> SPPFNodeId {
                let nonterminal_node_id = SPPFNodeId(self.sppf_nodes.len() as u32);
                self.#field_name
                    .entry(nonterminal_node.span)
                    .or_default()
                .push((return_value, nonterminal_node_id));
                record!(
                    self,
                    NonterminalNodeCreated,
                    nonterminal_node.nonterminal_id,
                    nonterminal_node.span,
                    nonterminal_node.child
                );
                self.sppf_nodes.push(SPPFNode::Nonterminal(nonterminal_node));
                nonterminal_node_id
            }
        }
    }

    fn gen_create_nonterminal_node_or_attach_children(nt: &Nonterminal) -> TokenStream {
        let method_name = format_ident!(
            "create_nonterminal_node_or_attach_children_{}",
            to_snake_case(&nt.name)
        );
        let lookup_method_name =
            format_ident!("lookup_nonterminal_node_{}", to_snake_case(&nt.name));
        let add_method_name = format_ident!("add_nonterminal_node_{}", to_snake_case(&nt.name));
        quote! {
            fn #method_name(
                &mut self,
                nonterminal_id: NonterminalId,
                return_slot: SlotId,
                left_extent: u32,
                right_extent: u32,
                child: SPPFNodeId,
                return_value: i32,
            ) -> SPPFNodeId {
                if let Some(existing_node_id) =
                    self.#lookup_method_name(left_extent, right_extent, return_value)
                {
                    record!(self, NonterminalNodeFound, existing_node_id);
                    let node = self.sppf_node_mut(existing_node_id);
                    let SPPFNode::Nonterminal(node) = node else {
                        unreachable!("Expects a nonterminal node");
                    };
                    node.ambiguous = true;
                    self.add_nonterminal_node_child(existing_node_id, child);
                    return existing_node_id;
                }
                let nonterminal_node = NonterminalNode {
                    nonterminal_id,
                    return_slot,
                    span: Span {
                        left_extent,
                        right_extent,
                    },
                    child,
                    ambiguous: false,
                };
                self.#add_method_name(nonterminal_node, return_value)
            }
        }
    }

    fn gen_gss_nodes_index_field_for_data_dependent_nt(nt: &Nonterminal) -> TokenStream {
        let field_name = format_ident!("gss_nodes_index_{}", to_snake_case(&nt.name));
        let types: Vec<_> = nt.parameters.iter().map(|p| &p.ty).collect();
        let comment = format!("GSS index for nonterminal {}", nt.name);
        quote! {
            #[comment = #comment]
            #field_name: Vec<(u32, #(#types,)* GssNodeId)>
        }
    }

    fn gen_specialized_nonterminal_nodes_index_field(nt: &Nonterminal) -> TokenStream {
        let field_name = format_ident!("nonterminal_nodes_index_{}", to_snake_case(&nt.name));
        quote! {
            #field_name: FxHashMap<Span, InlineVec<(i32, SPPFNodeId)>>
        }
    }

    fn gen_gss_nodes_index_field_init(nt: &Nonterminal) -> TokenStream {
        let field_name = format_ident!("gss_nodes_index_{}", to_snake_case(&nt.name));
        quote! {
            #field_name: vec![]
        }
    }

    fn gen_specialized_nonterminal_nodes_index_field_init(nt: &Nonterminal) -> TokenStream {
        let field_name = format_ident!("nonterminal_nodes_index_{}", to_snake_case(&nt.name));
        quote! {
            #field_name: FxHashMap::default()
        }
    }

    fn gen_start_nonterminal_method() -> TokenStream {
        quote! {
            fn start_nonterminal(&self) -> NonterminalId {
                self.start_nonterminal
            }
        }
    }

    // Builds the initial environment when the start nonterminal is data-dependent.
    // Today the only data-dependent use case is operator precedence, so every parameter
    // is bound to 0 ("any precedence"). TODO: once parameters can have non-i32 types,
    // pick a default per type (or let the user specify a value).
    fn gen_start_env_method(&self) -> TokenStream {
        let arms: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(|nt| {
                let id = self.nonterminal_ids.get_id(nt);
                let bindings: Vec<_> = nt
                    .parameters
                    .iter()
                    .map(|p| {
                        let key = &p.name;
                        quote! { env.bind(#key, 0); }
                    })
                    .collect();
                quote! {
                    #id => {
                        let (env_id, env) = self.new_env();
                        #(#bindings)*
                        Some(env_id)
                    }
                }
            })
            .collect();
        let body = if arms.is_empty() {
            quote! { None }
        } else {
            quote! {
                match self.start_nonterminal {
                    #(#arms)*
                    _ => None,
                }
            }
        };
        quote! {
            fn start_env(&mut self) -> Option<EnvId> {
                #body
            }
        }
    }

    // Resolves the SPPF root for a data-dependent start nonterminal by reading from
    // its per-nonterminal index. Multiple entries at the same span (different return
    // values) all represent valid parses; we pick the first.
    fn gen_lookup_start_nonterminal_node_method(&self) -> TokenStream {
        let arms: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(|nt| {
                let id = self.nonterminal_ids.get_id(nt);
                let field_name =
                    format_ident!("nonterminal_nodes_index_{}", to_snake_case(&nt.name));
                quote! {
                    #id => {
                        let span = Span::new(0, right_extent);
                        self.#field_name
                            .get(&span)
                            .and_then(|entries| entries.first().map(|(_, id)| *id))
                    }
                }
            })
            .collect();
        let body = if arms.is_empty() {
            quote! { self.lookup_nonterminal_node(self.start_nonterminal, 0, right_extent) }
        } else {
            quote! {
                match self.start_nonterminal {
                    #(#arms)*
                    _ => self.lookup_nonterminal_node(self.start_nonterminal, 0, right_extent),
                }
            }
        };
        quote! {
            fn lookup_start_nonterminal_node(&self, right_extent: u32) -> Option<SPPFNodeId> {
                #body
            }
        }
    }

    // Routes the start GSS node into the per-nonterminal index for data-dependent
    // nonterminals. Recursive callers read from `gss_nodes_index_<name>` keyed by
    // parameter values, so the start node has to live in the same specialized index
    // or a duplicate gets created on the first recursive call. Parameters are bound
    // to 0, matching `gen_start_env_method`.
    fn gen_add_start_gss_node_method(&self) -> TokenStream {
        let arms: Vec<_> = self
            .nonterminal_ids
            .dd_nonterminals()
            .map(|nt| {
                let id = self.nonterminal_ids.get_id(nt);
                let method_name = format_ident!("add_gss_node_{}", to_snake_case(&nt.name));
                let zeros: Vec<_> = nt.parameters.iter().map(|_| quote! { 0 }).collect();
                quote! {
                    #id => self.#method_name(input_index, #(#zeros,)* gss_node_id),
                }
            })
            .collect();
        let body = if arms.is_empty() {
            quote! { self.add_gss_node(nonterminal_id, input_index, gss_node_id); }
        } else {
            quote! {
                match nonterminal_id {
                    #(#arms)*
                    _ => self.add_gss_node(nonterminal_id, input_index, gss_node_id),
                }
            }
        };
        quote! {
            fn add_start_gss_node(
                &mut self,
                nonterminal_id: NonterminalId,
                input_index: u32,
                gss_node_id: GssNodeId,
            ) {
                #body
            }
        }
    }

    fn gen_new_env_method() -> TokenStream {
        quote! {
            fn new_env(&mut self) -> (EnvId, &mut Env) {
                let id = EnvId(self.envs.len() as u32);
                self.envs.push(Env::default());
                (id, &mut self.envs[id.index()])
            }
        }
    }

    fn gen_lookup_method() -> TokenStream {
        quote! {
            fn lookup(&self, name: &str, env_id: EnvId) -> i32 {
                let env = &self.envs[env_id.index()];
                env.get(name)
            }
        }
    }

    fn gen_clone_env() -> TokenStream {
        quote! {
            fn clone_env(&mut self, source: EnvId) -> (EnvId, &mut Env) {
                let bindings = self.envs[source.0 as usize].bindings.clone();
                let (new_id, new_env) = self.new_env();
                new_env.bindings = bindings;
                (new_id, new_env)
            }
        }
    }

    fn gen_envs_method() -> TokenStream {
        quote! {
            fn envs(&self) -> &[Env] {
                &self.envs
            }
        }
    }

    fn gen_record_stats_method() -> TokenStream {
        quote! {
            #[cfg(feature = "instrument")]
            fn record_stats(&self) -> iguana_runtime::instrument::Stats {
                let mut stats = iguana_runtime::instrument::Stats::new();

                // Counters
                stats.descriptors_count = self.count_descriptors();
                stats.gss_nodes_count = self.count_gss_nodes();
                stats.gss_edges_count = self.count_gss_edges();
                stats.nonterminal_nodes_count = self.count_nonterminal_nodes();
                stats.intermediate_nodes_count = self.count_intermediate_nodes();
                stats.terminal_nodes_count = self.count_terminal_nodes();
                stats.ambiguous_nodes_count = self.count_ambiguous_nodes();

                // Histograms
                for node in self.gss_nodes() {
                    stats.record("GssNode::edges: InlineVec", node.edges().len());
                    stats.record("GssNode::popped_elements: InlineSet", node.popped_elements().len());
                }
                for env in self.envs() {
                    stats.record("Env::bindings: InlineVec", env.bindings.len());
                }
                for m in self.nonterminal_nodes_index.iter() {
                    stats.record("Parser::nonterminal_nodes_index: InlineMap", m.len());
                }
                for m in self.intermediate_nodes_index.iter() {
                    stats.record("Parser::intermediate_nodes_index: InlineMap", m.len());
                }
                for m in self.terminal_nodes_index.iter() {
                    stats.record("Parser::terminal_nodes_index: InlineMap", m.len());
                }
                stats
            }
        }
    }

    fn gen_expr(expr: &Expr) -> TokenStream {
        match expr {
            Expr::Int(i) => {
                let val = Literal::i32_unsuffixed(*i as i32);
                quote! { #val }
            }
            Expr::Ref(name) => {
                quote! { self.lookup(#name, env.unwrap()) }
            }
            Expr::Cond(cond) => {
                let left = Self::gen_expr(&cond.left);
                let right = Self::gen_expr(&cond.right);
                match cond.op {
                    CondOp::Eq => quote! { #left == #right },
                    CondOp::Leq => quote! { #left <= #right },
                    CondOp::Geq => quote! { #left >= #right },
                }
            }
            Expr::Or(left, right) => {
                let left = Self::gen_expr(left);
                let right = Self::gen_expr(right);
                quote! { (#left) || (#right) }
            }
            Expr::Min(left, right) => {
                let left = Self::gen_expr(left);
                let right = Self::gen_expr(right);
                quote! { std::cmp::min(#left, #right) }
            }
            Expr::Ternary { cond, then, r#else } => {
                let cond = Self::gen_expr(cond);
                let then = Self::gen_expr(then);
                let r#else = Self::gen_expr(r#else);
                quote! {
                    if #cond {
                        #then
                    } else {
                        #r#else
                    }
                }
            }
        }
    }
}
