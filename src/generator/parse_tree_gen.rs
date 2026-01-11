use itertools::Itertools;
use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    generator::{id::{NonterminalIds, SlotIds, TerminalIds}, utils::{alternative_label, to_first_uppercase, to_pascal_case, to_snake_case}},
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
    let list_node_impls: Vec<_> = grammar
        .nonterminals()
        .filter(|n| n.is_plus() || n.is_star())
        .map(|n| gen_list_node_impl(grammar, n))
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
        #(#nonterminal_types)*
        #(#nonterminal_types_impl)*
        #(#list_node_impls)*
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
        use std::fmt::Write;
        use iguana::{
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
    let nonterminal_name = to_pascal_case(&nonterminal.name);
    let nonterminal_name_id = Ident::new(&nonterminal_name, Span::call_site());
    if alternatives.len() == 1 {
        gen_nonterminal_type_with_one_alternative(grammar, &nonterminal_name_id, &alternatives[0])
    } else {
        gen_nonterminal_type_with_more_than_one_alternative(grammar, nonterminal, &nonterminal_name_id, alternatives)
    }
}

fn gen_nonterminal_type_with_one_alternative(
    grammar: &Grammar, 
    nonterminal_name_id: &Ident, 
    alternative: &Alternative
) -> TokenStream {
    let fields: Vec<_> = alternative
        .symbols
        .iter()
        .map(|s| {
            let def_id = s.resolved_def();
            let def = grammar.definition(def_id);
            match def {
                Definition::Terminal(_) => Ident::new("Token", Span::call_site()),
                Definition::Nonterminal(_) => Ident::new(&to_pascal_case(def.name()), Span::call_site()),
            }
        })
        .collect();
    // Add Span as last field
    quote! {
        #[derive(Debug)]
        pub struct #nonterminal_name_id(#(#fields,)* Span);
    }
}

