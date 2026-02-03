use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use rustc_hash::FxHashMap;

use crate::{
    generator::{id::{NonterminalIds, SlotIds, TerminalIds}, utils::{alternative_label, is_rust_keyword, is_valid_rust_ident, to_first_uppercase, to_pascal_case, to_snake_case}},
    grammar::{def::{Alternative, Grammar}, symbols::{Definition, Nonterminal, Symbol}}, ids::TerminalId,
};

pub fn generate(
    grammar: &Grammar,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds,
) -> TokenStream {
    let terminals: Vec<(TerminalId, String)> = terminal_ids
        .ids()
        .zip(terminal_ids.terminals())
        .map(|(id, t)| (id, t.to_string()))
        .collect();
    let imports = gen_imports(grammar);
    let token_kind_enum = gen_token_kind_enum(&terminals);
    let token_kind_impl = gen_token_kind_impl(&terminals);
    let token_kind_function = gen_token_kind_function(&terminals);
    let token_struct = gen_token_struct();
    let token_impl = gen_token_impl();
    let parse_tree_enum = gen_parse_tree_enum(grammar);
    let parse_tree_impl = gen_parse_tree_impl(grammar);
    let parse_tree_ref_enum = gen_parse_tree_ref_enum(grammar);
    let parse_tree_ref_impl = gen_parse_tree_ref_impl(grammar);
    let list_node_trait = gen_list_node_trait();
    let list_node_impls_for_plus: Vec<_> = grammar
        .nonterminals()
        .filter(|n| n.is_plus())
        .map(|n| gen_list_node_impl_for_plus(grammar, n))
        .collect();
    let list_node_impls_for_star: Vec<_> = grammar
        .nonterminals()
        .filter(|n| n.is_star())
        .map(|n| gen_list_node_impl_for_star(grammar, n))
        .collect();
    let opt_node_trait = gen_opt_node_trait();
    let opt_node_impls: Vec<_> = grammar
        .nonterminals()
        .filter(|n| matches!(&n.origin, Some(Symbol::Opt(_))))
        .map(|n| gen_opt_node_impl(grammar, n))
        .collect();
    let alt_accessor_impls: Vec<TokenStream> = grammar
        .nonterminals()
        .filter(|n| matches!(&n.origin, Some(Symbol::Alt(_))))
        .map(|n| gen_alt_accessors(grammar, n))
        .collect();
    let from_for_tree_impls = gen_from_for_tree_impls(grammar);
    let parse_tree_builder_impl = gen_parse_tree_builder_impl(grammar, nonterminal_ids, slot_ids);
    let create_parse_tree_function = gen_create_parse_tree_function(grammar);
    let create_parse_tree_functions: Vec<_> = grammar
        .nonterminals()
        .map(|n| gen_create_parse_tree_nonterminal_function(grammar, &n.name))
        .collect();

    let nonterminal_types: Vec<_> = grammar
        .nonterminals()
        .map(|n| gen_nonterminal_type(grammar, n))
        .collect();

    let nonterminal_types_impl: Vec<_> = grammar
        .nonterminals()
        .map(|n| gen_nonterminal_type_impl(grammar, n))
        .collect();

    let to_sexpr_function = gen_to_sexpr_function();
    let node_to_sexpr_function = gen_node_to_sexpr_function();
    let to_json_function = gen_to_json_function();

    quote! {
        #imports
        #token_kind_enum
        #token_kind_impl
        #parse_tree_enum
        #parse_tree_impl
        #parse_tree_ref_enum
        #parse_tree_ref_impl
        #from_for_tree_impls
        #list_node_trait
        #opt_node_trait
        #(#nonterminal_types)*
        #(#nonterminal_types_impl)*
        #(#list_node_impls_for_plus)*
        #(#list_node_impls_for_star)*
        #(#opt_node_impls)*
        #(#alt_accessor_impls)*
        #token_struct
        #token_impl
        #token_kind_function
        #parse_tree_builder_impl
        #create_parse_tree_function
        #(#create_parse_tree_functions)*
        #to_sexpr_function
        #node_to_sexpr_function
        #to_json_function
    }
}

fn gen_imports(grammar: &Grammar) -> TokenStream {
    let parser_name = format_ident!("{}Parser", to_first_uppercase(&grammar.name));
    quote! {
        use core::fmt;
        use std::{fmt::Write, vec::IntoIter};
        use iguana_runtime::{
            ids::{NonterminalId, SlotId, TerminalId},
            parse_tree::{OneOrMany, ParseTreeBuilder, visit_sppf},
            parser::Parser,
            sppf::{NonterminalNode, SPPFNodeId, Span, TerminalNode},
        };
        use crate::parser::#parser_name;
    }
}

fn gen_nonterminal_type(
    grammar: &Grammar,
    nonterminal: &Nonterminal,
) -> TokenStream {
    let alternatives = grammar.alternatives(nonterminal);
    if alternatives.len() == 1 {
        gen_nonterminal_type_with_one_alternative(grammar, nonterminal, &alternatives[0])
    } else {
        gen_nonterminal_type_with_more_than_one_alternative(grammar, nonterminal, alternatives)
    }
}

