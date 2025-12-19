use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    generator::{id::{NonterminalIds, SlotIds, TerminalIds}, utils::{alternative_label, to_first_lowercase, to_first_uppercase}},
    grammar::{grammar::{Alternative, Grammar}, symbols::{Nonterminal, Symbol}}, ids::TerminalId,
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

    let to_sexpr_function = gen_to_sexpr_function();
    let node_to_sexpr_function = gen_node_to_sexpr_function();

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
        #token_impl
        #token_kind_function
        #parse_tree_builder_impl
        #create_parse_tree_method
        #to_sexpr_function
        #node_to_sexpr_function
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
            sppf::{NonterminalNode, SPPFNodeId},
        };
        use crate::parser::#parser_name;
    }
}

fn gen_nonterminal_type(
    nonterminal: &Nonterminal,
    alternatives: &[Alternative],
) -> TokenStream {
    let nonterminal_name_id = Ident::new(&nonterminal.name, Span::call_site());
    if alternatives.is_empty() {
        todo!("handle empty alternatives")
    } else if alternatives.len() == 1 {
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
}

fn gen_nonterminal_type_impl(
    grammar: &Grammar,
    nonterminal: &Nonterminal,
) -> TokenStream {
    let nonterminal_name = Ident::new(&nonterminal.name, Span::call_site());
    let child_method = gen_child_method(grammar, nonterminal);
    let child_count_method = gen_child_count_method(grammar, nonterminal);
    let as_node_ref_method = gen_as_parse_tree_ref_method(&nonterminal.name);
    quote! {
        impl #nonterminal_name {
            #child_method
            #child_count_method
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
    let alternatives = grammar.alternatives(nonterminal);
    if alternatives.is_empty() {
        todo!("handle empty alternatives")
    } else if alternatives.len() == 1 {
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
                let alt_variant = Ident::new(&to_first_uppercase(&label), Span::call_site());
                let children_names = children_names(alternative);
                let body = child_by_index(alternative, false);
                quote! {
                    #ident::#alt_variant(#(#children_names),*) => #body
                }
            }).collect();
            quote! {
                match self {
                    #(#arms),*
                }
        }
    }
}

fn child_by_index(alternative: &Alternative, single_rule: bool) -> TokenStream {
    let cases = (0..alternative.symbols.len()).map(|i| {
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
    });
    if single_rule {
        quote! {
            #(#cases),*,
            _ => None,
        }
    } else {
        quote! {
            match index {
                #(#cases),*,
                _ => None,
            }
        }
    }
}

fn gen_child_count_method(grammar: &Grammar, nonterminal: &Nonterminal) -> TokenStream {
    let ident = Ident::new(&to_first_uppercase(&nonterminal.name), Span::call_site());
    let alternatives = grammar.alternatives(nonterminal);
    let body = if alternatives.is_empty() {
        todo!("handle empty alternatives")
    }
    else if alternatives.len() == 1 {
        let count_symbols = alternatives[0].symbols.len();
        quote! {
            #count_symbols
        }
    } else {
        let arms: Vec<_> = alternatives.iter().enumerate().map(|(i, alternative)| {
            let label = alternative_label(alternative, i);
            let alt_variant = Ident::new(&to_first_uppercase(&label), Span::call_site());
            let count_symbols = alternative.symbols.len();
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

fn gen_token_impl() -> TokenStream {
    quote! {
        impl Token {
            pub fn as_parse_tree_ref(&self) -> ParseTreeRef<'_> {
                ParseTreeRef::Token(self)
            }
        }
    }
}

fn gen_token_kind_enum(terminals: &[(TerminalId, String)]) -> TokenStream {
    let terminal_ids: Vec<_> = terminals.iter().map(|(id, name)|{
        let ident = format_ident!("T{}", id.0);
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
                    #(#terminal_ids),*
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
                    let alternative = &alternatives[index];
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
                        .len();
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
    let unwrap_methods = gen_unwrap_methods(grammar);
    quote! {
        impl ParseTree {
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

fn gen_unwrap_methods(grammar: &Grammar) -> Vec<TokenStream> {
    grammar
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
        .collect()
}

fn gen_parse_tree_ref_enum(grammar: &Grammar) -> TokenStream {
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

fn gen_parse_tree_ref_impl(grammar: &Grammar) -> TokenStream {
    let name_method = gen_name_method(grammar);
    let children_method = gen_children_method();
    let child_count_method = gen_child_count_method_for_parse_tree_ref(grammar);
    quote! {
        impl<'a> ParseTreeRef<'a> {
            #children_method
            #name_method
            #child_count_method
        }
    }
}

fn gen_children_method() -> TokenStream {
    quote! {
        pub fn children(&self) -> ChildIter<'a> {
            ChildIter {
                node: *self,
                index: 0,
            }
        }
    }
}

fn gen_name_method(grammar: &Grammar) -> TokenStream {
    let arms = grammar.nonterminals().map(|n| {
        let name = &n.name;
        let name_ident = Ident::new(name, Span::call_site());
        quote! { ParseTreeRef::#name_ident(_) => #name }
    });
    quote! {
        pub fn name(&self) -> &'static str {
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
            let variant = Ident::new(&to_first_uppercase(&n.name), Span::call_site());
            let var_ident = Ident::new(&to_first_lowercase(&n.name), Span::call_site());
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

            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining = self.node.child_count().saturating_sub(self.index);
                (remaining, Some(remaining))
            }
        }

        impl<'a> ExactSizeIterator for ChildIter<'a> {}
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
            let children: Vec<_> = node.children().collect();
            if children.is_empty() {
                writeln!(w, "{:indent$}{}", "", node.name())
            } else {
                writeln!(w, "{:indent$}({}", "", node.name())?;
                for child in children {
                    node_to_sexpr(child, indent + 2, w)?;
                }
                writeln!(w, "{:indent$})", "")
            }
        }
    }
}
