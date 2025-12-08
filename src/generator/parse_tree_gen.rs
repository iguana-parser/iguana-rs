use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    generator::{id::{NonterminalIds, SlotIds, TerminalIds}, utils::{alternative_label, to_first_lowercase, to_first_uppercase}},
    grammar::{grammar::{Alternative, Grammar}, symbols::{Nonterminal, Symbol}},
};

pub fn generate(
    grammar: &Grammar,
    nonterminal_ids: &NonterminalIds,
    terminal_ids: &TerminalIds,
    slot_ids: &SlotIds,
) -> TokenStream {
    let terminals: Vec<(Ident, String)> = terminal_ids
        .ids()
        .zip(terminal_ids.terminals())
        .map(|(id, t)| (format_ident!("T{}", id.index()), t.to_string()))
        .collect();
    let imports = gen_imports(grammar);
    let token_kind_enum = gen_token_kind_enum(&terminals);
    let token_kind_impl = gen_token_kind_impl(&terminals);
    let token_kind_function = gen_token_kind_function(&terminals);
    let token_struct = gen_token_struct();
    let parse_tree_enum = gen_parse_tree_enum(grammar);
    let parse_tree_impl = gen_parse_tree_impl(grammar);
    let parse_tree_ref_enum = gen_parse_tree_as_ref_enum(grammar);
    let parse_tree_ref_impl = gen_parse_tree_ref_impl();
    let child_iter_struct = gen_child_iter_struct();
    let impl_iterator_for_child_iter = gen_impl_iterator_for_child_iter(grammar);
    let from_for_tree_impls = gen_from_for_tree_impls(grammar);
    let parse_tree_builder_impl = gen_parse_tree_builder_impl(grammar, nonterminal_ids, slot_ids);
    let create_parse_tree_method = gen_create_parse_tree_method(grammar);

    let nonterminal_types: Vec<_> = grammar
        .nonterminals()
        .map(|n| gen_nonterminal_type(n, grammar.alternatives(n)))
        .collect();

    let nonterminal_types_impl: Vec<_> = grammar
        .nonterminals()
        .map(|n| gen_nonterminal_type_impl(grammar, n))
        .collect();

    quote! {
        #imports
        #token_kind_enum
        #token_kind_impl
        #parse_tree_enum
        #parse_tree_impl
        #parse_tree_ref_enum
        #parse_tree_ref_impl
        #child_iter_struct
        #impl_iterator_for_child_iter
        #from_for_tree_impls
        #(#nonterminal_types)*
        #(#nonterminal_types_impl)*
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

fn gen_nonterminal_type_impl(
    grammar: &Grammar,
    nonterminal: &Nonterminal,
) -> TokenStream {
    let nonterminal_name = Ident::new(&nonterminal.name, Span::call_site());
    let child_method = gen_child_method(grammar, nonterminal);
    let as_node_ref_method = gen_as_parse_tree_ref(&nonterminal.name);
    quote! {
        impl #nonterminal_name {
            #child_method
            #as_node_ref_method
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
    let ident = Ident::new(&to_first_uppercase(&nonterminal.name), Span::call_site());
    if let Some(alternatives) = grammar.alternatives(nonterminal) {
        if alternatives.len() == 1 {
            let alternative = &alternatives[0];
            let body = child_by_index(alternative, true);
            quote! {
                match index {
                    #body
                }
            }
        } else {
            let cases: Vec<_> = alternatives.iter().enumerate().map(|(i, alternative)| {
                let label = alternative_label(alternative, i);
                let alt_variant = Ident::new(&to_first_uppercase(&label), Span::call_site());
                let children_names = children_names(alternative);
                let body = child_by_index(alternative, false);
                quote! {
                    #ident::#alt_variant(#(#children_names),*) => #body
                }
            }).collect();
            quote! {
                match self {
                    #(#cases),*
                }
            }
        }
    } else {
        // Handle empty alternatives later
        unreachable!()
    }
}

fn child_by_index(alternative: &Alternative, single_rule: bool) -> TokenStream {
    let cases = alternative.symbols.iter().enumerate().map(|(i, s)| {
        let name = match s {
            Symbol::Terminal(_) => "Token",
            Symbol::Nonterminal(nonterminal) => &nonterminal.name,
            _ => unreachable!()
        };
        let name_ident = Ident::new(name, Span::call_site());
        let i_lit = Literal::usize_unsuffixed(i);
        // For nonterminals with only one body, i.e., no alternatives,
        // generate the arms as 0 => Some(ParseTreeRef::E(&self.0))
        // As, we can index the children directly.
        if single_rule {
            quote! {
                #i_lit => Some(ParseTreeRef::#name_ident(&self.#i_lit))
            }
        } else {
            // For nonterminals with alternatives, we need to return the exact child:
            // case E::Plus(c0, c1, c2) {
            //     match index {
            //         0 => Some(ParseTreeRef::E(&c0)),
            //         1 => Some(ParseTreeRef::E(&c1)),
            //         2 => Some(ParseTreeRef::E(&c2)),
            //         _ => unreachable!()
            // }
            let child_name = format_ident!("c{}", i);
            quote! {
                #i_lit => Some(ParseTreeRef::#name_ident(#child_name))
            }
        }
    });
    if single_rule {
        quote! {
            #(#cases),*,
            _ => unreachable!(),
        }
    } else {
        quote! {
            match index {
                #(#cases),*,
                _ => unreachable!(),
            }
        }
    }
}

fn gen_as_parse_tree_ref(nonterminal_name: &str) -> TokenStream {
    let name_ident = Ident::new(&to_first_uppercase(nonterminal_name), Span::call_site());
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
        }
    }
}

fn gen_token_kind_enum(terminals: &[(Ident, String)]) -> TokenStream {
    let terminal_ids: Vec<_> = terminals.iter().map(|(id, name)|{
        quote! {
            #[comment = #name]
            #id
        }
    }).collect();
    quote! {
        #[derive(Debug)]
        enum TokenKind {
            #(#terminal_ids),*
        }
    }
}

fn gen_token_kind_impl(terminals: &[(Ident, String)]) -> TokenStream {
    let terminal_ids: Vec<_> = terminals.iter().map(|(id, name)|{
        quote! {
            TokenKind::#id => #name
        }
    }).collect();
    quote! {
        impl TokenKind {
            pub fn name(&self) -> &'static str {
                match self {
                    #(#terminal_ids),*
                }
            }
        }
    }
}