fn gen_nonterminal_type_with_one_alternative(
    grammar: &Grammar,
    nonterminal: &Nonterminal,
    alternative: &Alternative
) -> TokenStream {
    let counts = count_symbol_occurrences(grammar, &alternative.symbols);
    let fields: Vec<_> = alternative
        .symbols
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let base_name = get_symbol_base_name(grammar, s);
            let needs_index = base_name.map_or(false, |name| counts.get(&name).copied().unwrap_or(0) > 1);
            let field_name = gen_field_name(grammar, s, i, needs_index);
            let field_ident = Ident::new(&field_name, Span::call_site());
            let def = grammar.definition(s.resolved_def());
            let field_type = match def {
                Definition::Terminal(_) => {
                    let token = Ident::new("Token", Span::call_site());
                    quote! { #token }
                },
                Definition::Nonterminal(nt) => {
                    let name = Ident::new(&to_pascal_case(def.name()), Span::call_site());
                    if should_be_boxed(nt, nonterminal) {
                        quote! { Box<#name> }
                    } else {
                        quote! { #name }
                    }
                },
            };
            quote! { pub #field_ident: #field_type }
        })
        .collect();
    let nonterminal_name = &nonterminal.name;
    let comment = if nonterminal_name != &nonterminal.display_name() {
        let display_name = nonterminal.display_name();
        quote! { #[comment = #display_name] }
    } else {
        quote! {}
    };
    let nonterminal_name_id = Ident::new(&to_pascal_case(nonterminal_name), Span::call_site());
    quote! {
        #comment
        #[derive(Debug)]
        pub struct #nonterminal_name_id {
            #(#fields,)*
            pub span: Span,
        }
    }
}

/// Returns the base name for a symbol used for field name generation.
/// This is used to count occurrences of the same symbol type in an alternative.
fn get_symbol_base_name(grammar: &Grammar, symbol: &Symbol) -> Option<String> {
    if symbol.label().is_some() {
        return None;
    }

    match symbol {
        Symbol::Star(inner, _) | Symbol::Plus(inner, _) | Symbol::Opt(inner) => {
            if let Symbol::Identifier(ident) = inner.as_ref() {
                let snake = to_snake_case(&ident.name);
                if is_valid_rust_ident(&snake) {
                    return Some(snake);
                }
            }
            None
        }
        Symbol::Identifier(ident) => {
            if let Some(def_id) = ident.definition {
                if let Definition::Nonterminal(nt) = grammar.definition(def_id) {
                    if let Some(origin) = &nt.origin {
                        match origin {
                            Symbol::Star(inner, _) | Symbol::Plus(inner, _) | Symbol::Opt(inner) => {
                                if let Symbol::Identifier(inner_ident) = inner.as_ref() {
                                    let snake = to_snake_case(&inner_ident.name);
                                    if is_valid_rust_ident(&snake) {
                                        return Some(snake);
                                    }
                                }
                                return None;
                            }
                            _ => {}
                        }
                    }
                }
            }
            let snake = to_snake_case(&ident.name);
            if is_valid_rust_ident(&snake) {
                Some(snake)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Counts how many times each symbol base name appears in the alternative.
fn count_symbol_occurrences(grammar: &Grammar, symbols: &[Symbol]) -> FxHashMap<String, usize> {
    let mut counts = FxHashMap::default();
    for symbol in symbols {
        if let Some(base_name) = get_symbol_base_name(grammar, symbol) {
            *counts.entry(base_name).or_insert(0) += 1;
        }
    }
    counts
}

fn gen_field_name(grammar: &Grammar, symbol: &Symbol, position: usize, needs_index: bool) -> String {
    if let Some(label) = symbol.label() {
        return to_snake_case(label);
    }

    let field_name = match symbol {
        Symbol::Star(inner, _) | Symbol::Plus(inner, _) => {
            if let Symbol::Identifier(ident) = inner.as_ref() {
                let snake = to_snake_case(&ident.name);
                if is_valid_rust_ident(&snake) {
                    pluralize(&snake)
                } else {
                    format!("field_{}", position)
                }
            } else {
                format!("field_{}", position)
            }
        }
        Symbol::Opt(inner) => {
            if let Symbol::Identifier(ident) = inner.as_ref() {
                let snake = to_snake_case(&ident.name);
                if is_valid_rust_ident(&snake) {
                    snake
                } else {
                    format!("field_{}", position)
                }
            } else {
                format!("field_{}", position)
            }
        }
        Symbol::Identifier(ident) => {
            // Check if this identifier points to a derived nonterminal (Star/Plus/Opt)
            if let Some(def_id) = ident.definition {
                if let Definition::Nonterminal(nt) = grammar.definition(def_id) {
                    if let Some(origin) = &nt.origin {
                        match origin {
                            Symbol::Star(inner, _) | Symbol::Plus(inner, _) => {
                                if let Symbol::Identifier(inner_ident) = inner.as_ref() {
                                    let snake = to_snake_case(&inner_ident.name);
                                    if is_valid_rust_ident(&snake) {
                                        return pluralize(&snake);
                                    }
                                }
                            }
                            Symbol::Opt(inner) => {
                                if let Symbol::Identifier(inner_ident) = inner.as_ref() {
                                    let snake = to_snake_case(&inner_ident.name);
                                    if is_valid_rust_ident(&snake) {
                                        return snake;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            let snake_case = to_snake_case(&ident.name);
            if is_valid_rust_ident(&snake_case) {
                if needs_index {
                    format!("{}_{}", snake_case, position)
                } else {
                    snake_case
                }
            } else {
                format!("lit_{}", position)
            }
        }
        _ => format!("field_{}", position),
    };

    if is_rust_keyword(&field_name) {
        format!("r#{}", field_name)
    } else {
        field_name
    }
}

fn gen_nonterminal_type_with_more_than_one_alternative(
    grammar: &Grammar,
    nonterminal: &Nonterminal,
    alternatives: &[Alternative]
) -> TokenStream {
        let arms: Vec<_> = alternatives
            .iter()
            .enumerate()
            .map(|(index, alternative)| {
                let counts = count_symbol_occurrences(grammar, &alternative.symbols);
                let fields: Vec<_> = alternative
                    .symbols
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let base_name = get_symbol_base_name(grammar, s);
                        let needs_index = base_name.map_or(false, |name| counts.get(&name).copied().unwrap_or(0) > 1);
                        let field_name = gen_field_name(grammar, s, i, needs_index);
                        let field_ident = Ident::new(&field_name, Span::call_site());
                        let def_id = s.resolved_def();
                        let def = grammar.definition(def_id);
                        let type_token = match def {
                            Definition::Terminal(_) => {
                                let token = Ident::new("Token", Span::call_site());
                                quote! { #token }
                            },
                            Definition::Nonterminal(nt) => {
                                if should_be_boxed(nt, nonterminal) {
                                    let name = Ident::new(&to_pascal_case(def.name()), Span::call_site());
                                    quote! { Box<#name> }
                                } else {
                                    let name = Ident::new(&to_pascal_case(def.name()), Span::call_site());
                                    quote! { #name }
                                }
                            },
                        };
                        quote! { #field_ident: #type_token }
                    })
                    .collect();
                let label = alternative_label(alternative, index);
                let variant_name = Ident::new(&label, Span::call_site());
                // Add Span as last field in each variant
                quote! {
                    #variant_name { #(#fields,)* span: Span }
                }
            })
            .collect();
        let nonterminal_name = &nonterminal.name;
        let comment = if nonterminal_name != &nonterminal.display_name() {
            let display_name = nonterminal.display_name();
            quote! { #[comment = #display_name] }
        } else {
            quote! {}
        };
        let nonterminal_name_id = Ident::new(&to_pascal_case(nonterminal_name), Span::call_site());
        quote! {
            #comment
            #[derive(Debug)]
            pub enum #nonterminal_name_id {
                #(#arms),*
            }
        }
}

fn gen_nonterminal_type_impl(
    grammar: &Grammar,
    nonterminal: &Nonterminal,
) -> TokenStream {
    let nonterminal_name = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
    let child_method = gen_child_method(grammar, nonterminal);
    let child_count_method = gen_child_count_method(grammar, nonterminal);
    let as_node_ref_method = gen_as_parse_tree_ref_method(&nonterminal.name);
    let span_method = gen_span_method(grammar, nonterminal);
    let typed_accessor = gen_typed_accessor(nonterminal);
    quote! {
        impl #nonterminal_name {
            #child_method
            #child_count_method
            #as_node_ref_method
            #span_method
            #typed_accessor
        }
    }
}

fn gen_span_method(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    let ident = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
    let alternatives = grammar.alternatives(nonterminal);
    let body = if alternatives.len() == 1 {
        quote! { self.span }
    } else {
        let arms: Vec<_> = alternatives.iter().enumerate().map(|(i, alternative)| {
            let label = alternative_label(alternative, i);
            let alt_variant = Ident::new(&to_pascal_case(&label), Span::call_site());
            quote! {
                #ident::#alt_variant { span, .. } => *span
            }
        }).collect();
        quote! {
            match self {
                #(#arms),*
            }
        }
    };
    quote! {
        pub fn span(&self) -> Span {
            #body
        }
    }
}

fn gen_child_method(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    let children_by_index = gen_children_by_index(grammar, nonterminal);
    quote! {
        pub fn child(&self, index: usize) -> Option<ParseTreeRef<'_>> {
            #children_by_index
        }
    }
}

fn gen_children_by_index(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    let ident = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
    let alternatives = grammar.alternatives(nonterminal);
    if alternatives.len() == 1 {
        let alternative = &alternatives[0];
        let body = child_by_index(grammar, alternative, true);
        quote! {
            match index {
                #body
            }
        }
    } else {
        let arms: Vec<_> = alternatives.iter().enumerate().map(|(i, alternative)| {
            let label = alternative_label(alternative, i);
            let alt_variant = Ident::new(&to_pascal_case(&label), Span::call_site());
            let field_names = field_names(grammar, alternative);
            let body = child_by_index(grammar, alternative, false);
            // Use struct pattern with .. to ignore the span field
            quote! {
                #ident::#alt_variant { #(#field_names,)* .. } => #body
            }
        }).collect();
        quote! {
            match self {
                #(#arms),*
            }
        }
    }
}

// TODO: simplify the single_rule logic here:
fn child_by_index(grammar: &Grammar, alternative: &Alternative, single_rule: bool) -> TokenStream {
    let counts = count_symbol_occurrences(grammar, &alternative.symbols);
    let cases: Vec<_> = alternative.symbols.iter().enumerate().map(|(i, s)| {
        let i_lit = Literal::usize_unsuffixed(i);
        let base_name = get_symbol_base_name(grammar, s);
        let needs_index = base_name.map_or(false, |name| counts.get(&name).copied().unwrap_or(0) > 1);
        let field_name = Ident::new(&gen_field_name(grammar, s, i, needs_index), Span::call_site());
        // For nonterminals with only one body, i.e., no alternatives,
        // generate the arms as 0 => Some(self.field_name.as_parse_tree_ref())
        // As, we can access the children by field name directly.
        if single_rule {
            quote! {
                #i_lit => Some(self.#field_name.as_parse_tree_ref())
            }
        } else {
            // For nonterminals with alternatives, we need to return the exact child:
            // case E::Plus { symbol, layout1, lit2, .. } {
            //     match index {
            //         0 => Some(symbol.as_parse_tree_ref()),
            //         1 => Some(layout1.as_parse_tree_ref()),
            //         2 => Some(lit2.as_parse_tree_ref()),
            //         _ => None
            // }
            quote! {
                #i_lit => Some(#field_name.as_parse_tree_ref())
            }
        }
    }).collect();
    if single_rule {
        quote! {
            #(#cases,)*
            _ => None,
        }
    } else {
        quote! {
            match index {
                #(#cases,)*
                _ => None,
            }
        }
    }
}

fn gen_child_count_method(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    let ident = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
    let alternatives = grammar.alternatives(nonterminal);
    let body = if alternatives.len() == 1 {
        let count_symbols = alternatives[0].symbols.len();
        quote! {
            #count_symbols
        }
    } else {
        let arms: Vec<_> = alternatives.iter().enumerate().map(|(i, alternative)| {
            let label = alternative_label(alternative, i);
            let alt_variant = Ident::new(&to_pascal_case(&label), Span::call_site());
            let count_symbols = alternative.symbols.len();
            // Use { .. } to match any fields (including span)
            quote! {
                #ident::#alt_variant { .. } => #count_symbols
            }
        }).collect();
        quote! {
            match self {
                #(#arms),*
            }
        }
    };
    quote! {
        pub fn child_count(&self) -> usize {
            #body
        }
    }
}

fn gen_as_parse_tree_ref_method(nonterminal_name: &str) -> TokenStream {
    let name_ident = Ident::new(&to_pascal_case(nonterminal_name), Span::call_site());
    quote! {
        pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
            ParseTreeRef::#name_ident(self)
        }
    }
}

fn gen_token_struct() -> TokenStream {
    quote! {
        #[derive(Debug)]
        pub struct Token {
            kind: TokenKind,
            span: Span,
        }
    }
}

fn gen_token_impl() -> TokenStream {
    quote! {
        impl Token {
            pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
                ParseTreeRef::Token(self)
            }

            pub fn span(&self) -> Span {
                self.span
            }
        }
    }
}

fn gen_token_kind_enum(terminals: &[(TerminalId, String)]) -> TokenStream {
    let terminal_ids: Vec<_> = terminals.iter().map(|(terminal_id, name)|{
        let ident = format_ident!("T{}", terminal_id.0);
        quote! {
            #[comment = #name]
            #ident
        }
    }).collect();
    quote! {
        #[derive(Debug)]
        enum TokenKind {
            #(#terminal_ids),*
        }
    }
}

fn gen_token_kind_impl(terminals: &[(TerminalId, String)]) -> TokenStream {
    let terminal_ids: Vec<_> = terminals.iter().map(|(id, name)|{
        let ident = format_ident!("T{}", id.0);
        quote! {
            TokenKind::#ident => #name
        }
    }).collect();
    quote! {
        impl TokenKind {
            pub fn name(&self) -> &'static str {
                match self {
                    #(#terminal_ids,)*
                    _ => unreachable!()
                }
            }
        }
    }
}

fn gen_token_kind_function(terminals: &[(TerminalId, String)]) -> TokenStream {
    let cases: Vec<TokenStream> = terminals
        .iter()
        .map(|(id, name)| {
            let ident = format_ident!("T{}", id.0);
            quote! { 
                #[comment = #name]
                #id => TokenKind::#ident 
            }
        })
        .collect();
    quote! {
        fn token_kind(terminal_id: TerminalId) -> TokenKind {
            match terminal_id {
                #(#cases,)*
                _ => unreachable!("Unknown TerminalId: {:?}", terminal_id),
            }
        }
    }
}

fn gen_parse_tree_builder_impl(
    grammar: &Grammar,
    nonterminal_ids: &NonterminalIds,
    slot_ids: &SlotIds,
) -> TokenStream {
    let builder_name_ident = format_ident!("{}ParseTreeBuilder", grammar.name);
    let nonterminal_node_method = gen_nonterminal_node_method(grammar, nonterminal_ids, slot_ids);
    let new_token_method = gen_new_token_method();
    quote! {
        pub struct #builder_name_ident;
        impl ParseTreeBuilder<ParseTree> for #builder_name_ident {
            #nonterminal_node_method
            #new_token_method
        }
    }
}

fn field_names(grammar: &Grammar, alternative: &Alternative) -> Vec<Ident> {
    let counts = count_symbol_occurrences(grammar, &alternative.symbols);
    alternative
        .symbols
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let base_name = get_symbol_base_name(grammar, s);
            let needs_index = base_name.map_or(false, |name| counts.get(&name).copied().unwrap_or(0) > 1);
            Ident::new(&gen_field_name(grammar, s, i, needs_index), Span::call_site())
        })
        .collect::<Vec<_>>()
}

fn gen_nonterminal_node_method(
    grammar: &Grammar,
    nonterminal_ids: &NonterminalIds,
    slot_ids: &SlotIds,
) -> TokenStream {
    let nonterminal_cases: Vec<TokenStream> = nonterminal_ids
        .ids()
        .map(|nonterminal_id| {
            let nonterminal = nonterminal_ids.get_nonterminal(nonterminal_id);
            let slot_cases: Vec<TokenStream> = nonterminal_ids
                .end_slots(nonterminal_id)
                .map(|end_slot| {
                    let index = end_slot.index;
                    let alternatives = grammar.alternatives(nonterminal);
                    let alternative = &alternatives[index];
                    let end_slot_id = end_slot.slot_id;
                    let slot_name = slot_ids.display_name(&end_slot.slot_id);
                    let num_symbols = alternative.len();
                    let field_names = field_names(grammar, alternative);
                    let methods: Vec<_> = alternative
                        .symbols
                        .iter()
                        .map(|s| {
                            let def_id = s.resolved_def();
                            let def = grammar.definition(def_id);
                            match def {
                                Definition::Terminal(_) => {
                                    (Ident::new("unwrap_token", Span::call_site()), false)
                                },
                                Definition::Nonterminal(nt) => {
                                    let ident = format_ident!("unwrap_{}", to_snake_case(def.name()));
                                    (ident, should_be_boxed(nt, nonterminal))
                                }
                            }})
                        .collect();
                    let method_calls: Vec<_> = field_names
                        .iter()
                        .cloned()
                        .zip(methods)
                        .map(|(child, (method, should_be_boxed))| {
                            if should_be_boxed {
                                quote! {
                                    Box::new(#child.#method())
                                }
                            } else {
                                quote! { #child.#method() }
                            }
                        })
                        .collect();
                    let nonterminal_type = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
                    let num_alternatives = grammar.alternatives(nonterminal).len();
                    let construction = if num_alternatives == 1 {
                        quote! {
                            #nonterminal_type {
                                #(#field_names: #method_calls,)*
                                span: nonterminal_node.span,
                            }
                        }
                    } else {
                        let variant = Ident::new(
                            &to_pascal_case(&alternative_label(alternative, index)),
                            Span::call_site()
                        );
                        quote! {
                            #nonterminal_type::#variant {
                                #(#field_names: #method_calls,)*
                                span: nonterminal_node.span,
                            }
                        }
                    };
                    quote! {
                        #[comment = #slot_name]
                        #end_slot_id => {
                            let [#(#field_names),*] = <[ParseTree; #num_symbols]>::try_from(children).unwrap();
                            #construction.into()
                        }
                    }
                })
                .collect();
            let nonterminal_name = &nonterminal.name;
            quote! {
                #[comment = #nonterminal_name]
                #nonterminal_id => match nonterminal_node.return_slot {
                    #(#slot_cases),*,
                    _ => unreachable!()
                }
            }
        })
        .collect();
    quote! {
        fn new_nonterminal_node(
            &self,
            nonterminal_node: &NonterminalNode,
            children: OneOrMany<ParseTree>
        ) -> ParseTree {
            let children = children.into_vec();
            match nonterminal_node.nonterminal_id {
                #(#nonterminal_cases),*
                _ => unreachable!()
            }
        }
    }
}

// Returns true if the nonterminal corresponding to a symbol in the body of a rule has
// the same name as the nonterminal head, or it's origin symbol.
// This is to properly Box the generated types for recursive types, e.g., A+ or A*.
fn should_be_boxed(nonterminal: &Nonterminal, head: &Nonterminal) -> bool {
    if nonterminal.name == head.name {
        return true;
    }
    match &head.origin {
        Some(s) => match s {
            Symbol::Star(symbol, _) => symbol_contains_nonterminal(symbol, &nonterminal.name),
            Symbol::Plus(symbol, _) => symbol_contains_nonterminal(symbol, &nonterminal.name),
            Symbol::Group(symbols) => symbols.iter().any(|s| symbol_contains_nonterminal(s, &nonterminal.name)),
            Symbol::Opt(symbol) => symbol_contains_nonterminal(symbol, &nonterminal.name),
            Symbol::Alt(symbols) => symbols.iter().any(|s| symbol_contains_nonterminal(s, &nonterminal.name)),
            _ => false
        },
        None => false,
    }
}

// Recursively checks if a symbol contains a reference to a nonterminal with the given name.
fn symbol_contains_nonterminal(symbol: &Symbol, name: &str) -> bool {
    match symbol {
        Symbol::Identifier(identifier) => identifier.name == name,
        Symbol::Group(symbols) => symbols.iter().any(|s| symbol_contains_nonterminal(s, name)),
        Symbol::Labeled { symbol, .. } => symbol_contains_nonterminal(symbol, name),
        Symbol::Opt(inner) => symbol_contains_nonterminal(inner, name),
        Symbol::Alt(symbols) => symbols.iter().any(|s| symbol_contains_nonterminal(s, name)),
        Symbol::Star(inner, sep) => {
            symbol_contains_nonterminal(inner, name)
                || sep.as_ref().map_or(false, |s| symbol_contains_nonterminal(s, name))
        },
        Symbol::Plus(inner, sep) => {
            symbol_contains_nonterminal(inner, name)
                || sep.as_ref().map_or(false, |s| symbol_contains_nonterminal(s, name))
        },
        Symbol::Literal(_) => false,
    }
}

fn gen_new_token_method() -> TokenStream {
    quote! {
        fn new_token(&self, terminal_node: &TerminalNode) -> ParseTree {
            ParseTree::Token(Token {
                kind: token_kind(terminal_node.terminal_id),
                span: terminal_node.span,
            })
        }
    }
}

fn gen_parse_tree_enum(grammar: &Grammar) -> TokenStream {
    let arms: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let name = to_pascal_case(&n.name);
            let display_name = n.display_name();
            let ident = Ident::new(&name, Span::call_site());
            quote! { 
                #[comment = #display_name]
                #ident(#ident) 
            }
        })
        .collect();
    quote! {
        #[derive(Debug)]
        pub enum ParseTree {
            #(#arms),*,
            Token(Token)
        }
    }
}

fn gen_parse_tree_impl(grammar: &Grammar) -> TokenStream {
    let as_parse_tree_ref_method = gen_as_parse_tree_ref_method_for_parse_tree(grammar);
    let unwrap_methods = gen_unwrap_methods(grammar);
    quote! {
        impl ParseTree {
            #as_parse_tree_ref_method
            #(#unwrap_methods)*
            fn unwrap_token(self) -> Token {
                match self {
                    ParseTree::Token(t) => t,
                    _ => panic!(),
                }
            }
        }
    }
}

fn gen_as_parse_tree_ref_method_for_parse_tree(grammar: &Grammar) -> TokenStream {
    let arms = grammar.nonterminals().map(|n| {
        let name = &n.name;
        let variant = Ident::new(&to_pascal_case(name), Span::call_site());
        let var = Ident::new(&to_snake_case(name), Span::call_site());
        quote! { ParseTree::#variant(#var) => #var.as_parse_tree_ref() }
    });
    quote! {
        pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
            match self {
                #(#arms),*,
                ParseTree::Token(token) => token.as_parse_tree_ref(),
            }
        }
    }
}


fn gen_unwrap_methods(grammar: &Grammar) -> Vec<TokenStream> {
    grammar
        .nonterminals()
        .map(|n| {
            let method_ident = format_ident!("unwrap_{}", to_snake_case(&n.name));
            let return_type_ident = Ident::new(&to_pascal_case(&n.name), Span::call_site());
            let var_ident = Ident::new(&to_snake_case(&n.name), Span::call_site());
            quote! {
                fn #method_ident(self) -> #return_type_ident {
                    match self {
                        ParseTree::#return_type_ident(#var_ident) => #var_ident,
                        _ => panic!(),
                    }
                }
            }
        })
        .collect()
}

fn gen_parse_tree_ref_enum(grammar: &Grammar) -> TokenStream {
    let variants: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let name = to_pascal_case(&n.name);
            let ident = Ident::new(&name, Span::call_site());
            quote! { #ident(&'a #ident) }
        })
        .collect();
    quote! {
        #[derive(Clone, Copy)]
        pub enum ParseTreeRef<'a> {
            #(#variants),*,
            Token(&'a Token),
        }
    }
}

fn gen_parse_tree_ref_impl(grammar: &Grammar) -> TokenStream {
    let name_method = gen_name_method(grammar);
    let children_method = gen_children_method(grammar);
    let child_count_method = gen_child_count_method_for_parse_tree_ref(grammar);
    let span_method = gen_span_method_for_parse_tree_ref(grammar);
    quote! {
        impl<'a> ParseTreeRef<'a> {
            #children_method
            #name_method
            #child_count_method
            #span_method
        }
    }
}

fn gen_children_method(grammar: &Grammar) -> TokenStream {
    let arms: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let variant = Ident::new(&to_pascal_case(&n.name), Span::call_site());
            let var_ident = Ident::new(&to_snake_case(&n.name), Span::call_site());
            if n.is_plus() || n.is_star() {
                quote! {
                    ParseTreeRef::#variant(#var_ident) => #var_ident.iter().collect()
                }
            } else {
                quote! {
                    ParseTreeRef::#variant(#var_ident) => (0..#var_ident.child_count())
                        .filter_map(|i| #var_ident.child(i))
                        .collect()
                }
            }
        })
        .collect();
    quote! {
        pub fn children(&self) -> Vec<ParseTreeRef<'a>> {
            match self {
                #(#arms),*,
                ParseTreeRef::Token(_) => vec![],
            }
        }
    }
}    