fn gen_nonterminal_type_with_more_than_one_alternative(
    grammar: &Grammar, 
    nonterminal: &Nonterminal,
    nonterminal_name_id: &Ident, 
    alternatives: &[Alternative]
) -> TokenStream {
        let variants: Vec<_> = alternatives
            .iter()
            .enumerate()
            .map(|(index, alternative)| {
                let children: Vec<_> = alternative
                    .symbols
                    .iter()
                    .map(|s| {
                        let def_id = s.resolved_def();
                        let def = grammar.definition(def_id);
                        match def {
                            Definition::Terminal(_) => {
                                let token = Ident::new("Token", Span::call_site());
                                quote! { #token }
                            },
                            Definition::Nonterminal(_) => {
                                if def.name() == nonterminal.name {
                                    let name = Ident::new(&to_pascal_case(def.name()), Span::call_site());
                                    quote! { Box<#name> }
                                } else {
                                    let name = Ident::new(&to_pascal_case(def.name()), Span::call_site());
                                    quote! { #name }
                                }
                            },
                        }
                    })
                    .collect();
                let label = alternative_label(alternative, index);
                let variant_name = Ident::new(&label, Span::call_site());
                // Add Span as last field in each variant
                quote! {
                    #variant_name(#(#children,)* Span)
                }
            })
            .collect();
        quote! {
            #[derive(Debug)]
            pub enum #nonterminal_name_id {
                #(#variants),*
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
    quote! {
        impl #nonterminal_name {
            #child_method
            #child_count_method
            #as_node_ref_method
            #span_method
        }
    }
}

fn gen_span_method(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    let ident = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
    let alternatives = grammar.alternatives(nonterminal);
    let body = if alternatives.len() == 1 {
        // Single alternative: span is the last field
        let span_index = Literal::usize_unsuffixed(alternatives[0].symbols.len());
        quote! {
            self.#span_index
        }
    } else {
        // Multiple alternatives: pattern match to extract span from each variant
        let arms: Vec<_> = alternatives.iter().enumerate().map(|(i, alternative)| {
            let label = alternative_label(alternative, i);
            let alt_variant = Ident::new(&to_pascal_case(&label), Span::call_site());
            // Span is the last field, use .. to match the rest
            quote! {
                #ident::#alt_variant(.., span) => *span
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
        let body = child_by_index(alternative, true);
        quote! {
            match index {
                #body
            }
        }
    } else {
        let arms: Vec<_> = alternatives.iter().enumerate().map(|(i, alternative)| {
            let label = alternative_label(alternative, i);
            let alt_variant = Ident::new(&to_pascal_case(&label), Span::call_site());
            let children_names = children_names(alternative);
            let body = child_by_index(alternative, false);
            // Add _ to ignore the span field at the end
            quote! {
                #ident::#alt_variant(#(#children_names,)* _) => #body
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
fn child_by_index(alternative: &Alternative, single_rule: bool) -> TokenStream {
    let cases: Vec<_> = (0..alternative.symbols.len()).map(|i| {
        let i_lit = Literal::usize_unsuffixed(i);
        // For nonterminals with only one body, i.e., no alternatives,
        // generate the arms as 0 => Some(self.0.as_parse_tree_ref())
        // As, we can index the children directly.
        if single_rule {
            quote! {
                #i_lit => Some(self.#i_lit.as_parse_tree_ref())
            }
        } else {
            // For nonterminals with alternatives, we need to return the exact child:
            // case E::Plus(c0, c1, c2) {
            //     match index {
            //         0 => Some(c0.as_parse_tree_ref()),
            //         1 => Some(c1.as_parse_tree_ref()),
            //         2 => Some(c2.as_parse_tree_ref()),
            //         _ => unreachable!()
            // }
            let child_name = format_ident!("c{}", i);
            quote! {
                #i_lit => Some(#child_name.as_parse_tree_ref())
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
            // Use .. to match any fields (including span)
            quote! {
                #ident::#alt_variant(..) => #count_symbols
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

fn children_names(alternative: &Alternative) -> Vec<Ident> {
    let num_symbols = alternative.len();
    (0..num_symbols)
        .map(|i| Ident::new(&format!("c{i}"), Span::call_site()))
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
            let nonterminal_name = &nonterminal.name;
            let slot_cases: Vec<TokenStream> = nonterminal_ids
                .end_slots(nonterminal_id)
                .map(|end_slot| {
                    let index = end_slot.index;
                    let alternatives = grammar.alternatives(nonterminal);
                    let alternative = &alternatives[index];
                    let end_slot_id = end_slot.slot_id;
                    let slot_name = slot_ids.display_name(&end_slot.slot_id);
                    let num_symbols = alternative.len();
                    let children_names = children_names(alternative);
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
                                Definition::Nonterminal(_) => { 
                                    let ident = format_ident!("unwrap_{}", to_snake_case(def.name()));
                                    // Pass true if should be boxed.
                                    (ident, def.name() == nonterminal_name)
                                }
                            }})
                        .collect();
                    let method_calls: Vec<_> = children_names
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
                    // Todo: handle 0
                    let num_alternatives = grammar
                        .alternatives(nonterminal)
                        .len();
                    let constructor = if num_alternatives == 1 {
                        quote! {
                            #nonterminal_type
                        }
                    } else {
                        let variant = Ident::new(
                            &to_pascal_case(&alternative_label(alternative, index)), 
                            Span::call_site()
                        );
                        quote! {
                            #nonterminal_type::#variant
                        }
                    };
                    quote! {
                        #[comment = #slot_name]
                        #end_slot_id => {
                            let [#(#children_names),*] = <[ParseTree; #num_symbols]>::try_from(children).unwrap();
                            #constructor(#(#method_calls,)* nonterminal_node.span).into()
                        }
                    }
                })
                .collect();
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
    let variants: Vec<_> = grammar
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
            #(#variants),*,
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
                    ParseTreeRef::#variant(#var_ident) => #var_ident.iter().map(|a| a.as_parse_tree_ref()).collect()
                }
            } else {
                quote! {
                    ParseTreeRef::#variant(#var_ident) => (0..#var_ident.child_count()).filter_map(|i| #var_ident.child(i)).collect()
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
        trait ListNode {
            type Item;
            fn iter(&self) -> impl Iterator<Item = &Self::Item>;
        }
    }
}

fn gen_list_node_impl(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    let ident = Ident::new(&to_pascal_case(&nonterminal.name), Span::call_site());
    let alternatives = grammar.alternatives(nonterminal);
    // This method must only be called for list nodes, i.e., * and + nonterminals,
    // which always have two alternatives.
    assert_eq!(alternatives.len(), 2);
    let label = alternative_label(&alternatives[0], 0);
    let alt_variant = Ident::new(&to_pascal_case(&label), Span::call_site());
    let first_arm = quote! {
        // Add _ to ignore the span field at the end
        #ident::#alt_variant(rest, item, _) => {
            items.push(item);
            current = rest;
        }
    };
    let label = alternative_label(&alternatives[1], 1);
    let alt_variant = Ident::new(&to_pascal_case(&label), Span::call_site());
    let second_arm = if nonterminal.is_plus() {
        quote! {
            // Add _ to ignore the span field at the end
            #ident::#alt_variant(item, _) => {
                items.push(item);
                break;
            }
        }
    } else {
        // The star node's second alternative is empty
        quote! {
            // Add _ to ignore the span field at the end
            #ident::#alt_variant(_) => {
                break;
            }
        }
    };
    let origin = nonterminal.origin.as_ref().unwrap();
    let inner_symbol = match origin {
        Symbol::Plus(symbol) | Symbol::Star(symbol, _) => {
            if let Some(ebnf_symbol) = grammar.ebnf_symbol(symbol) {
                &ebnf_symbol.as_identifier()
            } else {
                &symbol.as_identifier()
            }
        }
        _ => panic!("Expected a Star or Plus symbol but got {}", origin)
    };

    let def_id = inner_symbol.definition.unwrap_or_else(|| panic!("{} is not resolved", &inner_symbol.name));
    let name = match grammar.definition(def_id) {
        Definition::Terminal(_) => "Token",
        Definition::Nonterminal(nonterminal) => &to_pascal_case(&nonterminal.name),
    };
    let inner_nonterminal_ident = Ident::new(name, Span::call_site());
    quote! {
        impl ListNode for #ident {
            type Item = #inner_nonterminal_ident;
            fn iter(&self) -> impl Iterator<Item = &#inner_nonterminal_ident> {
                let mut items = vec![];
                let mut current = self;
                loop {
                    match current {
                        #first_arm
                        #second_arm
                    }
                }
                items.into_iter().rev()
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
