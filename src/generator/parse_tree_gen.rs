use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    generator::{id::{NonterminalIds, SlotIds, TerminalIds}, utils::{to_first_lowercase, to_first_uppercase}},
    grammar::{grammar::{Alternative, Grammar}, symbols::{Nonterminal, Symbol}},
};

pub fn generate(
    grammar: &Grammar,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds,
) -> TokenStream {
    let token_names: Vec<Ident> = terminal_ids
        .ids()
        .map(|id| syn::Ident::new(&format!("T{}", id.index()), Span::call_site()))
        .collect();
    let imports = gen_imports(grammar);
    let token_kind_enum = gen_token_kind_enum(&token_names);
    let token_kind_function = gen_token_kind_function(terminal_ids, &token_names);
    let token_struct = gen_token_struct();
    let parse_tree_enum = gen_parse_tree_enum(grammar);
    let parse_tree_impl = gen_parse_tree_impl(grammar);
    let from_for_tree_impls = gen_from_for_tree_impls(grammar);
    let parse_tree_builder_impl = gen_parse_tree_builder_impl(grammar, nonterminal_ids, slot_ids);
    let create_parse_tree_method = gen_create_parse_tree_method(grammar);

    let nonterminal_types: Vec<_> = grammar
        .nonterminals()
        .map(|n| gen_nonterminal_type(n, grammar.alternatives(n)))
        .collect();

    quote! {
        #imports
        #token_kind_enum
        #parse_tree_enum
        #parse_tree_impl
        #from_for_tree_impls
        #(#nonterminal_types)*
        #token_struct
        #token_kind_function
        #parse_tree_builder_impl
        #create_parse_tree_method
    }
}

fn gen_imports(grammar: &Grammar) -> TokenStream {
    let parser_name = format_ident!("{}Parser", to_first_uppercase(&grammar.name));
    quote! {
        use iguana::{
            parse_tree::{OneOrMany, ParseTreeBuilder, visit_sppf},
            parser::{NonterminalId, Parser, SlotId, TerminalId},
            sppf::{NonterminalNode, SPPFNodeId},
        };
        use crate::parser::#parser_name;
    }
}