fn gen_name_method(grammar: &Grammar) -> TokenStream {
    let arms = grammar.nonterminals().map(|n| {
        let display_name = &n.display_name();
        let name_ident = Ident::new(&to_pascal_case(&n.name), Span::call_site());
        quote! { ParseTreeRef::#name_ident(_) => #display_name }
    });
    quote! {
        pub fn display_name(&self) -> &'static str {
            match self {
                #(#arms),*,
                ParseTreeRef::Token(token) => token.kind.name(),
            }
        }
    }
}

fn gen_child_count_method_for_parse_tree_ref(grammar: &Grammar) -> TokenStream {
    let arms: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let variant = Ident::new(&to_pascal_case(&n.name), Span::call_site());
            let var_ident = Ident::new(&to_snake_case(&n.name), Span::call_site());
            quote! {
                ParseTreeRef::#variant(#var_ident) => #var_ident.child_count()
            }
        })
        .collect();
    quote! {
        pub fn child_count(&self) -> usize {
            match self {
                #(#arms),*,
                ParseTreeRef::Token(_) => 0,
            }
        }
    }
}

fn gen_span_method_for_parse_tree_ref(grammar: &Grammar) -> TokenStream {
    let arms: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let variant = Ident::new(&to_pascal_case(&n.name), Span::call_site());
            let var_ident = Ident::new(&to_snake_case(&n.name), Span::call_site());
            quote! {
                ParseTreeRef::#variant(#var_ident) => #var_ident.span()
            }
        })
        .collect();
    quote! {
        pub fn span(&self) -> Span {
            match self {
                #(#arms),*,
                ParseTreeRef::Token(token) => token.span(),
            }
        }
    }
}