fn gen_token_kind_function(terminals: &[(Ident, String)]) -> TokenStream {
    let cases: Vec<TokenStream> = terminals
        .iter()
        .map(|(id, name)| {
            quote! { 
                #[comment = #name]
                #id => TokenKind::#id 
            }
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
                    let alternative = &alternatives.unwrap()[index];
                    let end_slot_id = end_slot.slot_id;
                    let slot_name = slot_ids.slot_name(&end_slot.slot_id);
                    let num_symbols = alternative.len();
                    let children_names = children_names(alternative);
                    let methods: Vec<_> = alternative
                        .symbols
                        .iter()
                        .map(|s| match s {
                            Symbol::Terminal(_) => {
                                (Ident::new("unwrap_token", Span::call_site()), false)
                            }
                            Symbol::Nonterminal(n) => { 
                                let ident = format_ident!("unwrap_{}", to_first_lowercase(&n.name));
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
    let unwrap_methods: Vec<_> = grammar
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
    let as_parse_tree_ref_method = gen_as_parse_tree_ref_method(grammar);
    quote! {
        impl ParseTree {
            #(#unwrap_methods)*
            fn unwrap_token(self) -> Token {
                match self {
                    ParseTree::Token(t) => t,
                    _ => panic!(),
                }
            }
            #as_parse_tree_ref_method
        }
    }
}

fn gen_as_parse_tree_ref_method(grammar: &Grammar) -> TokenStream {
    let cases: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let variant = Ident::new(&to_first_uppercase(&n.name), Span::call_site());
            let var_ident = Ident::new(&to_first_lowercase(&n.name), Span::call_site());
            quote! {
                ParseTree::#variant(#var_ident) => #var_ident.as_parse_tree_ref()
            }
        })
        .collect();
    quote! {
        fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
            match self {
                #(#cases),*,
                _ => unreachable!()
            }
        }
    }
}

fn gen_parse_tree_as_ref_enum(grammar: &Grammar) -> TokenStream {
    let variants: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let ident = Ident::new(&n.name, Span::call_site());
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

fn gen_parse_tree_ref_impl() -> TokenStream {
    quote! {
        impl<'a> ParseTreeRef<'a> {
            pub fn children(&self) -> ChildIter<'a> {
                ChildIter {
                    node: *self,
                    index: 0,
                }
            }
        }
    }
}

fn gen_child_iter_struct() -> TokenStream {
    quote! {
        pub struct ChildIter<'a> {
            node: ParseTreeRef<'a>,
            index: usize,
        }
    }
}

fn gen_impl_iterator_for_child_iter(grammar: &Grammar) -> TokenStream {
    let cases: Vec<_> = grammar
        .nonterminals()
        .map(|n| {
            let variant = Ident::new(&to_first_uppercase(&n.name), Span::call_site());
            let var_ident = Ident::new(&to_first_lowercase(&n.name), Span::call_site());
            quote! { ParseTreeRef::#variant(#var_ident) => #var_ident.child(self.index) }
        })
        .collect();
    quote! {
        impl<'a> Iterator for ChildIter<'a> {
            type Item = ParseTreeRef<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                let child = match self.node {
                    #(#cases),*,
                    ParseTreeRef::Token(_) => None,
                };
                self.index += 1;
                child
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