fn gen_nonterminal_type(
    nonterminal: &Nonterminal,
    alternatives: Option<&Vec<Alternative>>,
) -> TokenStream {
    let nonterminal_name_id = Ident::new(&nonterminal.name, Span::call_site());
    if let Some(alternatives) = alternatives {
        // TODO: handle empty alternatives
        if alternatives.len() == 1 {
            let alternative = &alternatives[0];
            let fields: Vec<_> = alternative
                .symbols
                .iter()
                .map(|s| match s {
                    Symbol::Terminal(_) => Ident::new("Token", Span::call_site()),
                    Symbol::Nonterminal(n) => {
                        Ident::new(&to_first_uppercase(&n.name), Span::call_site())
                    }
                    _ => panic!(),
                })
                .collect();
            quote! {
                #[derive(Debug)]
                pub struct #nonterminal_name_id(#(#fields),*);
            }
        } else {
            let variants: Vec<_> = alternatives
                .iter()
                .enumerate()
                .map(|(index, alternative)| {
                    let children: Vec<_> = alternative
                        .symbols
                        .iter()
                        .map(|s| match s {
                            Symbol::Terminal(_) => {
                                let token = Ident::new("Token", Span::call_site());
                                quote! { #token }
                            }
                            Symbol::Nonterminal(n) => {
                                if n.name == nonterminal.name {
                                    let name = Ident::new(&n.name, Span::call_site());
                                    quote! { Box<#name> }
                                } else {
                                    let name = Ident::new(&n.name, Span::call_site());
                                    quote! { #name }
                                }
                            }
                            _ => unimplemented!(),
                        })
                        .collect();
                    let label = alternative_label(alternative, index);
                    let variant_name = Ident::new(&label, Span::call_site());
                    quote! {
                        #variant_name(#(#children),*)
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
    } else {
        quote! {}
    }
}

fn alternative_label(alternative: &Alternative, index: usize) -> Cow<'_, str> {
    match &alternative.label {
        Some(label) => Cow::Borrowed(label),
        None => Cow::Owned(format!("Alt{}", index)),
    }
}

fn gen_token_struct() -> TokenStream {
    quote! {
        #[derive(Debug)]
        pub struct Token {
            kind: TokenKind,
        }
    }
}

fn gen_token_kind_enum(token_names: &[Ident]) -> TokenStream {
    quote! {
        #[derive(Debug)]
        enum TokenKind {
            #(#token_names),*
        }
    }
}

fn gen_token_kind_function(terminal_ids: &TerminalIds, token_names: &[Ident]) -> TokenStream {
    let cases: Vec<TokenStream> = terminal_ids
        .ids()
        .map(|id| {
            let token_name = &token_names[id.index()];
            quote! { #id => TokenKind::#token_name }
        })
        .collect();
    quote! {
        fn token_kind(terminal_id: TerminalId) -> TokenKind {
            match terminal_id {
                #(#cases),*,
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
                    let alternative = &alternatives.unwrap()[index];
                    let end_slot_id = end_slot.slot_id;
                    let slot_name = slot_ids.slot_name(&end_slot.slot_id);
                    let num_symbols = alternative.len();
                    let children_names = (0..num_symbols)
                        .map(|i| Ident::new(&format!("c{i}"), Span::call_site()))
                        .collect::<Vec<_>>();
                    let methods: Vec<_> = alternative
                        .symbols
                        .iter()
                        .map(|s| match s {
                            Symbol::Terminal(_) => {
                                (Ident::new("unwrap_token", Span::call_site()), false)
                            }
                            Symbol::Nonterminal(n) => { 
                                let ident = format_ident!("unwrap_{}", to_first_uppercase(&n.name));
                                // Pass true if should be boxed.
                                (ident, n.name == *nonterminal_name)
                            }
                            _ => panic!(),
                        })
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
                    let nonterminal_type = Ident::new(&to_first_uppercase(&nonterminal.name), Span::call_site());
                    // Todo: handle 0
                    let num_alternatives = grammar
                        .alternatives(nonterminal)
                        .map(|alt| alt.len())
                        .unwrap_or_default();
                    let constructor = if num_alternatives == 1 {
                        quote! {
                            #nonterminal_type
                        }
                    } else {
                        let variant = Ident::new(
                            &to_first_uppercase(&alternative_label(alternative, index)), 
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
                            #constructor(#(#method_calls),*).into()
                        }
                    }
                })
                .collect();
            quote! {
                #[comment = #nonterminal_name]
                #nonterminal_id => match nonterminal_node.return_slot {
                    #(#slot_cases),*
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
        fn new_token(&self, terminal_id: TerminalId) -> ParseTree {
            ParseTree::Token(Token {
                kind: token_kind(terminal_id),
            })
        }
    }
}

fn gen_parse_tree_enum(grammar: &Grammar) -> TokenStream {
    let variants: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let ident = Ident::new(&n.name, Span::call_site());
            quote! { #ident(#ident) }
        })
        .collect();
    quote! {
        #[derive(Debug)]
        enum ParseTree {
            #(#variants),*,
            Token(Token)
        }
    }
}

fn gen_parse_tree_impl(grammar: &Grammar) -> TokenStream {
    let methods: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let method_ident = format_ident!("unwrap_{}", to_first_lowercase(&n.name));
            let return_type_ident = Ident::new(&to_first_uppercase(&n.name), Span::call_site());
            let var_ident = Ident::new(&to_first_lowercase(&n.name), Span::call_site());
            quote! {
                fn #method_ident(self) -> #return_type_ident {
                    match self {
                        ParseTree::#return_type_ident(#var_ident) => #var_ident,
                        _ => panic!(),
                    }
                }
            }
        })
        .collect();
    quote! {
        impl ParseTree {
            #(#methods)*
            fn unwrap_token(self) -> Token {
                match self {
                    ParseTree::Token(t) => t,
                    _ => panic!(),
                }
            }
        }
    }
}

fn gen_from_for_tree_impls(grammar: &Grammar) -> TokenStream {
    let from_impls: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let type_ident = Ident::new(&to_first_uppercase(&n.name), Span::call_site());
            let ident = Ident::new(&to_first_lowercase(&n.name), Span::call_site());
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

fn gen_create_parse_tree_method(grammar: &Grammar) -> TokenStream {
    let parser_name_ident = format_ident!("{}Parser", grammar.name);
    let builder_name_ident = format_ident!("{}ParseTreeBuilder", grammar.name);
    let return_type = Ident::new(&to_first_uppercase(&grammar.start_symbol.name), Span::call_site());
    let unwrap_method = format_ident!("unwrap_{}", to_first_lowercase(&grammar.start_symbol.name));
    quote! {
        pub fn create_parse_tree(
        root_id: SPPFNodeId,
        parser: &#parser_name_ident,
        builder: &#builder_name_ident,
    ) -> #return_type {
        let node = parser.sppf_node(root_id);
        visit_sppf(node, parser, builder).unwrap_one().#unwrap_method()
    }
    }
}