fn gen_list_node_trait() -> TokenStream {
    quote! {
        pub trait ListNode<'a> {
            fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>>;
        }
    }
}

fn gen_opt_node_trait() -> TokenStream {
    quote! {
        pub trait OptNode {
            type Inner;
            fn value(&self) -> Option<&Self::Inner>;
        }
    }
}

fn gen_opt_node_impl(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    let opt_type = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
    let alternatives = grammar.alternatives(nonterminal);
    let alt0 = &alternatives[0];
    let inner_symbol = &alt0.symbols[0];
    let inner_def = grammar.definition(inner_symbol.resolved_def());
    let inner_type = match inner_def {
        Definition::Terminal(_) => Ident::new("Token", Span::call_site()),
        Definition::Nonterminal(_) => Ident::new(&to_pascal_case(inner_def.name()), Span::call_site()),
    };
    let field_name = Ident::new(&gen_field_name(grammar, inner_symbol, 0, false), Span::call_site());

    quote! {
        impl OptNode for #opt_type {
            type Inner = #inner_type;
            fn value(&self) -> Option<&Self::Inner> {
                match self {
                    #opt_type::Alt0 { #field_name, .. } => Some(#field_name),
                    #opt_type::Alt1 { .. } => None,
                }
            }
        }
    }
}

