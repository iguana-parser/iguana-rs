use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use rustc_hash::FxHashMap;

use crate::{
    generator::{
        GenConfig, grammar_utils,
        grammar_utils::{nonterminal_type_name, nt_ident},
        id::{NonterminalIds, SlotIds, TerminalIds},
        utils::{alternative_label, is_valid_rust_ident, safe_ident},
    },
    grammar::{
        def::{Alternative, Grammar},
        symbols::{Definition, DefinitionId, Identifier, Nonterminal, Symbol},
    },
    ids::TerminalId,
    utils::{to_first_uppercase, to_pascal_case, to_snake_case},
};

pub struct ParseTreeGen<'a> {
    grammar: &'a Grammar,
    nonterminal_ids: &'a NonterminalIds,
    terminal_ids: &'a TerminalIds,
    slot_ids: &'a SlotIds<'a>,
    config: GenConfig,
}

impl<'a> ParseTreeGen<'a> {
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
            config,
        }
    }

    /// True when the generated enum for a nonterminal takes a `'a` parameter;
    /// see [`grammar_utils::nonterminal_has_lifetime`].
    fn has_lifetime(&self, nonterminal: &Nonterminal) -> bool {
        grammar_utils::nonterminal_has_lifetime(self.grammar, nonterminal, self.config.unsafe_mode)
    }

    /// The parse tree type for a nonterminal, e.g. `Ident<'a>` or plain
    /// `Ident`; see [`grammar_utils::nonterminal_type`].
    fn nonterminal_type(&self, nonterminal: &Nonterminal) -> TokenStream {
        grammar_utils::nonterminal_type(self.grammar, nonterminal, self.config.unsafe_mode)
    }

    /// The type of a symbol as it appears in parse tree fields; see
    /// [`grammar_utils::symbol_type`].
    fn symbol_type(&self, def_id: DefinitionId) -> TokenStream {
        grammar_utils::symbol_type(self.grammar, def_id, self.config.unsafe_mode)
    }

    pub fn generate(&self) -> TokenStream {
        let terminals: Vec<(TerminalId, String)> = self
            .terminal_ids
            .ids()
            .zip(self.terminal_ids.terminals())
            .map(|(id, t)| (id, t.to_string()))
            .collect();
        let imports = self.gen_imports();
        let token_kind_enum = gen_token_kind_enum(&terminals);
        let token_kind_impl = gen_token_kind_impl(&terminals);
        let token_kind_function = gen_token_kind_function(&terminals);
        let token_struct = gen_token_struct();
        let token_impl = gen_token_impl();
        let parse_tree_enum = self.gen_parse_tree_enum();
        let parse_tree_impl = self.gen_parse_tree_impl();
        let list_node_trait = gen_list_node_trait();
        let list_node_impls_for_plus: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| n.is_plus())
            .map(|n| self.gen_list_node_impl_for_plus(n))
            .collect();
        let list_node_impls_for_star: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| n.is_star())
            .map(|n| self.gen_list_node_impl_for_star(n))
            .collect();
        let list_node_impls_for_group: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| n.is_group())
            .map(|n| self.gen_list_node_impl_for_group(n))
            .collect();
        let opt_node_trait = gen_opt_node_trait();
        let opt_node_impls: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| matches!(&n.origin, Some(Symbol::Opt(_))))
            .map(|n| self.gen_opt_node_impl(n))
            .collect();
        let alt_accessor_impls: Vec<TokenStream> = self
            .grammar
            .nonterminals()
            .filter(|n| is_single_symbol_alternation(self.grammar, n))
            .map(|n| self.gen_alt_accessors(n))
            .collect();
        let parse_tree_builder_impl = self.gen_parse_tree_builder_impl();
        let create_parse_tree_function = self.gen_create_parse_tree_function();
        let create_parse_tree_functions: Vec<_> = self
            .grammar
            .nonterminals()
            .map(|n| self.gen_create_parse_tree_nonterminal_function(n))
            .collect();

        let nonterminal_types: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| self.gen_nonterminal_type(n))
            .collect();

        let nonterminal_types_impl: Vec<_> = self
            .grammar
            .nonterminals()
            .map(|n| {
                if self.grammar.is_start(n) {
                    self.gen_start_type_impl(n)
                } else {
                    self.gen_nonterminal_type_impl(n)
                }
            })
            .collect();

        let has_start = self
            .grammar
            .nonterminals()
            .any(|n| self.grammar.is_start(n));
        let start_struct = if has_start {
            quote! {
                #[derive(Debug)]
                pub struct Start<T, L> {
                    pub before: L,
                    pub node: T,
                    pub after: L,
                    pub span: Span,
                }
            }
        } else {
            quote! {}
        };

        let parse_tree_node_impl = self.gen_parse_tree_node_impl();
        let to_sexpr_function = self.gen_to_sexpr_function();
        let to_json_function = gen_to_json_function();

        quote! {
            #imports
            #token_kind_enum
            #token_kind_impl
            #start_struct
            #parse_tree_enum
            #parse_tree_impl
            #list_node_trait
            #opt_node_trait
            #(#nonterminal_types)*
            #(#nonterminal_types_impl)*
            #(#list_node_impls_for_plus)*
            #(#list_node_impls_for_star)*
            #(#list_node_impls_for_group)*
            #(#opt_node_impls)*
            #(#alt_accessor_impls)*
            #token_struct
            #token_impl
            #token_kind_function
            #parse_tree_builder_impl
            #create_parse_tree_function
            #(#create_parse_tree_functions)*
            #parse_tree_node_impl
            #to_sexpr_function
            #to_json_function
        }
    }

    fn gen_imports(&self) -> TokenStream {
        let parser_name = format_ident!("{}Parser", to_first_uppercase(&self.grammar.name));
        quote! {
            use std::vec::IntoIter;
            use iguana_runtime::{
                ids::{NonterminalId, SlotId, TerminalId},
                input::Span,
                parse_tree::{
                    Bump, NodeKind, OneOrMany, Origin, ParseTreeBuilder, ParseTreeNode,
                    SexprOptions, visit_sppf,
                },
                sppf::{NonterminalNode, SPPFNodeId, TerminalNode},
            };
            use crate::parser::#parser_name;
        }
    }

    /// The generics on a nonterminal's enum declaration and inherent impl:
    /// `<'a>`, or nothing for a token-only nonterminal in the unsafe mode.
    fn gen_enum_generics(&self, nonterminal: &Nonterminal) -> TokenStream {
        if self.has_lifetime(nonterminal) {
            quote! { <'a> }
        } else {
            quote! {}
        }
    }

    fn gen_nonterminal_type(&self, nonterminal: &Nonterminal) -> TokenStream {
        let alternatives = self.grammar.alternatives(nonterminal);
        if alternatives.len() == 1 {
            self.gen_nonterminal_type_with_one_alternative(nonterminal, &alternatives[0])
        } else {
            self.gen_nonterminal_type_with_more_than_one_alternative(nonterminal, alternatives)
        }
    }

    /// Returns the Rust type for a parse tree field: `Token` for terminals,
    /// `&'a Type<'a>` (or `&'a Type`) for nonterminals.
    fn gen_field_type(&self, symbol: &Symbol) -> TokenStream {
        self.symbol_type(symbol.resolved_def())
    }

    /// Returns (name, type) pairs for each parse-tree-relevant symbol in an alternative.
    /// Literals are filtered out. For example, given `Expr = Expr "+" Expr`, this returns:
    ///   `[(expr_0, &'a Expr<'a>), (expr_1, &'a Expr<'a>)]`
    /// The `"+"` is skipped.
    fn gen_fields_for_alternative_symbols(
        &self,
        alternative: &Alternative,
    ) -> Vec<(Ident, TokenStream)> {
        let counts = count_symbol_occurrences(self.grammar, &alternative.symbols);
        alternative
            .symbols
            .iter()
            .filter(|s| s.is_parse_tree_symbol())
            .enumerate()
            .map(|(i, s)| {
                let base_name = get_symbol_base_name(self.grammar, s);
                let needs_index =
                    base_name.is_some_and(|name| counts.get(&name).copied().unwrap_or(0) > 1);
                let field_name = gen_field_name(self.grammar, s, i, needs_index);
                let field_ident = safe_ident(&field_name);
                let field_type = self.gen_field_type(s);
                (field_ident, field_type)
            })
            .collect()
    }

    fn gen_nonterminal_type_with_one_alternative(
        &self,
        nonterminal: &Nonterminal,
        alternative: &Alternative,
    ) -> TokenStream {
        let fields: Vec<_> = self
            .gen_fields_for_alternative_symbols(alternative)
            .into_iter()
            .map(|(ident, ty)| quote! { #ident: #ty })
            .collect();
        let nonterminal_name = &nonterminal.name;
        let comment = if nonterminal.is_derived() {
            let display_name = nonterminal.display_name();
            quote! { #[comment = #display_name] }
        } else {
            let rule = format!(
                "{} = {}",
                nonterminal_name,
                alternative.display_name(self.grammar)
            );
            quote! { #[comment = #rule] }
        };
        let nonterminal_name_id = nt_ident(nonterminal_name);
        let alt0_variant = nt_ident(&alternative_label(alternative, 0));
        // Single-alternative nonterminals are enums with `<Alt0> { fields, span }`
        // + `Amb(&[&Self])`, mirroring multi-alternative nonterminals. An
        // intermediate-node ambiguity inside the rule body produces multiple
        // derivations of the same span; each materializes into its own `<Alt0>`
        // value, and the alternatives land in `Amb`. The `<Alt0>` variant takes
        // the alternative's label when present, falling back to `Alt0`.
        // The unsafe mode produces no ambiguity, so it drops the `Amb` variant;
        // the type stays a one-variant enum so the rest of the codegen is
        // uniform across modes.
        let amb_variant = if self.config.unsafe_mode {
            quote! {}
        } else {
            quote! { Amb(&'a [&'a #nonterminal_name_id<'a>]), }
        };
        let generics = self.gen_enum_generics(nonterminal);
        quote! {
            #comment
            #[derive(Debug)]
            pub enum #nonterminal_name_id #generics {
                #alt0_variant { #(#fields,)* span: Span },
                #amb_variant
            }
        }
    }

    fn gen_nonterminal_type_with_more_than_one_alternative(
        &self,
        nonterminal: &Nonterminal,
        alternatives: &[Alternative],
    ) -> TokenStream {
        let arms: Vec<_> = alternatives
            .iter()
            .enumerate()
            .map(|(index, alternative)| {
                let fields: Vec<_> = self
                    .gen_fields_for_alternative_symbols(alternative)
                    .into_iter()
                    .map(|(ident, ty)| quote! { #ident: #ty })
                    .collect();
                let label = to_pascal_case(&alternative_label(alternative, index));
                let variant_name = Ident::new(&label, Span::call_site());
                let params = if nonterminal.parameters.is_empty() {
                    String::new()
                } else {
                    let names: Vec<_> = nonterminal
                        .parameters
                        .iter()
                        .map(|p| p.name.clone())
                        .collect();
                    format!("({})", names.join(", "))
                };
                let variant_comment = format!(
                    "{}{} = {}",
                    nonterminal.name,
                    params,
                    alternative.display_name(self.grammar)
                );
                quote! {
                    #[comment = #variant_comment]
                    #variant_name { #(#fields,)* span: Span }
                }
            })
            .collect();
        let nonterminal_name = &nonterminal.name;
        let comment = if nonterminal.is_derived() {
            let display_name = nonterminal.display_name();
            quote! { #[comment = #display_name] }
        } else {
            quote! {}
        };
        let nonterminal_name_id = nt_ident(nonterminal_name);
        let amb_variant = if self.config.unsafe_mode {
            quote! {}
        } else {
            quote! { Amb(&'a [&'a #nonterminal_name_id<'a>]) }
        };
        let generics = self.gen_enum_generics(nonterminal);
        quote! {
            #comment
            #[derive(Debug)]
            pub enum #nonterminal_name_id #generics {
                #(#arms,)*
                #amb_variant
            }
        }
    }

    fn gen_nonterminal_type_impl(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ty = self.nonterminal_type(nonterminal);
        let generics = self.gen_enum_generics(nonterminal);
        let as_parse_tree_method = self.gen_as_parse_tree_method(nonterminal);
        let child_method = self.gen_child_method(nonterminal);
        let child_count_method = self.gen_child_count_method(nonterminal);
        let span_method = self.gen_span_method(nonterminal);
        let display_name_method = self.gen_nonterminal_display_name_method(nonterminal);
        let origin_method = self.gen_nonterminal_origin_method(nonterminal);
        let typed_accessor = self.gen_typed_accessor(nonterminal);
        let field_accessors = self.gen_field_accessor_methods(nonterminal);
        quote! {
            impl #generics #ty {
                #as_parse_tree_method
                #child_method
                #child_count_method
                #span_method
                #display_name_method
                #origin_method
                #typed_accessor
                #field_accessors
            }
        }
    }

    /// Generates per-field accessor methods on a single-alternative
    /// nonterminal enum (`<Alt0> { fields, span } + Amb`) so callers can keep
    /// using `node.field` syntax (as `node.field()`) without matching on the
    /// enum. Accessors panic on the `Amb` variant, symmetric with the
    /// field-specific methods on multi-alternative nonterminals, which also
    /// panic when called on a non-matching variant. The unsafe mode has no
    /// `Amb` variant, so its accessors match the single variant alone.
    ///
    /// Derived single-alternative nonterminals (Group, Star) also get these
    /// accessors. For example, with `("\\" Identifier)+` the Plus's typed
    /// accessor `identifiers()` walks each Group child `r` and calls
    /// `r.identifier()` to read its `identifier` field (skipping the `"\\"`
    /// literal). That method is the field accessor generated here.
    fn gen_field_accessor_methods(&self, nonterminal: &Nonterminal) -> TokenStream {
        let alternatives = self.grammar.alternatives(nonterminal);
        if alternatives.len() != 1 {
            return quote! {};
        }
        let ident = nt_ident(&nonterminal.name);
        let nt_name = &nonterminal.name;
        let alternative = &alternatives[0];
        let alt0_label = alternative_label(alternative, 0);
        let alt0_variant = nt_ident(&alt0_label);
        let counts = count_symbol_occurrences(self.grammar, &alternative.symbols);
        let methods: Vec<_> = alternative
            .symbols
            .iter()
            .filter(|s| s.is_parse_tree_symbol())
            .enumerate()
            .map(|(i, s)| {
                let base_name = get_symbol_base_name(self.grammar, s);
                let needs_index =
                    base_name.is_some_and(|name| counts.get(&name).copied().unwrap_or(0) > 1);
                let field_ident = safe_ident(&gen_field_name(self.grammar, s, i, needs_index));
                let field_ty = self.gen_field_type(s);
                let is_terminal = matches!(
                    self.grammar.definition(s.resolved_def()),
                    Definition::Terminal(_)
                );
                // Terminal fields are `Token` (Copy value): the match binding is
                // `&Token`, so we deref to return the value. Nonterminal fields
                // are `&'a T<'a>` (a Copy reference): the binding is `&&'a T<'a>`,
                // and returning it directly relies on the `&&T` to `&T`
                // coercion. Clippy's `explicit_auto_deref` lint warns on an
                // explicit `*` in that case.
                let body = if is_terminal {
                    quote! { *#field_ident }
                } else {
                    quote! { #field_ident }
                };
                let amb_arm = if self.config.unsafe_mode {
                    quote! {}
                } else {
                    let panic_msg = format!("{} is ambiguous", nt_name);
                    quote! { #ident::Amb(_) => panic!(#panic_msg), }
                };
                quote! {
                    pub fn #field_ident(&self) -> #field_ty {
                        match self {
                            #ident::#alt0_variant { #field_ident, .. } => #body,
                            #amb_arm
                        }
                    }
                }
            })
            .collect();
        quote! { #(#methods)* }
    }

    /// Generates `display_name` on a nonterminal type. Returns `"Amb"` for the
    /// `Amb` variant and the nonterminal's name otherwise. The dispatch shape
    /// is uniform across single- and multi-alternative rules, since every
    /// nonterminal is an enum. The unsafe mode has no `Amb` variant, so its
    /// method returns the name with no dispatch.
    fn gen_nonterminal_display_name_method(&self, nonterminal: &Nonterminal) -> TokenStream {
        let display_name = nonterminal.display_name();
        let ident = nt_ident(&nonterminal.name);
        if self.config.unsafe_mode {
            return quote! {
                pub fn display_name(&self) -> &'static str {
                    #display_name
                }
            };
        }
        quote! {
            pub fn display_name(&self) -> &'static str {
                match self {
                    #ident::Amb(_) => "Amb",
                    _ => #display_name,
                }
            }
        }
    }

    /// Generates `origin` on a nonterminal type. Returns `None` for the `Amb`
    /// variant and the nonterminal's origin otherwise. Reporting `None` for `Amb`
    /// keeps the presentation transforms from splicing or dropping an ambiguity
    /// cluster. A user-declared nonterminal has no origin, so its method is a plain
    /// `None` with no dispatch, and the unsafe mode, which has no `Amb` variant,
    /// returns the origin the same way.
    fn gen_nonterminal_origin_method(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ident = nt_ident(&nonterminal.name);
        match origin_kind(nonterminal) {
            Some(kind) if self.config.unsafe_mode => quote! {
                pub fn origin(&self) -> Option<Origin> {
                    Some(#kind)
                }
            },
            Some(kind) => quote! {
                pub fn origin(&self) -> Option<Origin> {
                    match self {
                        #ident::Amb(_) => None,
                        _ => Some(#kind),
                    }
                }
            },
            None => quote! {
                pub fn origin(&self) -> Option<Origin> {
                    None
                }
            },
        }
    }

    fn gen_start_type_impl(&self, nonterminal: &Nonterminal) -> TokenStream {
        let start_ty = self.nonterminal_type(nonterminal);
        let inner_ident = nonterminal
            .origin
            .as_ref()
            .unwrap()
            .as_identifier()
            .unwrap();
        let layout_ident = self
            .grammar
            .layout
            .as_ref()
            .unwrap()
            .as_identifier()
            .unwrap();

        let inner_variant = nt_ident(&nonterminal_type_name(&inner_ident.name));
        let inner_child = quote! { ParseTree::#inner_variant(self.node) };
        let (layout_before, layout_after) = if self.grammar.is_terminal(layout_ident) {
            (
                quote! { ParseTree::Token(self.before) },
                quote! { ParseTree::Token(self.after) },
            )
        } else {
            let variant = nt_ident(&nonterminal_type_name(&layout_ident.name));
            (
                quote! { ParseTree::#variant(self.before) },
                quote! { ParseTree::#variant(self.after) },
            )
        };

        let start_variant = nt_ident(&nonterminal.name);
        quote! {
            impl<'a> #start_ty {
                pub fn as_parse_tree(&'a self) -> ParseTree<'a> {
                    ParseTree::#start_variant(self)
                }
                pub fn child(&self, index: usize) -> Option<ParseTree<'a>> {
                    match index {
                        0 => Some(#layout_before),
                        1 => Some(#inner_child),
                        2 => Some(#layout_after),
                        _ => None,
                    }
                }
                pub fn child_count(&self) -> usize {
                    3usize
                }
                pub fn span(&self) -> Span {
                    self.span
                }
                pub fn display_name(&self) -> &'static str {
                    "Start"
                }
                pub fn origin(&self) -> Option<Origin> {
                    Some(Origin::Start)
                }
            }
        }
    }

    fn gen_span_method(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ident = nt_ident(&nonterminal.name);
        let alternatives = self.grammar.alternatives(nonterminal);
        let arms: Vec<_> = alternatives
            .iter()
            .enumerate()
            .map(|(i, alternative)| {
                let label = alternative_label(alternative, i);
                let alt_variant = nt_ident(&label);
                quote! {
                    #ident::#alt_variant { span, .. } => *span
                }
            })
            .collect();
        let amb_arm = if self.config.unsafe_mode {
            quote! {}
        } else {
            quote! { #ident::Amb(alts) => alts[0].span(), }
        };
        quote! {
            pub fn span(&self) -> Span {
                match self {
                    #(#arms,)*
                    #amb_arm
                }
            }
        }
    }

    fn gen_child_method(&self, nonterminal: &Nonterminal) -> TokenStream {
        let children_by_index = self.gen_children_by_index(nonterminal);
        // A lifetime-free enum has no `'a` in scope from the impl header, so
        // the method declares `'a` itself. Its children are all `Token` (Copy
        // values), so any caller-chosen `'a` fits the returned `ParseTree<'a>`.
        let method_generics = if self.has_lifetime(nonterminal) {
            quote! {}
        } else {
            quote! { <'a> }
        };
        // When every alternative is field-free, every arm is `Alt { .. } => None`
        // and nothing reads `index` except the `Amb` arm. The unsafe mode has
        // no `Amb` arm, so it underscores the parameter to keep the generated
        // crate warning-free.
        let no_alternative_has_fields = self
            .grammar
            .alternatives(nonterminal)
            .iter()
            .all(|alt| !alt.symbols.iter().any(|s| s.is_parse_tree_symbol()));
        let index_param = if self.config.unsafe_mode && no_alternative_has_fields {
            quote! { _index }
        } else {
            quote! { index }
        };
        quote! {
            pub fn child #method_generics (&self, #index_param: usize) -> Option<ParseTree<'a>> {
                #children_by_index
            }
        }
    }

    fn gen_children_by_index(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ident = nt_ident(&nonterminal.name);
        let alternatives = self.grammar.alternatives(nonterminal);
        let arms: Vec<_> = alternatives
            .iter()
            .enumerate()
            .map(|(i, alternative)| {
                let label = alternative_label(alternative, i);
                let alt_variant = nt_ident(&label);
                let field_names = field_names(self.grammar, alternative);
                let body = child_by_index(self.grammar, alternative);
                quote! {
                    #ident::#alt_variant { #(#field_names,)* .. } => #body
                }
            })
            .collect();
        let amb_arm = if self.config.unsafe_mode {
            quote! {}
        } else {
            quote! { #ident::Amb(alts) => alts.get(index).copied().map(ParseTree::#ident), }
        };
        quote! {
            match self {
                #(#arms,)*
                #amb_arm
            }
        }
    }

    fn gen_child_count_method(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ident = nt_ident(&nonterminal.name);
        let alternatives = self.grammar.alternatives(nonterminal);
        let arms: Vec<_> = alternatives
            .iter()
            .enumerate()
            .map(|(i, alternative)| {
                let label = alternative_label(alternative, i);
                let alt_variant = nt_ident(&label);
                let count_symbols = alternative
                    .symbols
                    .iter()
                    .filter(|s| s.is_parse_tree_symbol())
                    .count();
                quote! {
                    #ident::#alt_variant { .. } => #count_symbols
                }
            })
            .collect();
        let amb_arm = if self.config.unsafe_mode {
            quote! {}
        } else {
            quote! { #ident::Amb(alts) => alts.len(), }
        };
        quote! {
            pub fn child_count(&self) -> usize {
                match self {
                    #(#arms,)*
                    #amb_arm
                }
            }
        }
    }

    fn gen_as_parse_tree_method(&self, nonterminal: &Nonterminal) -> TokenStream {
        let name_ident = nt_ident(&nonterminal.name);
        // A lifetime-free enum declares `'a` on the method instead of the impl.
        let method_generics = if self.has_lifetime(nonterminal) {
            quote! {}
        } else {
            quote! { <'a> }
        };
        quote! {
            pub fn as_parse_tree #method_generics (&'a self) -> ParseTree<'a> {
                ParseTree::#name_ident(self)
            }
        }
    }

    fn gen_parse_tree_builder_impl(&self) -> TokenStream {
        let builder_name_ident = format_ident!("{}ParseTreeBuilder", self.grammar.name);
        let nonterminal_node_method = self.gen_nonterminal_node_method();
        let new_token_method = gen_new_token_method();
        let new_ambiguity_node_method = self.gen_new_ambiguity_node_method();
        quote! {
            pub struct #builder_name_ident<'a> {
                pub arena: &'a Bump,
            }
            impl<'a> #builder_name_ident<'a> {
                pub fn new(tree_arena: &'a Bump) -> Self {
                    Self { arena: tree_arena }
                }
            }
            impl<'a> ParseTreeBuilder<ParseTree<'a>> for #builder_name_ident<'a> {
                #nonterminal_node_method
                #new_token_method
                #new_ambiguity_node_method
            }
        }
    }

    /// Generates a per-grammar `new_ambiguity_node` that dispatches on the parent
    /// nonterminal id. Every generated nonterminal type is an enum with an `Amb`
    /// variant, so every nonterminal can be the parent of an intermediate-node
    /// ambiguity. The start nonterminal is excluded: its type is the hardcoded
    /// `Start<T, L>` wrapper, which has no `Amb` variant; ambiguity surfaces
    /// inside the inner nonterminal instead.
    ///
    /// The unsafe mode emits no override: tree construction never reaches the
    /// ambiguity path there, so the trait's `unimplemented!` default covers the
    /// signature.
    fn gen_new_ambiguity_node_method(&self) -> TokenStream {
        if self.config.unsafe_mode {
            return quote! {};
        }
        let cases: Vec<TokenStream> = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| {
                let variant = nt_ident(&n.name);
                let const_name = format_ident!("{}", to_snake_case(&n.name).to_uppercase());
                let unwrap_method = format_ident!("unwrap_{}", to_snake_case(&n.name));
                quote! {
                    crate::grammar_data::#const_name => {
                        let slice = self.arena.alloc_slice_fill_iter(
                            alternatives.into_iter().map(|a| a.#unwrap_method())
                        );
                        ParseTree::#variant(self.arena.alloc(#variant::Amb(slice)))
                    }
                }
            })
            .collect();
        if cases.is_empty() {
            return quote! {};
        }
        quote! {
            fn new_ambiguity_node(
                &self,
                parent: NonterminalId,
                alternatives: Vec<ParseTree<'a>>,
            ) -> ParseTree<'a> {
                match parent {
                    #(#cases)*
                    _ => unreachable!("nonterminal cannot be ambiguous"),
                }
            }
        }
    }

    fn gen_nonterminal_node_method(&self) -> TokenStream {
        let nonterminal_cases: Vec<TokenStream> = self
            .nonterminal_ids
            .ids()
            .map(|nonterminal_id| {
                let nonterminal = self.nonterminal_ids.get_nonterminal(nonterminal_id);
                let slot_cases: Vec<TokenStream> = self
                    .nonterminal_ids
                    .end_slots(nonterminal_id)
                    .map(|end_slot| {
                        let index = end_slot.index;
                        let alternatives = self.grammar.alternatives(nonterminal);
                        let alternative = &alternatives[index];
                        let end_slot_id = end_slot.slot_id;
                        let slot_name = self.slot_ids.display_name(&end_slot.slot_id);
                        let num_symbols = alternative.symbols.iter().filter(|s| s.is_parse_tree_symbol()).count();
                        let field_names = field_names(self.grammar, alternative);
                        let method_calls: Vec<_> = alternative
                            .symbols
                            .iter()
                            .filter(|s| s.is_parse_tree_symbol())
                            .map(|s| {
                                let def_id = s.resolved_def();
                                let def = self.grammar.definition(def_id);
                                match def {
                                    Definition::Terminal(_) =>
                                        Ident::new("unwrap_token", Span::call_site()),
                                    Definition::Nonterminal(nt) => {
                                        format_ident!("unwrap_{}", to_snake_case(&nt.name))
                                    }
                                }
                            })
                            .zip(field_names.iter().cloned())
                            .map(|(method, child)| quote! { #child.#method() })
                            .collect();
                        let nonterminal_type = nt_ident(&nonterminal.name);
                        let parse_tree_variant = nt_ident(&nonterminal.name);
                        let construction = if self.grammar.is_start(nonterminal) {
                            let before = &method_calls[0];
                            let node = &method_calls[1];
                            let after = &method_calls[2];
                            quote! {
                                ParseTree::#parse_tree_variant(self.arena.alloc(Start {
                                    before: #before,
                                    node: #node,
                                    after: #after,
                                    span: nonterminal_node.span,
                                }))
                            }
                        } else {
                            let variant = Ident::new(
                                &to_pascal_case(&alternative_label(alternative, index)),
                                Span::call_site()
                            );
                            quote! {
                                ParseTree::#parse_tree_variant(self.arena.alloc(#nonterminal_type::#variant {
                                    #(#field_names: #method_calls,)*
                                    span: nonterminal_node.span,
                                }))
                            }
                        };
                        quote! {
                            #[comment = #slot_name]
                            #end_slot_id => {
                                let [#(#field_names),*] = children.into_array::<#num_symbols>();
                                #construction
                            }
                        }
                    })
                    .collect();
                let nonterminal_name = &nonterminal.name;
                quote! {
                    #[comment = #nonterminal_name]
                    #nonterminal_id => match nonterminal_node.return_slot {
                        #(#slot_cases,)*
                        _ => unreachable!()
                    }
                }
            })
            .collect();
        quote! {
            fn new_nonterminal_node(
                &self,
                nonterminal_node: &NonterminalNode,
                children: OneOrMany<ParseTree<'a>>
            ) -> ParseTree<'a> {
                match nonterminal_node.nonterminal_id {
                    #(#nonterminal_cases),*
                    _ => unreachable!()
                }
            }
        }
    }

    fn gen_parse_tree_enum(&self) -> TokenStream {
        let arms: Vec<_> = self
            .grammar
            .nonterminals()
            .map(|n| {
                let variant = nt_ident(&n.name);
                let ty = self.nonterminal_type(n);
                if n.is_derived() {
                    let display_name = n.display_name();
                    quote! {
                        #[comment = #display_name]
                        #variant(&'a #ty)
                    }
                } else {
                    quote! { #variant(&'a #ty) }
                }
            })
            .collect();
        quote! {
            #[derive(Debug, Clone, Copy)]
            pub enum ParseTree<'a> {
                #(#arms,)*
                Token(Token)
            }
        }
    }

    fn gen_parse_tree_impl(&self) -> TokenStream {
        let unwrap_methods = self.gen_unwrap_methods();
        let children_method = self.gen_parse_tree_children_method();
        let name_method = self.gen_parse_tree_name_method();
        let child_count_method = self.gen_parse_tree_child_count_method();
        let span_method = self.gen_parse_tree_span_method();
        let is_amb_method = self.gen_parse_tree_is_amb_method();
        let node_id_method = self.gen_parse_tree_node_id_method();
        let origin_method = self.gen_parse_tree_origin_method();
        quote! {
            impl<'a> ParseTree<'a> {
                #children_method
                #name_method
                #child_count_method
                #span_method
                #is_amb_method
                #node_id_method
                #origin_method
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

    fn gen_unwrap_methods(&self) -> Vec<TokenStream> {
        self.grammar
            .nonterminals()
            .map(|n| {
                let method_ident = format_ident!("unwrap_{}", to_snake_case(&n.name));
                let variant = nt_ident(&n.name);
                let var = safe_ident(&to_snake_case(&n.name));
                let return_type = self.nonterminal_type(n);
                quote! {
                    fn #method_ident(self) -> &'a #return_type {
                        match self {
                            ParseTree::#variant(#var) => #var,
                            _ => panic!(),
                        }
                    }
                }
            })
            .collect()
    }

    fn gen_parse_tree_children_method(&self) -> TokenStream {
        let arms: Vec<_> = self
            .grammar
            .nonterminals()
            .map(|n| {
                let variant = nt_ident(&n.name);
                let var_ident = safe_ident(&to_snake_case(&n.name));
                if n.is_plus() && !self.config.unsafe_mode {
                    // Plus::Amb wraps a slice of complete Plus subtrees. Show those as
                    // children directly; routing through iter() would push the same Amb
                    // back as a child and recurse forever. The unsafe mode has no Amb
                    // variant, so its Plus arm iterates like Star below.
                    quote! {
                        ParseTree::#variant(#var_ident) => match #var_ident {
                            #variant::Amb(alts) => alts.iter().copied().map(ParseTree::#variant).collect(),
                            _ => #var_ident.iter().collect(),
                        }
                    }
                } else if n.is_plus() || n.is_star() {
                    quote! {
                        ParseTree::#variant(#var_ident) => #var_ident.iter().collect()
                    }
                } else {
                    quote! {
                        ParseTree::#variant(#var_ident) => (0..#var_ident.child_count())
                            .filter_map(|i| #var_ident.child(i))
                            .collect()
                    }
                }
            })
            .collect();
        quote! {
            pub fn children(&self) -> Vec<ParseTree<'a>> {
                match self {
                    #(#arms,)*
                    ParseTree::Token(_) => vec![],
                }
            }
        }
    }

    fn gen_parse_tree_name_method(&self) -> TokenStream {
        let arms = self.grammar.nonterminals().map(|n| {
            let name_ident = nt_ident(&n.name);
            let var_ident = safe_ident(&to_snake_case(&n.name));
            quote! { ParseTree::#name_ident(#var_ident) => #var_ident.display_name() }
        });
        quote! {
            pub fn display_name(&self) -> &'static str {
                match self {
                    #(#arms,)*
                    ParseTree::Token(token) => token.kind.name(),
                }
            }
        }
    }

    fn gen_parse_tree_child_count_method(&self) -> TokenStream {
        let arms: Vec<_> = self
            .grammar
            .nonterminals()
            .map(|n| {
                let variant = nt_ident(&n.name);
                let var_ident = safe_ident(&to_snake_case(&n.name));
                quote! {
                    ParseTree::#variant(#var_ident) => #var_ident.child_count()
                }
            })
            .collect();
        quote! {
            pub fn child_count(&self) -> usize {
                match self {
                    #(#arms,)*
                    ParseTree::Token(_) => 0,
                }
            }
        }
    }

    fn gen_parse_tree_span_method(&self) -> TokenStream {
        let arms: Vec<_> = self
            .grammar
            .nonterminals()
            .map(|n| {
                let variant = nt_ident(&n.name);
                let var_ident = safe_ident(&to_snake_case(&n.name));
                quote! {
                    ParseTree::#variant(#var_ident) => #var_ident.span()
                }
            })
            .collect();
        quote! {
            pub fn span(&self) -> Span {
                match self {
                    #(#arms,)*
                    ParseTree::Token(token) => token.span(),
                }
            }
        }
    }

    fn gen_parse_tree_is_amb_method(&self) -> TokenStream {
        if self.config.unsafe_mode {
            return quote! {
                #[doc = "Always false: the unsafe mode produces no ambiguity nodes, and its"]
                #[doc = "nonterminal types have no `Amb` variant."]
                pub fn is_amb(&self) -> bool {
                    false
                }
            };
        }
        let arms = self.grammar.nonterminals().map(|n| {
            let variant = nt_ident(&n.name);
            let var_ident = safe_ident(&to_snake_case(&n.name));
            // A start nonterminal is wrapped in `Start<T, L>`, which has no `Amb` variant;
            // its ambiguity surfaces on the inner type, not here.
            if self.grammar.is_start(n) {
                quote! { ParseTree::#variant(_) => false }
            } else {
                quote! { ParseTree::#variant(#var_ident) => matches!(#var_ident, #variant::Amb(_)) }
            }
        });
        quote! {
            #[doc = "True when this node is an ambiguity cluster (any `*::Amb` variant). The"]
            #[doc = "uniform way to detect ambiguity without matching each nonterminal's enum."]
            pub fn is_amb(&self) -> bool {
                match self {
                    #(#arms,)*
                    ParseTree::Token(_) => false,
                }
            }
        }
    }

    fn gen_parse_tree_node_id_method(&self) -> TokenStream {
        if self.config.unsafe_mode {
            return quote! {
                #[doc = "Always `None`: node identity exists to detect sharing in an ambiguity"]
                #[doc = "DAG, and the unsafe mode builds a tree with no shared nodes. `None`"]
                #[doc = "also lets the s-expression and JSON renderers skip their sharing maps."]
                pub fn node_id(&self) -> Option<usize> {
                    None
                }
            };
        }
        let arms = self.grammar.nonterminals().map(|n| {
            let variant = nt_ident(&n.name);
            let var_ident = safe_ident(&to_snake_case(&n.name));
            quote! { ParseTree::#variant(#var_ident) => Some(*#var_ident as *const _ as usize) }
        });
        quote! {
            #[doc = "Pointer identity of the underlying node, or `None` for tokens (by-value"]
            #[doc = "leaves that are never shared). Two parse trees with the same `node_id` are"]
            #[doc = "the same allocation, i.e. a node shared between parents in the ambiguity DAG."]
            pub fn node_id(&self) -> Option<usize> {
                match self {
                    #(#arms,)*
                    ParseTree::Token(_) => None,
                }
            }
        }
    }

    fn gen_parse_tree_origin_method(&self) -> TokenStream {
        let arms = self.grammar.nonterminals().map(|n| {
            let variant = nt_ident(&n.name);
            let var_ident = safe_ident(&to_snake_case(&n.name));
            quote! { ParseTree::#variant(#var_ident) => #var_ident.origin() }
        });
        quote! {
            pub fn origin(&self) -> Option<Origin> {
                match self {
                    #(#arms,)*
                    ParseTree::Token(_) => None,
                }
            }
        }
    }

    fn gen_opt_node_impl(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ty = self.nonterminal_type(nonterminal);
        let generics = self.gen_enum_generics(nonterminal);
        let opt_type = nt_ident(&nonterminal.name);
        let alternatives = self.grammar.alternatives(nonterminal);
        let alt0 = &alternatives[0];
        let inner_symbol = &alt0.symbols[0];
        let def_id = inner_symbol.resolved_def();
        let inner_type = match self.grammar.definition(def_id) {
            Definition::Terminal(_) => quote! { Token },
            Definition::Nonterminal(nt) => self.nonterminal_type(nt),
        };
        let field_name = safe_ident(&gen_field_name(self.grammar, inner_symbol, 0, false));
        let amb_arm = if self.config.unsafe_mode {
            quote! {}
        } else {
            quote! { #opt_type::Amb(_) => panic!("unexpected ambiguity in optional node"), }
        };

        quote! {
            impl #generics OptNode for #ty {
                type Inner = #inner_type;
                fn value(&self) -> Option<&Self::Inner> {
                    match self {
                        #opt_type::Alt0 { #field_name, .. } => Some(#field_name),
                        #opt_type::Alt1 { .. } => None,
                        #amb_arm
                    }
                }
            }
        }
    }

    /// Generates `as_xxx` accessor methods for single-symbol alternation nonterminals.
    ///
    /// For alternations where each alternative contains exactly one symbol (terminal or nonterminal),
    /// this generates accessor methods that return `Option<&T>` for each variant.
    ///
    /// # Example
    ///
    /// For `RangeElement = Range | RangeChar`, generates:
    /// ```ignore
    /// impl RangeElement {
    ///     pub fn as_range(&self) -> Option<&Range> { ... }
    ///     pub fn as_range_char(&self) -> Option<&Token> { ... }
    /// }
    /// ```
    fn gen_alt_accessors(&self, nonterminal: &Nonterminal) -> TokenStream {
        let alt_type = nt_ident(&nonterminal.name);
        let alternatives = self.grammar.alternatives(nonterminal);

        let accessors: Vec<_> = alternatives
            .iter()
            .enumerate()
            .filter_map(|(i, alt)| {
                let symbol = &alt.symbols[0];
                let def = self.grammar.definition(symbol.resolved_def());
                let (method_name, return_type) = match def {
                    // For alternations with a single terminal, e.g., `Type | "void"`,
                    // we generate `as_void()`. Terminal names for keyword literals include
                    // quotes ("\"void\""), so strip them first. If the stripped name isn't
                    // a valid identifier (e.g., "+"), skip -- those need explicit naming.
                    Definition::Terminal(t) => {
                        let stripped = t.name.trim_matches('"');
                        let snake = to_snake_case(stripped);
                        if !is_valid_rust_ident(&snake) {
                            return None;
                        }
                        let method = format_ident!("as_{}", snake);
                        (method, quote! { Token })
                    }
                    Definition::Nonterminal(nt) => {
                        let method = format_ident!("as_{}", to_snake_case(&nt.name));
                        let ret = nt_ident(&nt.name);
                        let ret_ty = if self.has_lifetime(nt) {
                            quote! { #ret<'_> }
                        } else {
                            quote! { #ret }
                        };
                        (method, ret_ty)
                    }
                };
                let variant = format_ident!("{}", to_pascal_case(&alternative_label(alt, i)));
                let field_name = safe_ident(&gen_field_name(self.grammar, symbol, 0, false));

                Some(quote! {
                    pub fn #method_name(&self) -> Option<&#return_type> {
                        match self {
                            #alt_type::#variant { #field_name, .. } => Some(#field_name),
                            _ => None,
                        }
                    }
                })
            })
            .collect();

        let ty = self.nonterminal_type(nonterminal);
        let generics = self.gen_enum_generics(nonterminal);
        quote! {
            impl #generics #ty {
                #(#accessors)*
            }
        }
    }

    /// Generates a typed accessor method for Plus/Star/Opt nonterminals.
    ///
    /// These accessors provide a convenient way to iterate over elements
    /// without manually navigating through wrapper types. The method name
    /// is the pluralized snake_case form of the child element type.
    ///
    /// # Type Hierarchy
    ///
    /// EBNF operators desugar into a type hierarchy:
    /// - `Symbol*` → Star (struct wrapping `Symbol+?`)
    /// - `Symbol+?` → Opt (enum: None | Some(Symbol+))
    /// - `Symbol+` → Plus (recursive enum: Base(Symbol) | Rec(Symbol+, Symbol))
    ///
    /// # Generated Accessors
    ///
    /// For `Symbol+` (Plus):
    /// ```ignore
    /// pub fn symbols(&self) -> impl Iterator<Item = &Symbol> { ... }
    /// ```
    ///
    /// For `Symbol+?` (Opt wrapping Plus):
    /// ```ignore
    /// pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
    ///     self.value().into_iter().flat_map(|inner| inner.symbols())
    /// }
    /// ```
    ///
    /// For `Symbol*` (Star):
    /// ```ignore
    /// pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
    ///     self.symbol_opt.symbols()  // delegates to inner Opt's accessor
    /// }
    /// ```
    ///
    /// # Nested Types
    ///
    /// For nested constructs like `{Regex+ "|"}+` (Plus of Plus with separator),
    /// the accessor returns an iterator of iterators to preserve the grouping structure:
    /// ```ignore
    /// // For {Regex+ "|"}+, returns iterator over groups, each group is an iterator over Regex
    /// pub fn regexes(&self) -> impl Iterator<Item = impl Iterator<Item = &Regex>> {
    ///     self.iter().filter_map(|node| match node {
    ///         ParseTree::RegexPlus(r) => Some(r.regexes()),
    ///         _ => None,
    ///     })
    /// }
    /// ```
    fn gen_typed_accessor(&self, nonterminal: &Nonterminal) -> Option<TokenStream> {
        match &nonterminal.origin {
            Some(Symbol::Plus(inner, _)) => {
                let element_types = get_list_element_types(self.grammar, inner);
                if element_types.is_empty() {
                    return None;
                }
                if element_types.len() > 1 {
                    return self.gen_alt_variant_accessors_for_plus(nonterminal, &element_types);
                }
                let innermost = &element_types[0];
                let child_name = get_element_type_name(self.grammar, nonterminal)?;
                let innermost_type = symbol_ident(self.grammar, innermost);

                let method_name = safe_ident(&pluralize(&to_snake_case(&innermost.name)));
                let child_type = nt_ident(child_name);
                // An ambiguous list segment appears here as the list's own `Amb` node, not
                // an element. A typed accessor can't represent it, so fail loud rather than
                // silently drop it and return a short list. Ambiguity-aware callers use the
                // general `iter()` / `children()` path with `is_amb()` instead.
                let amb_arm = if self.config.unsafe_mode {
                    quote! {}
                } else {
                    let panic_msg = format!("{} is ambiguous", nonterminal.name);
                    quote! { other if other.is_amb() => panic!(#panic_msg), }
                };

                let return_type = self.gen_accessor_return_type(nonterminal, innermost);
                if child_name == innermost.name {
                    // Simple case: e.g., `Regex+` or `Identifier+`
                    Some(quote! {
                        pub fn #method_name(&'a self) -> #return_type {
                            self.iter().filter_map(|node| match node {
                                ParseTree::#innermost_type(r) => Some(r),
                                #amb_arm
                                _ => None,
                            })
                        }
                    })
                } else if let Symbol::Group(_) = inner.as_ref() {
                    // Group case: e.g., `("|" Regex)+` or `("!" Identifier)+`
                    let field_name = safe_ident(&to_snake_case(&innermost.name));
                    Some(quote! {
                        pub fn #method_name(&'a self) -> #return_type {
                            self.iter().filter_map(|node| match node {
                                ParseTree::#child_type(r) => Some(r.#field_name()),
                                #amb_arm
                                _ => None,
                            })
                        }
                    })
                } else {
                    // Nested case: e.g., `{Regex+ "|"}+` where child is an intermediate Plus/Star type.
                    Some(quote! {
                        pub fn #method_name(&'a self) -> #return_type {
                            self.iter().filter_map(|node| match node {
                                ParseTree::#child_type(r) => Some(r.#method_name()),
                                #amb_arm
                                _ => None,
                            })
                        }
                    })
                }
            }
            Some(Symbol::Star(inner, _)) => {
                let element_types = get_list_element_types(self.grammar, inner);
                if element_types.is_empty() {
                    return None;
                }
                let alternatives = self.grammar.alternatives(nonterminal);
                let opt_symbol = alternatives[0].symbols.first()?;
                let opt_field_name =
                    safe_ident(&gen_field_name(self.grammar, opt_symbol, 0, false));
                let ident = nt_ident(&nonterminal.name);
                let alt0_variant = nt_ident(&alternative_label(&alternatives[0], 0));
                let amb_arm = if self.config.unsafe_mode {
                    quote! {}
                } else {
                    let panic_msg = format!("{} is ambiguous", nonterminal.name);
                    quote! { #ident::Amb(_) => panic!(#panic_msg), }
                };

                let methods: Vec<_> = element_types
                    .iter()
                    .map(|elem| {
                        let method_name = safe_ident(&pluralize(&to_snake_case(&elem.name)));
                        let return_type = self.gen_accessor_return_type(nonterminal, elem);
                        quote! {
                            pub fn #method_name(&self) -> #return_type {
                                match self {
                                    #ident::#alt0_variant { #opt_field_name, .. } => #opt_field_name.#method_name(),
                                    #amb_arm
                                }
                            }
                        }
                    })
                    .collect();
                Some(quote! { #(#methods)* })
            }
            Some(Symbol::Opt(inner)) => {
                // Opt types that wrap Plus/Star (e.g., `SyntaxRule+?`).
                // Delegate to the inner Plus/Star's accessor via OptNode::value().
                let inner_inner = match inner.as_ref() {
                    Symbol::Plus(s, _) | Symbol::Star(s, _) => s.as_ref(),
                    _ => return None,
                };

                let element_types = get_list_element_types(self.grammar, inner_inner);
                if element_types.is_empty() {
                    return None;
                }
                let methods: Vec<_> = element_types
                    .iter()
                    .map(|elem| {
                        let method_name = safe_ident(&pluralize(&to_snake_case(&elem.name)));
                        let return_type = self.gen_accessor_return_type(nonterminal, elem);
                        quote! {
                            pub fn #method_name(&'a self) -> #return_type {
                                self.value().into_iter().flat_map(|inner| inner.#method_name())
                            }
                        }
                    })
                    .collect();
                Some(quote! { #(#methods)* })
            }
            Some(Symbol::Group(_)) => {
                // Field accessors on a Group come from gen_field_accessor_methods.
                // An iter-based accessor here would pick the wrong token when the
                // Group has multiple positional `Token` fields (e.g. for
                // `("\\" Identifier)`, find_map returns the `"\\"`).
                None
            }
            _ => self.gen_delegated_accessors(nonterminal),
        }
    }

    // For user-defined rules with a single-symbol alternative, delegate the
    // EBNF field's typed accessors onto the parent struct. For example, given
    // `S = Expr+`, the struct `S` gets an `exprs()` method that delegates to
    // the inner Plus field.
    // Returns None when no accessor is generated. This happens when the
    // nonterminal has more than one alternative, more than one symbol in its
    // sole alternative, the single symbol is not EBNF-derived, or the EBNF
    // child itself has no typed accessors.
    fn gen_delegated_accessors(&self, nonterminal: &Nonterminal) -> Option<TokenStream> {
        let alternatives = self.grammar.alternatives(nonterminal);
        if alternatives.len() != 1 || alternatives[0].symbols.len() != 1 {
            return None;
        }
        let symbol = &alternatives[0].symbols[0];
        let child_nt = match self.grammar.definition(symbol.resolved_def()) {
            Definition::Nonterminal(nt) => nt,
            _ => return None,
        };
        let inner = match child_nt.origin.as_ref() {
            Some(Symbol::Plus(s, _) | Symbol::Star(s, _)) => s.as_ref(),
            Some(Symbol::Opt(s)) => match s.as_ref() {
                Symbol::Plus(s, _) | Symbol::Star(s, _) => s.as_ref(),
                _ => return None,
            },
            Some(origin @ Symbol::Group(_)) => origin,
            _ => return None,
        };
        let field_name = safe_ident(&gen_field_name(self.grammar, symbol, 0, false));
        let ident = nt_ident(&nonterminal.name);
        let alt0_variant = nt_ident(&alternative_label(&alternatives[0], 0));
        let amb_arm = if self.config.unsafe_mode {
            quote! {}
        } else {
            let panic_msg = format!("{} is ambiguous", nonterminal.name);
            quote! { #ident::Amb(_) => panic!(#panic_msg), }
        };
        let element_types = get_list_element_types(self.grammar, inner);
        let methods: Vec<_> = element_types
            .iter()
            .filter(|elem| {
                // Skip delegated methods whose name matches the field name: the
                // field accessor on `self` already covers it, and emitting both
                // produces duplicate definitions (and a self-recursive call here).
                let m = pluralize(&to_snake_case(&elem.name));
                safe_ident(&m) != field_name
            })
            .map(|elem| {
                let method_name = safe_ident(&pluralize(&to_snake_case(&elem.name)));
                let return_type = self.gen_accessor_return_type(child_nt, elem);
                quote! {
                    pub fn #method_name(&self) -> #return_type {
                        match self {
                            #ident::#alt0_variant { #field_name, .. } => #field_name.#method_name(),
                            #amb_arm
                        }
                    }
                }
            })
            .collect();
        if methods.is_empty() {
            None
        } else {
            Some(quote! { #(#methods)* })
        }
    }

    // Returns the return type of a typed accessor that iterates over the children
    // of a desugared EBNF node in the parse tree.
    // - Plus (`Expr+`): `impl Iterator<Item = &Expr>`
    //   - separator (`{Expr ","}+`):
    //     `impl Iterator<Item = impl Iterator<Item = &Expr> + '_>`
    //   - alternation (`(Expr | Stmt)+`): one method per variant,
    //     `impl Iterator<Item = &Expr>` and `impl Iterator<Item = &Stmt>`
    //   - group (`("," Expr)+`): `impl Iterator<Item = &Expr>`
    // - Star (`Expr*`): desugared internally as Star -> Opt -> Plus, so the
    //   return type is determined by the inner Plus
    // - Group (`("," Expr)`): `&Expr`
    fn gen_accessor_return_type(
        &self,
        nonterminal: &Nonterminal,
        elem: &Identifier,
    ) -> TokenStream {
        let item_type = self.symbol_type(elem.resolve());
        match &nonterminal.origin {
            Some(Symbol::Group(_)) => quote! { #item_type },
            Some(Symbol::Plus(inner, _)) => {
                let child_name = get_element_type_name(self.grammar, nonterminal);
                let is_nested = child_name.is_some_and(|cn| cn != elem.name)
                    && !matches!(inner.as_ref(), Symbol::Group(_) | Symbol::Alt(_));
                if is_nested {
                    quote! { impl Iterator<Item = impl Iterator<Item = #item_type> + 'a> }
                } else {
                    quote! { impl Iterator<Item = #item_type> }
                }
            }
            Some(Symbol::Star(_, _) | Symbol::Opt(_)) => {
                let alternatives = self.grammar.alternatives(nonterminal);
                let child_symbol = &alternatives[0].symbols[0];
                match self.grammar.definition(child_symbol.resolved_def()) {
                    Definition::Nonterminal(nt) => self.gen_accessor_return_type(nt, elem),
                    _ => unreachable!("Star/Opt child should be a nonterminal"),
                }
            }
            _ => unreachable!("gen_accessor_return_type called on non-EBNF nonterminal"),
        }
    }

    /// Generates per-variant accessors for a Plus whose inner element is an Alt.
    /// For `(WS | LineComment)+`, generates `wses()` and `line_comments()` methods
    /// that filter the iter for each Alt variant.
    fn gen_alt_variant_accessors_for_plus(
        &self,
        nonterminal: &Nonterminal,
        element_types: &[Identifier],
    ) -> Option<TokenStream> {
        let child_name = get_element_type_name(self.grammar, nonterminal)?;
        let child_type = nt_ident(child_name);
        let alt_nonterminal = self.grammar.nonterminal(child_name)?;
        let alt_alternatives = self.grammar.alternatives(alt_nonterminal);

        if alt_alternatives.len() != element_types.len() {
            return None;
        }

        // Each accessor keeps only its own element variant; `_ => None` skips the sibling
        // variants. An ambiguous segment, however, must fail loud rather than be dropped.
        let amb_arm = if self.config.unsafe_mode {
            quote! {}
        } else {
            let panic_msg = format!("{} is ambiguous", nonterminal.name);
            quote! { other if other.is_amb() => panic!(#panic_msg), }
        };

        let methods: Vec<_> = element_types
            .iter()
            .enumerate()
            .map(|(i, elem)| {
                let alt = &alt_alternatives[i];
                let variant_name = nt_ident(&alternative_label(alt, i));
                let method_name = safe_ident(&pluralize(&to_snake_case(&elem.name)));
                let return_type = self.gen_accessor_return_type(nonterminal, elem);
                let field_name =
                    safe_ident(&gen_field_name(self.grammar, &alt.symbols[0], 0, false));
                let extract = if self.grammar.is_terminal(elem) {
                    quote! { Some(*#field_name) }
                } else {
                    quote! { Some(#field_name) }
                };
                quote! {
                    pub fn #method_name(&'a self) -> #return_type {
                        self.iter().filter_map(|node| match node {
                            ParseTree::#child_type(#child_type::#variant_name { #field_name, .. }) => #extract,
                            #amb_arm
                            _ => None,
                        })
                    }
                }
            })
            .collect();

        Some(quote! { #(#methods)* })
    }

    fn gen_list_node_impl_for_plus(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ident = nt_ident(&nonterminal.name);
        let alternatives = self.grammar.alternatives(nonterminal);
        // This method must only be called for list nodes, i.e., * and + nonterminals,
        // which always have two alternatives.
        assert_eq!(alternatives.len(), 2);
        let label = alternative_label(&alternatives[0], 0);
        let alt_variant = nt_ident(&label);
        let first_alt_fields = field_names(self.grammar, &alternatives[0]);
        let first_arm = match &nonterminal.origin {
            Some(Symbol::Plus(_symbol, sep)) => match sep {
                Some(_) => {
                    if first_alt_fields.len() == 5 {
                        // With layout: Plus ::= Plus Layout Sep Layout Item
                        let (f0, f1, f2, f3, f4) = (
                            &first_alt_fields[0],
                            &first_alt_fields[1],
                            &first_alt_fields[2],
                            &first_alt_fields[3],
                            &first_alt_fields[4],
                        );
                        quote! {
                            #ident::#alt_variant { #f0: rest, #f1: layout1, #f2: sep, #f3: layout2, #f4: item, .. } => {
                                items.push(item.as_parse_tree());
                                items.push(layout2.as_parse_tree());
                                items.push(sep.as_parse_tree());
                                items.push(layout1.as_parse_tree());
                                current = rest;
                            }
                        }
                    } else {
                        // Without layout: Plus ::= Plus Sep Item
                        let (f0, f1, f2) = (
                            &first_alt_fields[0],
                            &first_alt_fields[1],
                            &first_alt_fields[2],
                        );
                        quote! {
                            #ident::#alt_variant { #f0: rest, #f1: sep, #f2: item, .. } => {
                                items.push(item.as_parse_tree());
                                items.push(sep.as_parse_tree());
                                current = rest;
                            }
                        }
                    }
                }
                None => {
                    if first_alt_fields.len() == 3 {
                        // With layout: Plus ::= Plus Layout Item | Item
                        let (f0, f1, f2) = (
                            &first_alt_fields[0],
                            &first_alt_fields[1],
                            &first_alt_fields[2],
                        );
                        quote! {
                            #ident::#alt_variant { #f0: rest, #f1, #f2: item, .. } => {
                                items.push(item.as_parse_tree());
                                items.push(#f1.as_parse_tree());
                                current = rest;
                            }
                        }
                    } else {
                        // Without layout (@layout(none)): Plus ::= Plus Item | Item
                        let (f0, f1) = (&first_alt_fields[0], &first_alt_fields[1]);
                        quote! {
                            #ident::#alt_variant { #f0: rest, #f1: item, .. } => {
                                items.push(item.as_parse_tree());
                                current = rest;
                            }
                        }
                    }
                }
            },
            _ => unreachable!("Expected plus"),
        };
        let label = alternative_label(&alternatives[1], 1);
        let alt_variant = nt_ident(&label);
        let second_alt_fields = field_names(self.grammar, &alternatives[1]);
        let f0 = &second_alt_fields[0];
        let second_arm = quote! {
            #ident::#alt_variant { #f0: item, .. } => {
                items.push(item.as_parse_tree());
                break;
            }
        };
        let ty = self.nonterminal_type(nonterminal);
        let amb_arm = if self.config.unsafe_mode {
            quote! {}
        } else {
            quote! {
                #ident::Amb(_) => {
                    items.push(ParseTree::#ident(current));
                    break;
                }
            }
        };
        quote! {
            impl<'a> ListNode<'a> for #ty {
                fn iter(&'a self) -> IntoIter<ParseTree<'a>> {
                    let mut items = vec![];
                    let mut current = self;
                    loop {
                        match current {
                            #first_arm
                            #second_arm
                            #amb_arm
                        }
                    }
                    items.reverse();
                    items.into_iter()
                }
            }
        }
    }

    fn gen_list_node_impl_for_star(&self, nonterminal: &Nonterminal) -> TokenStream {
        let star_ident = nt_ident(&nonterminal.name);
        let star_ty = self.nonterminal_type(nonterminal);
        let star_alternatives = self.grammar.alternatives(nonterminal);
        // Star is a single-alternative enum: `Alt0 { opt_field, span } + Amb`.
        let star_alt0 = nt_ident(&alternative_label(&star_alternatives[0], 0));
        let first_symbol = &star_alternatives[0].symbols[0];
        let field_name = safe_ident(&gen_field_name(self.grammar, first_symbol, 0, false));
        let def_id = first_symbol.resolved_def();
        let opt_nonterminal = self.grammar.definition(def_id).as_nonterminal();
        let opt_alternatives = self.grammar.alternatives(opt_nonterminal);

        let opt_ident = nt_ident(&opt_nonterminal.name);
        let var_ident = safe_ident(&to_snake_case(&opt_nonterminal.name));
        let label = alternative_label(&opt_alternatives[0], 0);
        let opt_alt0 = nt_ident(&label);
        let opt_first_alt_fields = field_names(self.grammar, &opt_alternatives[0]);
        let f0 = &opt_first_alt_fields[0];
        let label = alternative_label(&opt_alternatives[1], 1);
        let opt_alt1 = nt_ident(&label);
        let (opt_amb_arm, star_amb_arm) = if self.config.unsafe_mode {
            (quote! {}, quote! {})
        } else {
            (
                quote! { #opt_ident::Amb(_) => vec![ParseTree::#opt_ident(#field_name)].into_iter(), },
                quote! { #star_ident::Amb(_) => vec![ParseTree::#star_ident(self)].into_iter(), },
            )
        };
        quote! {
            impl<'a> ListNode<'a> for #star_ty {
                fn iter(&'a self) -> IntoIter<ParseTree<'a>> {
                    match self {
                        #star_ident::#star_alt0 { #field_name, .. } => match #field_name {
                            #opt_ident::#opt_alt0 { #f0: #var_ident, .. } => #var_ident.iter(),
                            #opt_ident::#opt_alt1 { .. } => vec![].into_iter(),
                            #opt_amb_arm
                        },
                        #star_amb_arm
                    }
                }
            }
        }
    }

    fn gen_list_node_impl_for_group(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ident = nt_ident(&nonterminal.name);
        let ty = self.nonterminal_type(nonterminal);
        let alternatives = self.grammar.alternatives(nonterminal);
        // Groups always have exactly one alternative
        assert_eq!(alternatives.len(), 1);
        let alternative = &alternatives[0];
        let alt0_variant = nt_ident(&alternative_label(alternative, 0));
        let fields = field_names(self.grammar, alternative);

        let item_exprs: Vec<_> = fields
            .iter()
            .map(|field| quote! { #field.as_parse_tree() })
            .collect();

        let amb_arm = if self.config.unsafe_mode {
            quote! {}
        } else {
            quote! { #ident::Amb(_) => vec![ParseTree::#ident(self)].into_iter(), }
        };
        quote! {
            impl<'a> ListNode<'a> for #ty {
                fn iter(&'a self) -> IntoIter<ParseTree<'a>> {
                    match self {
                        #ident::#alt0_variant { #(#fields,)* .. } => {
                            vec![#(#item_exprs),*].into_iter()
                        }
                        #amb_arm
                    }
                }
            }
        }
    }

    fn gen_create_parse_tree_function(&self) -> TokenStream {
        let parser_name_ident = format_ident!("{}Parser", self.grammar.name);
        let builder_name_ident = format_ident!("{}ParseTreeBuilder", self.grammar.name);
        let arms: Vec<_> = self
            .grammar
            .nonterminals()
            .map(|n| {
                let name = &n.name;
                let const_name = format_ident!("{}", to_snake_case(name).to_uppercase());
                let function_name = format_ident!("create_parse_tree_{}", to_snake_case(name));
                let variant_name = nt_ident(name);
                quote! { crate::grammar_data::#const_name => ParseTree::#variant_name(#function_name(root_id, parser, builder)) }
            })
            .collect();
        quote! {
            pub fn create_parse_tree<'a>(
                root_id: SPPFNodeId,
                nonterminal_id: NonterminalId,
                parser: &#parser_name_ident,
                builder: &#builder_name_ident<'a>,
            ) -> ParseTree<'a> {
                match nonterminal_id {
                    #(#arms,)*
                    _ => panic!()
                }
            }
        }
    }

    /// Generates functions with the name create_parse_tree_#name, where name is the name of a nonterminal.
    fn gen_create_parse_tree_nonterminal_function(&self, nonterminal: &Nonterminal) -> TokenStream {
        let nonterminal_name = &nonterminal.name;
        let parser_name_ident = format_ident!("{}Parser", self.grammar.name);
        let builder_name_ident = format_ident!("{}ParseTreeBuilder", self.grammar.name);
        let return_type = self.nonterminal_type(nonterminal);
        let function_name = format_ident!("create_parse_tree_{}", to_snake_case(nonterminal_name));
        let unwrap_method = format_ident!("unwrap_{}", to_snake_case(nonterminal_name));
        quote! {
            pub fn #function_name<'a>(
                root_id: SPPFNodeId,
                parser: &#parser_name_ident,
                builder: &#builder_name_ident<'a>,
            ) -> &'a #return_type {
                visit_sppf(root_id, parser, builder).unwrap_one().#unwrap_method()
            }
        }
    }

    fn gen_to_sexpr_function(&self) -> TokenStream {
        let layout_name = self
            .grammar
            .layout
            .as_ref()
            .and_then(|s| s.as_identifier())
            .map(|i| i.name.as_str())
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! { None });

        quote! {
            const LAYOUT_NAME: Option<&str> = #layout_name;

            pub fn to_sexpr(node: ParseTree<'_>) -> String {
                iguana_runtime::parse_tree::to_sexpr(node, LAYOUT_NAME)
            }

            pub fn to_sexpr_with(node: ParseTree<'_>, options: SexprOptions) -> String {
                iguana_runtime::parse_tree::to_sexpr_with(node, LAYOUT_NAME, options)
            }
        }
    }

    fn gen_parse_tree_node_impl(&self) -> TokenStream {
        let kind_method = if self.config.unsafe_mode {
            quote! {
                fn kind(&self) -> NodeKind {
                    if matches!(self, ParseTree::Token(_)) {
                        NodeKind::Token
                    } else {
                        NodeKind::Nonterminal
                    }
                }
            }
        } else {
            quote! {
                fn kind(&self) -> NodeKind {
                    if self.is_amb() {
                        NodeKind::Amb
                    } else if matches!(self, ParseTree::Token(_)) {
                        NodeKind::Token
                    } else {
                        NodeKind::Nonterminal
                    }
                }
            }
        };
        quote! {
            impl<'a> ParseTreeNode for ParseTree<'a> {
                fn children(&self) -> Vec<Self> {
                    ParseTree::children(self)
                }

                fn display_name(&self) -> &'static str {
                    ParseTree::display_name(self)
                }

                fn span(&self) -> Span {
                    ParseTree::span(self)
                }

                #kind_method

                fn node_id(&self) -> Option<usize> {
                    ParseTree::node_id(self)
                }

                fn origin(&self) -> Option<Origin> {
                    ParseTree::origin(self)
                }
            }
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
        Symbol::Identifier(ident) | Symbol::Call { name: ident, .. } => {
            if let Some(def_id) = ident.definition {
                if let Definition::Nonterminal(nt) = grammar.definition(def_id) {
                    if let Some(origin) = &nt.origin {
                        match origin {
                            Symbol::Star(inner, _)
                            | Symbol::Plus(inner, _)
                            | Symbol::Opt(inner) => {
                                if let Symbol::Identifier(inner_ident) = inner.as_ref() {
                                    let snake = to_snake_case(&inner_ident.name);
                                    if is_valid_rust_ident(&snake) {
                                        return Some(snake);
                                    }
                                }
                                return None;
                            }
                            // Must match gen_field_name, which uses the inner name for
                            // Exclude origins. Without this, `E!X '+' E` would count
                            // "e_except_x" and "e" as different base names, miss the
                            // duplicate, and produce two fields both named `e`.
                            Symbol::Exclude { symbol, .. } => {
                                if let Some(inner_ident) = symbol.as_identifier() {
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
        Symbol::Binding { symbol, .. } => get_symbol_base_name(grammar, symbol),
        Symbol::Labeled { .. } => None,
        Symbol::Literal(_) => None,
        Symbol::Group(_) => None,
        Symbol::Alt(_) => None,
        Symbol::Except { symbol, .. }
        | Symbol::FollowRestriction { symbol, .. }
        | Symbol::PrecedeRestriction { symbol, .. } => get_symbol_base_name(grammar, symbol),
        Symbol::Exclude { .. } => {
            unreachable!("Exclude should be desugared before code generation")
        }
        Symbol::Condition(_) => None,
        Symbol::Return(_) => None,
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

/// Appends `_{position}` to a field name to avoid duplicates, where
/// `position` is the symbol's index in the alternative.
fn with_index(name: String, position: usize, needs_index: bool) -> String {
    if needs_index {
        format!("{}_{}", name, position)
    } else {
        name
    }
}

fn gen_field_name(
    grammar: &Grammar,
    symbol: &Symbol,
    position: usize,
    needs_index: bool,
) -> String {
    if let Some(label) = symbol.label() {
        return to_snake_case(label);
    }

    match symbol {
        Symbol::Star(inner, _) | Symbol::Plus(inner, _) => {
            if let Symbol::Identifier(ident) = inner.as_ref() {
                let snake = to_snake_case(&ident.name);
                if is_valid_rust_ident(&snake) {
                    with_index(pluralize(&snake), position, needs_index)
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
                    with_index(snake, position, needs_index)
                } else {
                    format!("field_{}", position)
                }
            } else {
                format!("field_{}", position)
            }
        }
        Symbol::Identifier(ident) | Symbol::Call { name: ident, .. } => {
            // Check if this identifier points to a derived nonterminal (Star/Plus/Opt)
            if let Some(def_id) = ident.definition {
                if let Definition::Nonterminal(nt) = grammar.definition(def_id) {
                    if let Some(origin) = &nt.origin {
                        match origin {
                            Symbol::Star(inner, _) | Symbol::Plus(inner, _) => {
                                if let Symbol::Identifier(inner_ident) = inner.as_ref() {
                                    let snake = to_snake_case(&inner_ident.name);
                                    if is_valid_rust_ident(&snake) {
                                        return with_index(
                                            pluralize(&snake),
                                            position,
                                            needs_index,
                                        );
                                    }
                                }
                            }
                            Symbol::Opt(inner) => {
                                if let Symbol::Identifier(inner_ident) = inner.as_ref() {
                                    let snake = to_snake_case(&inner_ident.name);
                                    if is_valid_rust_ident(&snake) {
                                        return with_index(snake, position, needs_index);
                                    }
                                }
                            }
                            // Exclude-derived nonterminals have Exclude as their origin
                            Symbol::Exclude { symbol, .. } => {
                                if let Some(inner_ident) = symbol.as_identifier() {
                                    let snake = to_snake_case(&inner_ident.name);
                                    if is_valid_rust_ident(&snake) {
                                        return with_index(snake, position, needs_index);
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
                with_index(snake_case, position, needs_index)
            } else {
                format!("lit_{}", position)
            }
        }
        Symbol::Binding { symbol, .. } => gen_field_name(grammar, symbol, position, needs_index),
        Symbol::Labeled { .. } => format!("field_{}", position),
        Symbol::Literal(_) => format!("field_{}", position),
        Symbol::Group(_) => format!("field_{}", position),
        Symbol::Alt(_) => format!("field_{}", position),
        Symbol::Except { symbol, .. }
        | Symbol::FollowRestriction { symbol, .. }
        | Symbol::PrecedeRestriction { symbol, .. } => {
            gen_field_name(grammar, symbol, position, needs_index)
        }
        Symbol::Exclude { .. } => {
            unreachable!("Exclude should be desugared before code generation")
        }
        Symbol::Condition(_) => format!("field_{}", position),
        Symbol::Return(_) => format!("field_{}", position),
    }
}

/// The `Origin` variant a derived nonterminal reports, mapped from the EBNF
/// operator it was derived from, or `None` for a nonterminal with no
/// presentation origin (user-declared, or derived by exclude or precedence
/// desugaring).
fn origin_kind(nonterminal: &Nonterminal) -> Option<TokenStream> {
    match &nonterminal.origin {
        Some(Symbol::Opt(_)) => Some(quote! { Origin::Opt }),
        Some(Symbol::Star(_, _) | Symbol::Plus(_, _)) => Some(quote! { Origin::List }),
        Some(Symbol::Group(_)) => Some(quote! { Origin::Group }),
        Some(Symbol::Alt(_)) => Some(quote! { Origin::Alt }),
        _ => None,
    }
}

fn child_by_index(grammar: &Grammar, alternative: &Alternative) -> TokenStream {
    let counts = count_symbol_occurrences(grammar, &alternative.symbols);
    let cases: Vec<_> = alternative
        .symbols
        .iter()
        .filter(|s| s.is_parse_tree_symbol())
        .enumerate()
        .map(|(i, s)| {
            let i_lit = Literal::usize_unsuffixed(i);
            let base_name = get_symbol_base_name(grammar, s);
            let needs_index =
                base_name.is_some_and(|name| counts.get(&name).copied().unwrap_or(0) > 1);
            let field_name = safe_ident(&gen_field_name(grammar, s, i, needs_index));
            let def = grammar.definition(s.resolved_def());
            let wrap = match def {
                Definition::Terminal(_) => quote! { ParseTree::Token(*#field_name) },
                Definition::Nonterminal(nt) => {
                    let variant = nt_ident(&nt.name);
                    quote! { ParseTree::#variant(#field_name) }
                }
            };
            quote! { #i_lit => Some(#wrap) }
        })
        .collect();
    if cases.is_empty() {
        return quote! { None };
    }
    quote! {
        match index {
            #(#cases,)*
            _ => None,
        }
    }
}

fn gen_token_struct() -> TokenStream {
    quote! {
        #[derive(Debug, Clone, Copy)]
        pub struct Token {
            pub kind: TokenKind,
            span: Span,
        }
    }
}

fn gen_token_impl() -> TokenStream {
    quote! {
        impl Token {
            pub fn as_parse_tree<'a>(&self) -> ParseTree<'a> {
                ParseTree::Token(*self)
            }
            pub fn span(&self) -> Span {
                self.span
            }
        }
    }
}

fn gen_token_kind_enum(terminals: &[(TerminalId, String)]) -> TokenStream {
    let terminal_ids: Vec<_> = terminals
        .iter()
        .map(|(terminal_id, name)| {
            let ident = format_ident!("T{}", terminal_id.0);
            quote! {
                #[comment = #name]
                #ident
            }
        })
        .collect();
    quote! {
        #[derive(Debug, Clone, Copy)]
        pub enum TokenKind {
            #(#terminal_ids),*
        }
    }
}

fn gen_token_kind_impl(terminals: &[(TerminalId, String)]) -> TokenStream {
    let terminal_ids: Vec<_> = terminals
        .iter()
        .map(|(id, name)| {
            let ident = format_ident!("T{}", id.0);
            quote! {
                TokenKind::#ident => #name
            }
        })
        .collect();
    quote! {
        impl TokenKind {
            pub fn name(&self) -> &'static str {
                match self {
                    #(#terminal_ids,)*
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

fn field_names(grammar: &Grammar, alternative: &Alternative) -> Vec<Ident> {
    let counts = count_symbol_occurrences(grammar, &alternative.symbols);
    alternative
        .symbols
        .iter()
        .filter(|s| s.is_parse_tree_symbol())
        .enumerate()
        .map(|(i, s)| {
            let base_name = get_symbol_base_name(grammar, s);
            let needs_index =
                base_name.is_some_and(|name| counts.get(&name).copied().unwrap_or(0) > 1);
            safe_ident(&gen_field_name(grammar, s, i, needs_index))
        })
        .collect::<Vec<_>>()
}

fn gen_new_token_method() -> TokenStream {
    quote! {
        fn new_token(&self, terminal_node: &TerminalNode) -> ParseTree<'a> {
            ParseTree::Token(Token {
                kind: token_kind(terminal_node.terminal_id),
                span: terminal_node.span,
            })
        }
    }
}

fn gen_list_node_trait() -> TokenStream {
    quote! {
        pub trait ListNode<'a> {
            fn iter(&'a self) -> IntoIter<ParseTree<'a>>;
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

/// Returns true if the nonterminal is an alternation where each alternative has exactly one symbol.
/// This includes both anonymous inline alternations (Symbol::Alt origin) and named nonterminals
/// like `RangeElement = Range | RangeChar`.
fn is_single_symbol_alternation(grammar: &Grammar, nonterminal: &Nonterminal) -> bool {
    // Anonymous inline alternations always qualify
    if matches!(&nonterminal.origin, Some(Symbol::Alt(_))) {
        return true;
    }
    // Named nonterminals: check if they have multiple alternatives, each with exactly one symbol.
    let alternatives = grammar.alternatives(nonterminal);
    alternatives.len() > 1
        && alternatives.iter().all(|alt| {
            alt.symbols.len() == 1
                && match grammar.definition(alt.symbols[0].resolved_def()) {
                    Definition::Nonterminal(_) => true,
                    Definition::Terminal(t) => !t.is_literal(),
                }
        })
}

/// Returns `Token` for terminals, or the PascalCase ident for nonterminals.
/// Use this for match patterns and enum variant names.
fn symbol_ident(grammar: &Grammar, ident: &Identifier) -> Ident {
    if grammar.is_terminal(ident) {
        Ident::new("Token", Span::call_site())
    } else {
        Ident::new(&nonterminal_type_name(&ident.name), Span::call_site())
    }
}

/// Returns the named element types inside an EBNF symbol.
///
/// Walks through Plus, Star, Group, and Alt wrappers to find the inner
/// named identifiers. Literal terminals (e.g., `"!"`, `"|"`) are
/// excluded since they carry no semantic value for typed accessors.
///
/// Returns a single element for simple repetitions, multiple elements
/// for alternations:
/// - `A+` -> `[A]`
/// - `{A ","}+` -> `[A]`
/// - `("!" A)+` -> `[A]`
/// - `(A | B)+` -> `[A, B]`
///
/// Multiple elements occur only for `Symbol::Alt`. For alternations,
/// only variants that are simple identifiers are included; nested EBNF
/// variants like `A*` in `(A* | C)` are skipped since the desugared
/// nonterminal type would not match. Returns an empty vec when no named
/// elements are found (e.g., only literals).
fn get_list_element_types(grammar: &Grammar, symbol: &Symbol) -> Vec<Identifier> {
    match symbol {
        Symbol::Identifier(ident) => {
            if let Some(def_id) = ident.definition {
                if let Definition::Terminal(t) = grammar.definition(def_id) {
                    if t.is_literal() {
                        return vec![];
                    }
                }
            }
            vec![ident.clone()]
        }
        Symbol::Plus(inner, _) | Symbol::Star(inner, _) => get_list_element_types(grammar, inner),
        // No recursion into Alt variants: only simple identifiers are collected.
        // Nested EBNF like `A*` in `(A* | C)` is skipped.
        Symbol::Alt(variants) => variants
            .iter()
            .filter_map(|v| match v {
                Symbol::Identifier(ident) => {
                    if let Some(def_id) = ident.definition {
                        if let Definition::Terminal(t) = grammar.definition(def_id) {
                            if t.is_literal() {
                                return None;
                            }
                        }
                    }
                    Some(ident.clone())
                }
                _ => None,
            })
            .collect(),
        Symbol::Group(elements) => {
            let named: Vec<_> = elements
                .iter()
                .flat_map(|elem| get_list_element_types(grammar, elem))
                .collect();
            // A group has a flat element type only when its named element is not
            // itself a list. For `(Annotation* "[" "]")` the named element is a
            // list, whose Group field accessor returns the inner list node rather
            // than the element, so there is no flat type and callers fall back to
            // iter()/children().
            let names_a_list = elements
                .iter()
                .any(|elem| elem.is_list() && !get_list_element_types(grammar, elem).is_empty());
            if named.len() == 1 && !names_a_list {
                named
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

/// Gets the element type name from the base alternative of a Plus/Star nonterminal.
fn get_element_type_name<'a>(grammar: &'a Grammar, nonterminal: &Nonterminal) -> Option<&'a str> {
    let alternatives = grammar.alternatives(nonterminal);
    let base_alt = if alternatives.len() == 1 {
        &alternatives[0]
    } else {
        &alternatives[1]
    };
    let child_symbol = base_alt.symbols.first()?;
    let child_def = grammar.definition(child_symbol.resolved_def());
    Some(child_def.name())
}

fn pluralize(word: &str) -> String {
    if word.ends_with("s") || word.ends_with("x") || word.ends_with("ch") || word.ends_with("sh") {
        format!("{}es", word)
    } else if word.ends_with("y")
        && !word.ends_with("ay")
        && !word.ends_with("ey")
        && !word.ends_with("oy")
        && !word.ends_with("uy")
    {
        format!("{}ies", &word[..word.len() - 1])
    } else {
        format!("{}s", word)
    }
}

fn gen_to_json_function() -> TokenStream {
    quote! {
        /// Converts a parse tree to a JSON string of nodes and edges, for visualization.
        pub fn to_json(node: ParseTree<'_>) -> String {
            iguana_runtime::parse_tree::to_json(node, LAYOUT_NAME)
        }
    }
}