fn gen_alt_accessors(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    let alt_type = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
    let alternatives = grammar.alternatives(nonterminal);

    let accessors: Vec<_> = alternatives
        .iter()
        .enumerate()
        .map(|(i, alt)| {
            let symbol = &alt.symbols[0];
            let def = grammar.definition(symbol.resolved_def());
            let (method_name, return_type) = match def {
                Definition::Terminal(t) => {
                    let method = format_ident!("as_{}", to_snake_case(&t.name));
                    let ret = Ident::new("Token", Span::call_site());
                    (method, ret)
                }
                Definition::Nonterminal(nt) => {
                    let method = format_ident!("as_{}", to_snake_case(&nt.name));
                    let ret = Ident::new(&to_pascal_case(&nt.name), Span::call_site());
                    (method, ret)
                }
            };
            let variant = format_ident!("Alt{}", i);
            let field_name = Ident::new(&gen_field_name(grammar, symbol, 0, false), Span::call_site());

            quote! {
                pub fn #method_name(&self) -> Option<&#return_type> {
                    match self {
                        #alt_type::#variant { #field_name, .. } => Some(#field_name),
                        _ => None,
                    }
                }
            }
        })
        .collect();

    quote! {
        impl #alt_type {
            #(#accessors)*
        }
    }
}

/// Generates a typed accessor for Plus/Star nonterminals that returns
/// an iterator over the direct children. Relies on the iter() method
/// that returns all children including layout.
fn gen_typed_accessor(nonterminal: &Nonterminal) -> Option<TokenStream> {
    let child_name = match &nonterminal.origin {
        Some(Symbol::Plus(symbol, _)) | Some(Symbol::Star(symbol, _)) => match symbol.as_ref() {
            Symbol::Identifier(ident) => &ident.name,
            _ => return None, // Skip complex nested EBNF for now
        },
        _ => return None,
    };

    let child_type = Ident::new(&to_pascal_case(child_name), Span::call_site());
    let method_name = format_ident!("{}", pluralize(&to_snake_case(child_name)));
    let variant = Ident::new(&to_pascal_case(child_name), Span::call_site());

    Some(quote! {
        pub fn #method_name(&self) -> impl Iterator<Item = &#child_type> {
            self.iter().filter_map(|node| match node {
                ParseTreeRef::#variant(r) => Some(r),
                _ => None,
            })
        }
    })
}

fn pluralize(word: &str) -> String {
    if word.ends_with("s") || word.ends_with("x") || word.ends_with("ch") || word.ends_with("sh") {
        format!("{}es", word)
    } else if word.ends_with("y") && !word.ends_with("ay") && !word.ends_with("ey") && !word.ends_with("oy") && !word.ends_with("uy") {
        format!("{}ies", &word[..word.len() - 1])
    } else {
        format!("{}s", word)
    }
}

fn gen_list_node_impl_for_plus(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    let ident = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
    let alternatives = grammar.alternatives(nonterminal);
    // This method must only be called for list nodes, i.e., * and + nonterminals,
    // which always have two alternatives.
    assert_eq!(alternatives.len(), 2);
    let label = alternative_label(&alternatives[0], 0);
    let alt_variant = Ident::new(&to_pascal_case(&label), Span::call_site());
    let first_alt_fields = field_names(grammar, &alternatives[0]);
    let first_arm = match &nonterminal.origin {
        Some(Symbol::Plus(_symbol, sep)) => {
            match sep {
                Some(_) => {
                    let (f0, f1, f2, f3, f4) = (&first_alt_fields[0], &first_alt_fields[1], &first_alt_fields[2], &first_alt_fields[3], &first_alt_fields[4]);
                    quote! {
                        #ident::#alt_variant { #f0: rest, #f1: layout1, #f2: sep, #f3: layout2, #f4: item, .. } => {
                            items.push(item.as_parse_tree_ref());
                            items.push(layout2.as_parse_tree_ref());
                            items.push(sep.as_parse_tree_ref());
                            items.push(layout1.as_parse_tree_ref());
                            current = rest;
                        }
                    }
                },
                None => {
                    let (f0, f1, f2) = (&first_alt_fields[0], &first_alt_fields[1], &first_alt_fields[2]);
                    quote! {
                        #ident::#alt_variant { #f0: rest, #f1: layout, #f2: item, .. } => {
                            items.push(item.as_parse_tree_ref());
                            items.push(layout.as_parse_tree_ref());
                            current = rest;
                        }
                    }
                },
            }
        },
        _ => unreachable!("Expected plus"),
    };
    let label = alternative_label(&alternatives[1], 1);
    let alt_variant = Ident::new(&to_pascal_case(&label), Span::call_site());
    let second_alt_fields = field_names(grammar, &alternatives[1]);
    let f0 = &second_alt_fields[0];
    let second_arm = quote! {
        #ident::#alt_variant { #f0: item, .. } => {
            items.push(item.as_parse_tree_ref());
            break;
        }
    };
    quote! {
        impl<'a> ListNode<'a> for #ident {
            fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
                let mut items = vec![];
                let mut current = self;
                loop {
                    match current {
                        #first_arm
                        #second_arm
                    }
                }
                items.reverse();
                items.into_iter()
            }
        }
    }
}

fn gen_list_node_impl_for_star(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    let star_ident = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
    let alternatives = grammar.alternatives(nonterminal);
    let first_symbol = &alternatives[0].symbols[0];
    let field_name = Ident::new(&gen_field_name(grammar, first_symbol, 0, false), Span::call_site());
    let def_id = first_symbol.resolved_def();
    let nonterminal = grammar.definition(def_id).as_nonterminal();
    let alternatives = grammar.alternatives(nonterminal);

    let opt_ident = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
    let var_ident = Ident::new(&to_snake_case(&nonterminal.name), Span::call_site());
    let label = alternative_label(&alternatives[0], 0);
    let alt_variant = Ident::new(&to_pascal_case(&label), Span::call_site());
    let first_alt_fields = field_names(grammar, &alternatives[0]);
    let f0 = &first_alt_fields[0];
    let first_arm = quote! {
        #opt_ident::#alt_variant { #f0: #var_ident, .. } => #var_ident.iter(),
    };
    let label = alternative_label(&alternatives[1], 1);
    let alt_variant = Ident::new(&to_pascal_case(&label), Span::call_site());
    let second_arm = quote! {
        #opt_ident::#alt_variant { .. } => vec![].into_iter(),
    };
    quote! {
        impl<'a> ListNode<'a> for #star_ident {
            fn iter(&'a self) -> IntoIter<ParseTreeRef<'a>> {
                match &self.#field_name {
                    #first_arm
                    #second_arm
                }
            }
        }
    }
}

fn gen_from_for_tree_impls(grammar: &Grammar) -> TokenStream {
    let from_impls: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let type_ident = Ident::new(&to_pascal_case(&n.name), Span::call_site());
            let ident = Ident::new(&to_snake_case(&n.name), Span::call_site());
            quote! {
                impl From<#type_ident> for ParseTree {
                    fn from(#ident: #type_ident) -> Self {
                        ParseTree::#type_ident(#ident)
                    }
                }
            }
        })
        .collect();
    quote! { #(#from_impls)* }
}


fn gen_create_parse_tree_function(grammar: &Grammar) -> TokenStream {
    let parser_name_ident = format_ident!("{}Parser", grammar.name);
    let builder_name_ident = format_ident!("{}ParseTreeBuilder", grammar.name);
    let arms: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let name = &n.name;
            let function_name = format_ident!("create_parse_tree_{}", to_snake_case(name));
            let variant_name = Ident::new(&to_pascal_case(name), Span::call_site());
            quote! { #name => ParseTree::#variant_name(#function_name(root_id, parser, builder)) }
        })
        .collect();
    quote! {
        pub fn create_parse_tree(
            root_id: SPPFNodeId,
            name: &str,
            parser: &#parser_name_ident,
            builder: &#builder_name_ident,
        ) -> ParseTree {
            match name {
                #(#arms),*,
                _ => panic!()
            }
        }
    }
}

/// Generates functions with the name create_parse_tree_#name, where name is the name of a nonterminal. 
fn gen_create_parse_tree_nonterminal_function(grammar: &Grammar, nonterminal_name: &str) -> TokenStream {
    let parser_name_ident = format_ident!("{}Parser", grammar.name);
    let builder_name_ident = format_ident!("{}ParseTreeBuilder", grammar.name);
    let return_type = Ident::new(&to_pascal_case(nonterminal_name), Span::call_site());
    let function_name = format_ident!("create_parse_tree_{}", to_snake_case(nonterminal_name));
    let unwrap_method = format_ident!("unwrap_{}", to_snake_case(nonterminal_name));
    quote! {
        pub fn #function_name(
            root_id: SPPFNodeId,
            parser: &#parser_name_ident,
            builder: &#builder_name_ident,
        ) -> #return_type {
            let node = parser.sppf_node(root_id);
            visit_sppf(node, parser, builder).unwrap_one().#unwrap_method()
        }
    }
}

fn gen_to_sexpr_function() -> TokenStream {
    quote! {
        pub fn to_sexpr(node: ParseTreeRef<'_>) -> String {
            let mut s = String::new();
            node_to_sexpr(node, 0, &mut s).expect("error");
            s
        }
    }
}

fn gen_node_to_sexpr_function() -> TokenStream {
    quote! {
        fn node_to_sexpr(node: ParseTreeRef<'_>, indent: usize, w: &mut impl Write) -> fmt::Result {
            let children = node.children();
            if children.is_empty() {
                writeln!(w, "{:indent$}{}", "", node.display_name())
            } else {
                writeln!(w, "{:indent$}({}", "", node.display_name())?;
                for child in children {
                    node_to_sexpr(child, indent + 2, w)?;
                }
                writeln!(w, "{:indent$})", "")
            }
        }
    }
}

fn gen_to_json_function() -> TokenStream {
    quote! {
        /// Converts a parse tree to JSON format for visualization.
        /// Returns a JSON string with nodes and edges arrays.
        pub fn to_json(node: ParseTreeRef<'_>) -> String {
            let mut nodes = Vec::new();
            let mut edges = Vec::new();
            let mut next_id = 0u32;
            build_json_graph(node, &mut nodes, &mut edges, &mut next_id);

            let result = serde_json::json!({
                "nodes": nodes,
                "edges": edges
            });

            result.to_string()
        }

        fn build_json_graph(
            node: ParseTreeRef<'_>,
            nodes: &mut Vec<serde_json::Value>,
            edges: &mut Vec<serde_json::Value>,
            next_id: &mut u32,
        ) -> u32 {
            let my_id = *next_id;
            *next_id += 1;

            let span = node.span();
            let kind = match node {
                ParseTreeRef::Token(_) => "Token",
                _ => "Nonterminal",
            };

            nodes.push(serde_json::json!({
                "id": my_id,
                "kind": kind,
                "label": node.display_name(),
                "start": span.left_extent,
                "end": span.right_extent
            }));

            for child in node.children() {
                let child_id = build_json_graph(child, nodes, edges, next_id);
                edges.push(serde_json::json!({
                    "src": my_id,
                    "dest": child_id
                }));
            }

            my_id
        }
    }
}
