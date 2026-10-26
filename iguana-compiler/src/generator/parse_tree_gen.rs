use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    generator::{
        grammar_utils,
        grammar_utils::{nonterminal_type_name, nt_ident, parse_tree_builder_ident, parser_ident},
        id::{NonterminalIds, TerminalIds},
        utils::{alternative_label, is_valid_rust_ident, safe_ident},
    },
    grammar::{
        def::{Alternative, Grammar},
        symbols::{Definition, DefinitionId, Identifier, Nonterminal, Symbol},
        transformations::transform_symbol,
    },
    ids::TerminalId,
    utils::{to_pascal_case, to_snake_case},
};

pub struct ParseTreeGen<'a> {
    grammar: &'a Grammar,
    nonterminal_ids: &'a NonterminalIds,
    terminal_ids: &'a TerminalIds,
    cyclic_nonterminals: FxHashSet<&'a Nonterminal>,
}

/// Formats a rule for both its type declaration and its tree-builder cases.
fn structural_rule(grammar: &Grammar, head: &Nonterminal, alternative: &Alternative) -> String {
    let mut result = alternative
        .symbols
        .iter()
        .filter(|symbol| symbol.is_parse_tree_symbol())
        .map(|symbol| structural_symbol(grammar, symbol.clone()).to_string())
        .collect::<Vec<_>>()
        .join(" ");
    if let Some(label) = &alternative.label {
        result.push_str(&format!(" #{label}"));
    }
    format!("{} = {result}", head.name)
}

/// The grammar symbol behind a parse-tree field or a derived type's name.
/// Calls and derived-rule origins use the same treatment. Conditions, returns,
/// call arguments, bindings, and restrictions add no fields.
fn structural_symbol(grammar: &Grammar, symbol: Symbol) -> Symbol {
    transform_symbol(symbol, &mut |symbol| match symbol {
        Symbol::Binding { symbol, .. }
        | Symbol::Restricted { symbol, .. }
        | Symbol::Exclude { symbol, .. } => *symbol,
        Symbol::Identifier(name) | Symbol::Call { name, .. } => {
            if let Some(id) = name.definition
                && let Definition::Nonterminal(nonterminal) = grammar.definition(id)
                && let Some(origin) = &nonterminal.origin
            {
                structural_symbol(grammar, origin.clone())
            } else {
                Symbol::Identifier(name)
            }
        }
        Symbol::Group(symbols) => Symbol::Group(
            symbols
                .into_iter()
                .filter(Symbol::is_parse_tree_symbol)
                .collect(),
        ),
        other => other,
    })
}

fn structural_name(grammar: &Grammar, nonterminal: &Nonterminal) -> String {
    nonterminal.origin.as_ref().map_or_else(
        || nonterminal.name.clone(),
        |origin| structural_symbol(grammar, origin.clone()).to_string(),
    )
}

impl<'a> ParseTreeGen<'a> {
    pub fn new(
        grammar: &'a Grammar,
        nonterminal_ids: &'a NonterminalIds,
        terminal_ids: &'a TerminalIds,
        cyclic_nonterminals: FxHashSet<&'a Nonterminal>,
    ) -> Self {
        Self {
            grammar,
            nonterminal_ids,
            terminal_ids,
            cyclic_nonterminals,
        }
    }

    fn cycle_arm(&self, nonterminal: &Nonterminal, arm: TokenStream) -> TokenStream {
        if self.cyclic_nonterminals.contains(nonterminal) {
            arm
        } else {
            quote! {}
        }
    }

    fn cycle_panic_arm(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ident = nt_ident(&nonterminal.name);
        let message = format!("{} is a cycle", nonterminal.name);
        self.cycle_arm(
            nonterminal,
            quote! { #ident::Cycle { .. } => panic!(#message), },
        )
    }

    fn cycle_variant(&self, nonterminal: &Nonterminal) -> TokenStream {
        self.cycle_arm(
            nonterminal,
            quote! {
                Cycle { target: CycleTarget<'a, Self> },
            },
        )
    }

    fn cycle_accessor_guard(&self, nonterminal: &Nonterminal) -> TokenStream {
        if self.cyclic_nonterminals.is_empty() {
            quote! {}
        } else {
            let panic_msg = format!("{} is a cycle", nonterminal.name);
            quote! { other if other.is_cycle() => panic!(#panic_msg), }
        }
    }

    /// The parse tree type for a nonterminal, e.g. `Ident<'a>`; see
    /// [`grammar_utils::nonterminal_type`].
    fn nonterminal_type(&self, nonterminal: &Nonterminal) -> TokenStream {
        grammar_utils::nonterminal_type(self.grammar, nonterminal)
    }

    /// The type of a symbol as it appears in parse tree fields; see
    /// [`grammar_utils::symbol_type`].
    fn symbol_type(&self, def_id: DefinitionId) -> TokenStream {
        grammar_utils::symbol_type(self.grammar, def_id)
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
        let opt_node_trait = gen_opt_node_trait();
        let opt_node_impls: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| {
                matches!(&n.origin, Some(Symbol::Opt(_))) && opt_symbol_count(self.grammar, n) == 1
            })
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
        let nonterminal_types: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| self.gen_nonterminal_type(n))
            .collect();

        let nonterminal_types_impl: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| self.gen_nonterminal_type_impl(n))
            .collect();

        let parse_tree_node_impl = self.gen_parse_tree_node_impl();
        let start_aliases = self.gen_start_aliases();

        quote! {
            #imports
            #token_kind_enum
            #token_kind_impl
            #(#start_aliases)*
            #parse_tree_enum
            #parse_tree_impl
            #opt_node_trait
            #(#nonterminal_types)*
            #(#nonterminal_types_impl)*
            #(#opt_node_impls)*
            #(#alt_accessor_impls)*
            #token_struct
            #token_impl
            #token_kind_function
            #parse_tree_builder_impl
            #create_parse_tree_function
            #parse_tree_node_impl
        }
    }

    fn gen_imports(&self) -> TokenStream {
        let parser_name = parser_ident(&self.grammar.name);
        let cycle_target = if self.cyclic_nonterminals.is_empty() {
            quote! {}
        } else {
            quote! { CycleTarget, }
        };
        // Only the typed accessors of `Plus` nonterminals call `list_children`.
        let has_list_accessor = self.grammar.nonterminals().any(|n| {
            matches!(n.origin, Some(Symbol::Plus(..))) && self.gen_typed_accessor(n).is_some()
        });
        let list_children = if has_list_accessor {
            quote! { list_children, }
        } else {
            quote! {}
        };
        quote! {
            use iguana_runtime::{
                arena::Arena,
                ids::{NonterminalId, SPPFNodeId, SlotId, TerminalId},
                input::Span,
                parse_tree::{
                    #cycle_target NodeKind, Origin, ParseTreeBuilder,
                    ParseTreeNode, #list_children visit_sppf,
                },
                sppf::{NonterminalNode, TerminalNode},
            };
            pub use iguana_runtime::parse_tree::Start;
            use crate::{grammar, parser::#parser_name};
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

    /// Returns (name, type) pairs for each parse-tree symbol in an alternative:
    /// every symbol except `Condition` and `Return` (see
    /// [`Symbol::is_parse_tree_symbol`]). Terminals stay in as `Token` fields; a
    /// literal with no readable name becomes `lit_N`. For example, given
    /// `Expr = Expr "+" Expr`, this returns:
    ///   `[(expr_0, &'a Expr<'a>), (lit_1, Token), (expr_2, &'a Expr<'a>)]`
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
                let field_type = self.symbol_type(s.resolved_def());
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
            let display_name = structural_name(self.grammar, nonterminal);
            quote! { #[comment = #display_name] }
        } else {
            let rule = structural_rule(self.grammar, nonterminal, alternative);
            quote! { #[comment = #rule] }
        };
        let nonterminal_name_id = nt_ident(nonterminal_name);
        let alt0_variant = nt_ident(&alternative_label(alternative, 0));
        // Single-alternative nonterminals are enums with `<Alt0> { fields, span }`
        // + `Amb(&[ParseTree])`, mirroring multi-alternative nonterminals. An
        // intermediate-node ambiguity inside the rule body produces multiple
        // derivations of the same span; each materializes into its own `<Alt0>`
        // value, and the alternatives land in `Amb`. The `<Alt0>` variant takes
        // the alternative's label when present, falling back to `Alt0`.
        let cycle_variant = self.cycle_variant(nonterminal);
        quote! {
            #comment
            #[derive(Debug)]
            pub enum #nonterminal_name_id<'a> {
                #alt0_variant { #(#fields,)* span: Span },
                Amb(&'a [ParseTree<'a>]),
                #cycle_variant
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
                let variant_comment = structural_rule(self.grammar, nonterminal, alternative);
                quote! {
                    #[comment = #variant_comment]
                    #variant_name { #(#fields,)* span: Span }
                }
            })
            .collect();
        let nonterminal_name = &nonterminal.name;
        let comment = if nonterminal.is_derived() {
            let display_name = structural_name(self.grammar, nonterminal);
            quote! { #[comment = #display_name] }
        } else {
            quote! {}
        };
        let nonterminal_name_id = nt_ident(nonterminal_name);
        let cycle_variant = self.cycle_variant(nonterminal);
        quote! {
            #comment
            #[derive(Debug)]
            pub enum #nonterminal_name_id<'a> {
                #(#arms,)*
                Amb(&'a [ParseTree<'a>]),
                #cycle_variant
            }
        }
    }

    fn gen_nonterminal_type_impl(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ty = self.nonterminal_type(nonterminal);
        let as_parse_tree_method = self.gen_as_parse_tree_method(nonterminal);
        let child_method = self.gen_child_method(nonterminal);
        let child_count_method = self.gen_child_count_method(nonterminal);
        let span_method = self.gen_span_method(nonterminal);
        let display_name_method = self.gen_nonterminal_display_name_method(nonterminal);
        let origin_method = self.gen_nonterminal_origin_method(nonterminal);
        let typed_accessor = self.gen_typed_accessor(nonterminal);
        let field_accessors = self.gen_field_accessor_methods(nonterminal);
        quote! {
            impl<'a> #ty {
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
    /// panic when called on a non-matching variant.
    ///
    /// A derived Group nonterminal, which has one alternative, also gets these
    /// accessors. For example, with `{("\\" Identifier) ","}+` the Plus's
    /// typed accessor `identifiers()` walks each Group child `r` and calls
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
        let panic_msg = format!("{} is ambiguous", nt_name);
        let cycle_arm = self.cycle_panic_arm(nonterminal);
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
                let field_ty = self.symbol_type(s.resolved_def());
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
                quote! {
                    pub fn #field_ident(&self) -> #field_ty {
                        match self {
                            #ident::#alt0_variant { #field_ident, .. } => #body,
                            #ident::Amb(_) => panic!(#panic_msg),
                            #cycle_arm
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
    /// nonterminal is an enum.
    fn gen_nonterminal_display_name_method(&self, nonterminal: &Nonterminal) -> TokenStream {
        let display_name = nonterminal.display_name();
        let ident = nt_ident(&nonterminal.name);
        let cycle_arm = self.cycle_arm(nonterminal, quote! { #ident::Cycle { .. } => "Cycle", });
        quote! {
            pub fn display_name(&self) -> &'static str {
                match self {
                    #ident::Amb(_) => "Amb",
                    #cycle_arm
                    _ => #display_name,
                }
            }
        }
    }

    /// Generates `origin` on a nonterminal type. Returns `None` for the `Amb`
    /// variant and the nonterminal's origin otherwise. Reporting `None` for `Amb`
    /// keeps the presentation transforms from splicing or dropping an ambiguity
    /// cluster. A user-declared nonterminal has no origin, so its method is a plain
    /// `None` with no dispatch.
    fn gen_nonterminal_origin_method(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ident = nt_ident(&nonterminal.name);
        let cycle_arm = self.cycle_arm(nonterminal, quote! { #ident::Cycle { .. } => None, });
        match origin_kind(nonterminal) {
            Some(kind) => quote! {
                pub fn origin(&self) -> Option<Origin> {
                    match self {
                        #ident::Amb(_) => None,
                        #cycle_arm
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
        let cycle_arm = self.cycle_arm(
            nonterminal,
            quote! {
                #ident::Cycle { target } => target.get().span(),
            },
        );
        quote! {
            pub fn span(&self) -> Span {
                match self {
                    #(#arms,)*
                    #ident::Amb(alts) => alts[0].span(),
                    #cycle_arm
                }
            }
        }
    }

    fn gen_child_method(&self, nonterminal: &Nonterminal) -> TokenStream {
        let children_by_index = self.gen_children_by_index(nonterminal);
        quote! {
            pub(crate) fn child(&self, index: usize) -> Option<ParseTree<'a>> {
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
        let cycle_arm = self.cycle_arm(nonterminal, quote! { #ident::Cycle { .. } => None, });
        quote! {
            match self {
                #(#arms,)*
                #ident::Amb(alts) => alts.get(index).copied(),
                #cycle_arm
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
        let cycle_arm = self.cycle_arm(nonterminal, quote! { #ident::Cycle { .. } => 0, });
        quote! {
            pub(crate) fn child_count(&self) -> usize {
                match self {
                    #(#arms,)*
                    #ident::Amb(alts) => alts.len(),
                    #cycle_arm
                }
            }
        }
    }

    fn gen_as_parse_tree_method(&self, nonterminal: &Nonterminal) -> TokenStream {
        let name_ident = nt_ident(&nonterminal.name);
        quote! {
            pub fn as_parse_tree(&'a self) -> ParseTree<'a> {
                ParseTree::#name_ident(self)
            }
        }
    }

    fn gen_parse_tree_builder_impl(&self) -> TokenStream {
        let builder_name_ident = parse_tree_builder_ident(&self.grammar.name);
        let nonterminal_node_method = self.gen_nonterminal_node_method();
        let new_token_method = gen_new_token_method();
        let (ambiguity_node_methods, new_ambiguity_node_method) = self.gen_ambiguity_node_methods();
        let cycle_methods = self.gen_builder_cycle_methods();
        quote! {
            pub struct #builder_name_ident<'a> {
                pub arena: &'a Arena,
            }
            impl<'a> #builder_name_ident<'a> {
                pub fn new(tree_arena: &'a Arena) -> Self {
                    Self { arena: tree_arena }
                }
                #ambiguity_node_methods
            }
            impl<'a> ParseTreeBuilder<ParseTree<'a>> for #builder_name_ident<'a> {
                #nonterminal_node_method
                #new_token_method
                #new_ambiguity_node_method
                #cycle_methods
            }
        }
    }

    fn gen_builder_cycle_methods(&self) -> TokenStream {
        if self.cyclic_nonterminals.is_empty() {
            return quote! {};
        }
        let mut new_arms = Vec::new();
        let mut set_arms = Vec::new();
        for nt in self
            .grammar
            .nonterminals()
            .filter(|nt| self.cyclic_nonterminals.contains(nt))
        {
            let variant = nt_ident(&nt.name);
            let id = format_ident!("{}", to_snake_case(&nt.name).to_uppercase());
            new_arms.push(quote! {
                grammar::#id => ParseTree::#variant(self.arena.alloc(
                    #variant::Cycle { target: CycleTarget::new() }
                )),
            });
            set_arms.push(quote! {
                (ParseTree::#variant(#variant::Cycle { target: cell }), ParseTree::#variant(target)) => {
                    cell.set(target);
                }
            });
        }
        quote! {
            fn new_cycle_node(&self, node: &NonterminalNode) -> ParseTree<'a> {
                match node.nonterminal_id {
                    #(#new_arms)*
                    _ => unreachable!("nonterminal cannot be cyclic"),
                }
            }

            fn set_cycle_target(&self, cycle: &ParseTree<'a>, target: &ParseTree<'a>) {
                match (cycle, target) {
                    #(#set_arms)*
                    _ => unreachable!("cycle target has a different nonterminal type"),
                }
            }
        }
    }

    /// Generates the builder's `ambiguity_node`, which dispatches on the parent
    /// nonterminal id, and the trait's `new_ambiguity_node`, which calls it.
    /// Every generated nonterminal type is an enum with an `Amb` variant, so
    /// every nonterminal can be the parent of an intermediate-node ambiguity.
    ///
    /// A start nonterminal can also be the parent of an ambiguity node. A
    /// data-dependent inner nonterminal pops its derivations keyed by return
    /// value: derivations with different return values are distinct nodes of
    /// the inner nonterminal and cannot merge there. The start node above
    /// them is the lowest node such derivations have in common, and the
    /// ambiguity is recorded on it. `Start` does not have an `Amb` variant, so
    /// `start_ambiguity_node` builds the ambiguity node of the inner
    /// nonterminal from the derivations' inner nodes and wraps one `Start`
    /// around it. The layout and the span come from the first derivation;
    /// every derivation shares the start node's span.
    fn gen_ambiguity_node_methods(&self) -> (TokenStream, TokenStream) {
        let cases: Vec<TokenStream> = self
            .grammar
            .nonterminals()
            .map(|n| {
                let const_name = format_ident!("{}", to_snake_case(&n.name).to_uppercase());
                if self.grammar.is_start(n) {
                    let inner_ident = n.origin.as_ref().unwrap().as_identifier().unwrap();
                    let inner_const =
                        format_ident!("{}", to_snake_case(&inner_ident.name).to_uppercase());
                    quote! {
                        grammar::#const_name => self.start_ambiguity_node(grammar::#inner_const, alternatives),
                    }
                } else {
                    let variant = nt_ident(&n.name);
                    quote! {
                        grammar::#const_name => ParseTree::#variant(self.arena.alloc(#variant::Amb(alternatives))),
                    }
                }
            })
            .collect();
        if cases.is_empty() {
            return (quote! {}, quote! {});
        }
        let builder_methods = quote! {
            fn ambiguity_node(
                &self,
                parent: NonterminalId,
                alternatives: &'a [ParseTree<'a>],
            ) -> ParseTree<'a> {
                match parent {
                    #(#cases)*
                    _ => unreachable!("nonterminal cannot be ambiguous"),
                }
            }

            fn start_ambiguity_node(
                &self,
                inner: NonterminalId,
                alternatives: &'a [ParseTree<'a>],
            ) -> ParseTree<'a> {
                let first = alternatives[0].unwrap_start();
                let inner_alternatives = self
                    .arena
                    .alloc_slice(alternatives.iter().map(|a| a.unwrap_start().node));
                ParseTree::Start(self.arena.alloc(Start {
                    before: first.before,
                    node: self.ambiguity_node(inner, inner_alternatives),
                    after: first.after,
                    span: first.span,
                }))
            }
        };
        let trait_method = quote! {
            fn new_ambiguity_node(
                &self,
                parent: NonterminalId,
                alternatives: Vec<ParseTree<'a>>,
            ) -> ParseTree<'a> {
                self.ambiguity_node(parent, self.arena.alloc_slice(alternatives))
            }
        };
        (builder_methods, trait_method)
    }

    fn gen_nonterminal_node_method(&self) -> TokenStream {
        let mut nonterminal_cases: Vec<TokenStream> = self
            .nonterminal_ids
            .ids()
            .filter(|id| !self.grammar.is_start(self.nonterminal_ids.get_nonterminal(*id)))
            .flat_map(|nonterminal_id| {
                let nonterminal = self.nonterminal_ids.get_nonterminal(nonterminal_id);
                let slot_cases: Vec<TokenStream> = self
                    .nonterminal_ids
                    .end_slots(nonterminal_id)
                    .map(|end_slot| {
                        let index = end_slot.index;
                        let alternatives = self.grammar.alternatives(nonterminal);
                        let alternative = &alternatives[index];
                        let end_slot_id = end_slot.slot_id;
                        let rule_comment = structural_rule(self.grammar, nonterminal, alternative);
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
                        let variant = Ident::new(
                            &to_pascal_case(&alternative_label(alternative, index)),
                            Span::call_site()
                        );
                        quote! {
                            #[comment = #rule_comment]
                            #end_slot_id => {
                                let &[#(#field_names),*] = children else { ParseTree::unexpected_node() };
                                ParseTree::#nonterminal_type(self.arena.alloc(#nonterminal_type::#variant {
                                    #(#field_names: #method_calls,)*
                                    span: nonterminal_node.span,
                                }))
                            }
                        }
                    })
                    .collect();
                slot_cases
            })
            .collect();
        // All start nonterminals share two arms, one with layout and one
        // without.
        let (with_layout, without_layout): (Vec<_>, Vec<_>) = self
            .nonterminal_ids
            .ids()
            .map(|id| (id, self.nonterminal_ids.get_nonterminal(id)))
            .filter(|(_, nonterminal)| self.grammar.is_start(nonterminal))
            .flat_map(|(id, nonterminal)| {
                let has_layout = self.grammar.start_layout(nonterminal).is_some();
                self.nonterminal_ids
                    .end_slots(id)
                    .map(move |end_slot| (end_slot.slot_id, has_layout))
            })
            .partition(|(_, has_layout)| *has_layout);
        if !with_layout.is_empty() {
            let slots = with_layout.iter().map(|(slot, _)| slot);
            nonterminal_cases.push(quote! {
                #[comment = "The start nonterminals with layout"]
                #(#slots)|* => {
                    let &[before, node, after] = children else { ParseTree::unexpected_node() };
                    ParseTree::Start(self.arena.alloc(Start {
                        before: Some(before),
                        node,
                        after: Some(after),
                        span: nonterminal_node.span,
                    }))
                }
            });
        }
        if !without_layout.is_empty() {
            let slots = without_layout.iter().map(|(slot, _)| slot);
            nonterminal_cases.push(quote! {
                #[comment = "The start nonterminals without layout"]
                #(#slots)|* => {
                    let &[node] = children else { ParseTree::unexpected_node() };
                    ParseTree::Start(self.arena.alloc(Start {
                        before: None,
                        node,
                        after: None,
                        span: nonterminal_node.span,
                    }))
                }
            });
        }
        // Slot ids are unique across the grammar, so the end slot alone
        // selects the alternative, in one jump table.
        quote! {
            fn new_nonterminal_node(
                &self,
                nonterminal_node: &NonterminalNode,
                children: &[ParseTree<'a>]
            ) -> ParseTree<'a> {
                match nonterminal_node.return_slot {
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
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| {
                let variant = nt_ident(&n.name);
                let ty = self.nonterminal_type(n);
                if n.is_derived() {
                    let display_name = structural_name(self.grammar, n);
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
                #[comment = "Every start nonterminal. `before` and `after` are `None` without layout."]
                Start(&'a Start<ParseTree<'a>, Option<ParseTree<'a>>>),
                Token(Token)
            }
        }
    }

    fn gen_parse_tree_impl(&self) -> TokenStream {
        let unwrap_methods = self.gen_unwrap_methods();
        let name_method = self.gen_parse_tree_name_method();
        let child_method = self.gen_parse_tree_child_method();
        let child_count_method = self.gen_parse_tree_child_count_method();
        let span_method = self.gen_parse_tree_span_method();
        let is_amb_method = self.gen_parse_tree_is_amb_method();
        let node_id_method = self.gen_parse_tree_node_id_method();
        let nonterminal_id_method = self.gen_parse_tree_nonterminal_id_method();
        let origin_method = self.gen_parse_tree_origin_method();
        let cycle_methods = self.gen_parse_tree_cycle_methods();
        quote! {
            impl<'a> ParseTree<'a> {
                #name_method
                #child_method
                #child_count_method
                #span_method
                #is_amb_method
                #node_id_method
                #nonterminal_id_method
                #origin_method
                #cycle_methods
                #(#unwrap_methods)*
                pub(crate) fn unwrap_start(self) -> &'a Start<ParseTree<'a>, Option<ParseTree<'a>>> {
                    match self {
                        ParseTree::Start(start) => start,
                        _ => Self::unexpected_node(),
                    }
                }
                pub(crate) fn unwrap_token(self) -> Token {
                    match self {
                        ParseTree::Token(t) => t,
                        _ => Self::unexpected_node(),
                    }
                }
                #[doc = "The shared failure of the `unwrap_*` methods. One cold function keeps"]
                #[doc = "the builder from carrying a separate panic site per child."]
                #[cold]
                #[inline(never)]
                fn unexpected_node() -> ! {
                    unreachable!("unexpected parse tree node")
                }
            }
        }
    }

    fn gen_parse_tree_cycle_methods(&self) -> TokenStream {
        if self.cyclic_nonterminals.is_empty() {
            return quote! {};
        }
        let variants: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|nt| self.cyclic_nonterminals.contains(nt))
            .map(|nt| nt_ident(&nt.name))
            .collect();
        quote! {
            #[doc = "Whether this node is a cyclic reference (`Cycle`)."]
            pub fn is_cycle(&self) -> bool {
                matches!(self, #(ParseTree::#variants(#variants::Cycle { .. }))|*)
            }

            #[doc = "The target of a `Cycle` node as a `ParseTree`, or `None` for any other node."]
            #[doc = "The target is not among `children()`."]
            pub fn cycle_target(&self) -> Option<Self> {
                match self {
                    #(ParseTree::#variants(#variants::Cycle { target }) => Some(ParseTree::#variants(
                        target.get()
                    )),)*
                    _ => None,
                }
            }
        }
    }

    fn gen_unwrap_methods(&self) -> Vec<TokenStream> {
        self.grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| {
                let method_ident = format_ident!("unwrap_{}", to_snake_case(&n.name));
                let variant = nt_ident(&n.name);
                let var = safe_ident(&to_snake_case(&n.name));
                let return_type = self.nonterminal_type(n);
                quote! {
                    pub(crate) fn #method_ident(self) -> &'a #return_type {
                        match self {
                            ParseTree::#variant(#var) => #var,
                            _ => Self::unexpected_node(),
                        }
                    }
                }
            })
            .collect()
    }

    fn gen_parse_tree_child_method(&self) -> TokenStream {
        let arms: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| {
                let variant = nt_ident(&n.name);
                let var_ident = safe_ident(&to_snake_case(&n.name));
                quote! {
                    ParseTree::#variant(#var_ident) => #var_ident.child(index)
                }
            })
            .collect();
        quote! {
            pub(crate) fn child(&self, index: usize) -> Option<ParseTree<'a>> {
                match self {
                    #(#arms,)*
                    ParseTree::Start(start) => start.child(index),
                    ParseTree::Token(_) => None,
                }
            }
        }
    }

    fn gen_parse_tree_name_method(&self) -> TokenStream {
        let arms = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| {
                let name_ident = nt_ident(&n.name);
                let var_ident = safe_ident(&to_snake_case(&n.name));
                quote! { ParseTree::#name_ident(#var_ident) => #var_ident.display_name() }
            });
        quote! {
            pub fn display_name(&self) -> &'static str {
                match self {
                    #(#arms,)*
                    ParseTree::Start(start) => start.display_name(),
                    ParseTree::Token(token) => token.kind.name(),
                }
            }
        }
    }

    fn gen_parse_tree_child_count_method(&self) -> TokenStream {
        let arms: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| {
                let variant = nt_ident(&n.name);
                let var_ident = safe_ident(&to_snake_case(&n.name));
                quote! {
                    ParseTree::#variant(#var_ident) => #var_ident.child_count()
                }
            })
            .collect();
        quote! {
            pub(crate) fn child_count(&self) -> usize {
                match self {
                    #(#arms,)*
                    ParseTree::Start(start) => start.child_count(),
                    ParseTree::Token(_) => 0,
                }
            }
        }
    }

    fn gen_parse_tree_span_method(&self) -> TokenStream {
        let arms: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
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
                    ParseTree::Start(start) => start.span(),
                    ParseTree::Token(token) => token.span(),
                }
            }
        }
    }

    fn gen_parse_tree_is_amb_method(&self) -> TokenStream {
        let arms = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| {
                let variant = nt_ident(&n.name);
                let var_ident = safe_ident(&to_snake_case(&n.name));
                quote! { ParseTree::#variant(#var_ident) => matches!(#var_ident, #variant::Amb(_)) }
            });
        // `Start` does not have an `Amb` variant. The ambiguity of a start
        // nonterminal is an `Amb` of the node it wraps.
        quote! {
            #[doc = "True when this node is an ambiguity cluster (any `*::Amb` variant). The"]
            #[doc = "uniform way to detect ambiguity without matching each nonterminal's enum."]
            pub fn is_amb(&self) -> bool {
                match self {
                    #(#arms,)*
                    ParseTree::Start(_) | ParseTree::Token(_) => false,
                }
            }
        }
    }

    fn gen_parse_tree_node_id_method(&self) -> TokenStream {
        let arms = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| {
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
                    ParseTree::Start(start) => Some(*start as *const _ as usize),
                    ParseTree::Token(_) => None,
                }
            }
        }
    }

    fn gen_parse_tree_nonterminal_id_method(&self) -> TokenStream {
        let arms = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| {
                let variant = nt_ident(&n.name);
                let id = format_ident!("{}", to_snake_case(&n.name).to_uppercase());
                quote! { ParseTree::#variant(_) => Some(grammar::#id) }
            });
        quote! {
            pub fn nonterminal_id(&self) -> Option<NonterminalId> {
                match self {
                    #(#arms,)*
                    ParseTree::Start(_) | ParseTree::Token(_) => None,
                }
            }
        }
    }

    fn gen_parse_tree_origin_method(&self) -> TokenStream {
        let arms = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| {
                let variant = nt_ident(&n.name);
                let var_ident = safe_ident(&to_snake_case(&n.name));
                quote! { ParseTree::#variant(#var_ident) => #var_ident.origin() }
            });
        quote! {
            pub fn origin(&self) -> Option<Origin> {
                match self {
                    #(#arms,)*
                    ParseTree::Start(start) => start.origin(),
                    ParseTree::Token(_) => None,
                }
            }
        }
    }

    fn gen_opt_node_impl(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ty = self.nonterminal_type(nonterminal);
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
        let cycle_arm = self.cycle_panic_arm(nonterminal);

        quote! {
            impl<'a> OptNode for #ty {
                type Inner = #inner_type;
                fn value(&self) -> Option<&Self::Inner> {
                    match self {
                        #opt_type::Alt0 { #field_name, .. } => Some(#field_name),
                        #opt_type::Alt1 { .. } => None,
                        #opt_type::Amb(_) => panic!("unexpected ambiguity in optional node"),
                        #cycle_arm
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
                        (method, self.nonterminal_type(nt))
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
        quote! {
            impl<'a> #ty {
                #(#accessors)*
            }
        }
    }

    /// Generates a typed accessor method for Plus/Star/Opt nonterminals.
    ///
    /// These accessors provide a convenient way to iterate over elements
    /// without manually navigating through wrapper types. A list accessor's
    /// name is the pluralized snake_case form of the child element type.
    ///
    /// # Generated Accessors
    ///
    /// For `("," Item)+` (Plus whose element is a sequence with one named
    /// symbol, see `gen_sequence_element_accessor`):
    /// ```ignore
    /// pub fn items(&self) -> impl Iterator<Item = &Item> { ... }
    /// ```
    ///
    /// For `("else" Stmt)?` (Opt with several symbols), one accessor per
    /// field (see `gen_opt_field_accessors`):
    /// ```ignore
    /// pub fn stmt(&self) -> Option<&Stmt> { ... }
    /// ```
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
    /// For `Symbol*` (Star), which delegates to the inner Plus's accessor:
    /// ```ignore
    /// pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
    ///     let plus = match self {
    ///         Star0::Alt0 { symbols, .. } => Some(*symbols),
    ///         Star0::Alt1 { .. } => None,
    ///         // The Amb and Cycle arms panic.
    ///     };
    ///     plus.into_iter().flat_map(|plus| plus.symbols())
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
    ///     list_children(self.as_parse_tree()).into_iter().filter_map(|node| match node {
    ///         ParseTree::RegexPlus(r) => Some(r.regexes()),
    ///         _ => None,
    ///     })
    /// }
    /// ```
    fn gen_typed_accessor(&self, nonterminal: &Nonterminal) -> Option<TokenStream> {
        match &nonterminal.origin {
            Some(list @ Symbol::Plus(inner, None))
                if matches!(inner.as_ref(), Symbol::Group(_)) =>
            {
                self.gen_sequence_element_accessor(nonterminal, list)
            }
            Some(list @ Symbol::Plus(inner, _)) => {
                let element_types = list_element_types(self.grammar, list);
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
                // silently drop it and return a short list. Ambiguity-aware callers use
                // `children()` with `is_amb()` instead.
                let panic_msg = format!("{} is ambiguous", nonterminal.name);
                let amb_arm = quote! { other if other.is_amb() => panic!(#panic_msg), };
                let cycle_arm = self.cycle_accessor_guard(nonterminal);

                let return_type = self.gen_accessor_return_type(nonterminal, innermost);
                if child_name == innermost.name {
                    // Simple case: e.g., `Regex+` or `Identifier+`
                    Some(quote! {
                        pub fn #method_name(&'a self) -> #return_type {
                            list_children(self.as_parse_tree())
                                .into_iter()
                                .filter_map(|node| match node {
                                    ParseTree::#innermost_type(r) => Some(r),
                                    #amb_arm
                                    #cycle_arm
                                    _ => None,
                                })
                        }
                    })
                } else if let Symbol::Group(_) = inner.as_ref() {
                    // Group case: the element of a separated list, e.g.,
                    // `{("!" Identifier) ","}+`
                    let field_name = safe_ident(&to_snake_case(&innermost.name));
                    Some(quote! {
                        pub fn #method_name(&'a self) -> #return_type {
                            list_children(self.as_parse_tree())
                                .into_iter()
                                .filter_map(|node| match node {
                                    ParseTree::#child_type(r) => Some(r.#field_name()),
                                    #amb_arm
                                    #cycle_arm
                                    _ => None,
                                })
                        }
                    })
                } else {
                    // Nested case: e.g., `{Regex+ "|"}+` where child is a nested Plus/Star type.
                    Some(quote! {
                        pub fn #method_name(&'a self) -> #return_type {
                            list_children(self.as_parse_tree())
                                .into_iter()
                                .filter_map(|node| match node {
                                    ParseTree::#child_type(r) => Some(r.#method_name()),
                                    #amb_arm
                                    #cycle_arm
                                    _ => None,
                                })
                        }
                    })
                }
            }
            Some(list @ Symbol::Star(..)) => {
                let element_types = list_element_types(self.grammar, list);
                if element_types.is_empty() {
                    return None;
                }
                // Star is `Alt0 { plus_field, span } | Alt1 { span }`: a list or empty.
                let alternatives = self.grammar.alternatives(nonterminal);
                let plus_symbol = alternatives[0].symbols.first()?;
                let plus_field_name =
                    safe_ident(&gen_field_name(self.grammar, plus_symbol, 0, false));
                let ident = nt_ident(&nonterminal.name);
                let alt0_variant = nt_ident(&alternative_label(&alternatives[0], 0));
                let alt1_variant = nt_ident(&alternative_label(&alternatives[1], 1));
                let panic_msg = format!("{} is ambiguous", nonterminal.name);
                let cycle_arm = self.cycle_panic_arm(nonterminal);

                let methods: Vec<_> = element_types
                    .iter()
                    .map(|elem| {
                        let method_name = safe_ident(&pluralize(&to_snake_case(&elem.name)));
                        let return_type = self.gen_accessor_return_type(nonterminal, elem);
                        quote! {
                            pub fn #method_name(&self) -> #return_type {
                                let plus = match self {
                                    #ident::#alt0_variant { #plus_field_name, .. } => Some(*#plus_field_name),
                                    #ident::#alt1_variant { .. } => None,
                                    #ident::Amb(_) => panic!(#panic_msg),
                                    #cycle_arm
                                };
                                plus.into_iter().flat_map(|plus| plus.#method_name())
                            }
                        }
                    })
                    .collect();
                Some(quote! { #(#methods)* })
            }
            Some(Symbol::Opt(_)) if opt_symbol_count(self.grammar, nonterminal) > 1 => {
                Some(self.gen_opt_field_accessors(nonterminal))
            }
            Some(Symbol::Opt(inner)) => {
                // Opt types that wrap Plus/Star (e.g., `SyntaxRule+?`).
                // Delegate to the inner Plus/Star's accessor via OptNode::value().
                let list = ungroup_single(inner);
                if !list.is_list() {
                    return None;
                }
                let element_types = list_element_types(self.grammar, list);
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
                // An accessor that searches the fields would pick the wrong token
                // when the Group has multiple positional `Token` fields (e.g. for
                // `("\\" Identifier)`, find_map returns the `"\\"`).
                None
            }
            _ => self.gen_delegated_accessors(nonterminal),
        }
    }

    /// Generates the accessor of a Plus whose element is a sequence with one
    /// named symbol: `("," Item)+` gets `items()`, which returns the `Item`
    /// children in order. When the named symbol is a terminal, as in
    /// `("," Name)+`, the accessor selects the tokens whose kind is `Name`,
    /// since the `","` children are tokens too. The accessor panics on an Amb
    /// or Cycle node of the list, as the other list accessors do.
    fn gen_sequence_element_accessor(
        &self,
        nonterminal: &Nonterminal,
        list: &Symbol,
    ) -> Option<TokenStream> {
        let element_types = list_element_types(self.grammar, list);
        if element_types.is_empty() {
            return None;
        }
        let list_variant = nt_ident(&nonterminal.name);
        let amb_msg = format!("{} is ambiguous", nonterminal.name);
        let cycle_msg = format!("{} is a cycle", nonterminal.name);
        let cycle_arm = self.cycle_arm(
            nonterminal,
            quote! { other @ ParseTree::#list_variant(_) if other.is_cycle() => panic!(#cycle_msg), },
        );
        let methods = element_types.iter().map(|elem| {
            let method_name = safe_ident(&pluralize(&to_snake_case(&elem.name)));
            let return_type = self.gen_accessor_return_type(nonterminal, elem);
            let element_arm = match self.grammar.definition(elem.resolve()) {
                Definition::Terminal(terminal) => {
                    let kind = format_ident!("T{}", self.terminal_ids.get_id(terminal).0);
                    quote! {
                        ParseTree::Token(r) if matches!(r.kind, TokenKind::#kind) => Some(r),
                    }
                }
                Definition::Nonterminal(_) => {
                    let variant = symbol_ident(self.grammar, elem);
                    quote! { ParseTree::#variant(r) => Some(r), }
                }
            };
            quote! {
                pub fn #method_name(&'a self) -> #return_type {
                    list_children(self.as_parse_tree())
                        .into_iter()
                        .filter_map(|node| match node {
                            #element_arm
                            other @ ParseTree::#list_variant(_) if other.is_amb() => panic!(#amb_msg),
                            #cycle_arm
                            _ => None,
                        })
                }
            }
        });
        Some(quote! { #(#methods)* })
    }

    /// Generates one accessor per field of an optional with several symbols,
    /// such as `("else" Stmt)?`. The accessor of a field returns the field when
    /// the optional is present and `None` when it is empty. An optional with one
    /// symbol implements `OptNode` instead.
    fn gen_opt_field_accessors(&self, nonterminal: &Nonterminal) -> TokenStream {
        let ident = nt_ident(&nonterminal.name);
        let alternatives = self.grammar.alternatives(nonterminal);
        let present = nt_ident(&alternative_label(&alternatives[0], 0));
        let empty = nt_ident(&alternative_label(&alternatives[1], 1));
        let panic_msg = format!("{} is ambiguous", nonterminal.name);
        let cycle_arm = self.cycle_panic_arm(nonterminal);
        let methods = self
            .gen_fields_for_alternative_symbols(&alternatives[0])
            .into_iter()
            .map(|(field, ty)| {
                quote! {
                    pub fn #field(&self) -> Option<#ty> {
                        match self {
                            #ident::#present { #field, .. } => Some(*#field),
                            #ident::#empty { .. } => None,
                            #ident::Amb(_) => panic!(#panic_msg),
                            #cycle_arm
                        }
                    }
                }
            });
        quote! { #(#methods)* }
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
        let identifier = symbol.as_identifier()?;
        let child_nt = match self.grammar.definition(identifier.resolve()) {
            Definition::Nonterminal(nt) => nt,
            _ => return None,
        };
        let list = match child_nt.origin.as_ref() {
            Some(list @ (Symbol::Plus(..) | Symbol::Star(..))) => list,
            Some(Symbol::Opt(s)) => match ungroup_single(s) {
                list @ (Symbol::Plus(..) | Symbol::Star(..)) => list,
                _ => return None,
            },
            _ => return None,
        };
        let field_name = safe_ident(&gen_field_name(self.grammar, symbol, 0, false));
        let ident = nt_ident(&nonterminal.name);
        let alt0_variant = nt_ident(&alternative_label(&alternatives[0], 0));
        let panic_msg = format!("{} is ambiguous", nonterminal.name);
        let cycle_arm = self.cycle_panic_arm(nonterminal);
        let element_types = list_element_types(self.grammar, list);
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
                            #ident::Amb(_) => panic!(#panic_msg),
                            #cycle_arm
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
    //   - sequence (`("," Expr)+`): `impl Iterator<Item = &Expr>`
    // - Star (`Expr*`): desugared as `Expr* = Expr+ | ()`, so the return type
    //   is determined by the inner Plus
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
    /// that filter the list's children for each Alt variant.
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
        let panic_msg = format!("{} is ambiguous", nonterminal.name);
        let cycle_arm = self.cycle_accessor_guard(nonterminal);

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
                quote! {
                    pub fn #method_name(&'a self) -> #return_type {
                        list_children(self.as_parse_tree())
                            .into_iter()
                            .filter_map(|node| match node {
                                ParseTree::#child_type(#child_type::#variant_name { #field_name, .. }) => Some(*#field_name),
                                other if other.is_amb() => panic!(#panic_msg),
                                #cycle_arm
                                _ => None,
                            })
                    }
                }
            })
            .collect();

        Some(quote! { #(#methods)* })
    }

    fn gen_create_parse_tree_function(&self) -> TokenStream {
        let parser_name_ident = parser_ident(&self.grammar.name);
        let builder_name_ident = parse_tree_builder_ident(&self.grammar.name);
        let mut arms: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| !self.grammar.is_start(n))
            .map(|n| {
                let name = &n.name;
                let const_name = format_ident!("{}", to_snake_case(name).to_uppercase());
                let unwrap_method = format_ident!("unwrap_{}", to_snake_case(name));
                let variant_name = nt_ident(name);
                quote! {
                    grammar::#const_name => ParseTree::#variant_name(
                        visit_sppf(root_id, parser, builder).#unwrap_method(),
                    )
                }
            })
            .collect();
        let start_consts: Vec<_> = self
            .grammar
            .nonterminals()
            .filter(|n| self.grammar.is_start(n))
            .map(|n| format_ident!("{}", to_snake_case(&n.name).to_uppercase()))
            .collect();
        if !start_consts.is_empty() {
            arms.push(quote! {
                #(grammar::#start_consts)|* => ParseTree::Start(
                    visit_sppf(root_id, parser, builder).unwrap_start(),
                )
            });
        }
        quote! {
            #[comment = "The parse tree of a finished parse, from the nonterminal node that spans the input."]
            pub fn create_parse_tree<'a>(
                root_id: SPPFNodeId,
                parser: &#parser_name_ident,
                builder: &#builder_name_ident<'a>,
            ) -> ParseTree<'a> {
                let iguana_runtime::sppf::SPPFNode::Nonterminal(root) =
                    iguana_runtime::parser::Parser::sppf_node(parser, root_id)
                else {
                    panic!("the SPPF root is not a nonterminal node")
                };
                match root.nonterminal_id {
                    #(#arms,)*
                    _ => panic!()
                }
            }
        }
    }

    /// One type alias per start wrapper, `StartX<'a>`, naming the wrapper
    /// type the typed entry point for `X` returns.
    fn gen_start_aliases(&self) -> Vec<TokenStream> {
        self.grammar
            .nonterminals()
            .filter(|nt| !nt.is_derived())
            .filter_map(|nt| self.grammar.start_nonterminal(nt))
            .map(|start| {
                let alias = nt_ident(&start.name);
                let ty = self.nonterminal_type(start);
                quote! { pub type #alias<'a> = #ty; }
            })
            .collect()
    }

    fn gen_parse_tree_node_impl(&self) -> TokenStream {
        let (cycle_check, cycle_target) = if self.cyclic_nonterminals.is_empty() {
            (quote! {}, quote! {})
        } else {
            (
                quote! { if self.is_cycle() { NodeKind::Cycle } else },
                quote! {
                    fn cycle_target(&self) -> Option<Self> {
                        ParseTree::cycle_target(self)
                    }
                },
            )
        };
        quote! {
            impl<'a> ParseTreeNode for ParseTree<'a> {
                fn child(&self, index: usize) -> Option<Self> {
                    ParseTree::child(self, index)
                }

                fn child_count(&self) -> usize {
                    ParseTree::child_count(self)
                }

                fn display_name(&self) -> &'static str {
                    ParseTree::display_name(self)
                }

                fn span(&self) -> Span {
                    ParseTree::span(self)
                }

                fn kind(&self) -> NodeKind {
                    if self.is_amb() { NodeKind::Amb } else #cycle_check if matches!(self, ParseTree::Token(_)) {
                        NodeKind::Token
                    } else {
                        NodeKind::Nonterminal
                    }
                }

                fn node_id(&self) -> Option<usize> {
                    ParseTree::node_id(self)
                }

                fn nonterminal_id(&self) -> Option<NonterminalId> {
                    ParseTree::nonterminal_id(self)
                }

                #cycle_target

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
        Symbol::Restricted { symbol, .. } => get_symbol_base_name(grammar, symbol),
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
        Symbol::Restricted { symbol, .. } => gen_field_name(grammar, symbol, position, needs_index),
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
        Some(Symbol::Star(_, _)) => Some(quote! { Origin::Star }),
        Some(Symbol::Plus(_, _)) => Some(quote! { Origin::Plus }),
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

/// Returns the named types of a list element, given the element symbol.
/// Literal terminals (e.g., `"!"`, `"|"`) are not named types, since typed
/// accessors do not return them.
///
/// Returns one type for a single symbol, a group with one named symbol, or a
/// nested list with one element type, and one type per alternative for an
/// alternation:
/// - `A` -> `[A]`
/// - `("!" A)`, the element of `{("!" A) ","}+` -> `[A]`
/// - `A+` -> `[A]`
/// - `(A | B)` -> `[A, B]`
///
/// Multiple types occur only for `Symbol::Alt`. For alternations,
/// only alternatives that are simple identifiers are included; nested EBNF
/// alternatives like `A*` in `(A* | C)` are skipped since the desugared
/// nonterminal type would not match. Returns an empty vec when no named
/// types are found (e.g., only literals).
fn element_types(grammar: &Grammar, symbol: &Symbol) -> Vec<Identifier> {
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
        // A nested list contributes an element type only when the nested list
        // has exactly one element type. The accessor of the outer list calls the accessor of
        // the nested list with the same name, and a nested list with several
        // element types does not have a single accessor to call.
        list @ (Symbol::Plus(..) | Symbol::Star(..)) => {
            let types = list_element_types(grammar, list);
            if types.len() == 1 { types } else { vec![] }
        }
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
                .flat_map(|elem| element_types(grammar, elem))
                .collect();
            // A group has a flat element type only when its named element is not
            // itself a list. For `(Annotation* "[" "]")` the named element is a
            // list, whose Group field accessor returns the inner list node rather
            // than the element, so there is no flat type and callers fall back to
            // children().
            let names_a_list = elements
                .iter()
                .any(|elem| elem.is_list() && !element_types(grammar, elem).is_empty());
            if named.len() == 1 && !names_a_list {
                named
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

/// Returns the named element types of a list symbol, `Plus` or `Star`:
/// - `A+` -> `[A]`
/// - `{A ","}+` -> `[A]`
/// - `(A | B)+` -> `[A, B]`
/// - `("," A)+` -> `[A]`
/// - `(Comma A)+` -> `[]`
///
/// A list whose element is a sequence, such as `("," A)+`, holds the symbols
/// of the sequence directly and has an element type only when the sequence
/// has one named symbol (`sequence_element_type`). The element of a
/// separated list keeps its group, so `{(A B) ","}+` takes the element types
/// of the group.
fn list_element_types(grammar: &Grammar, list: &Symbol) -> Vec<Identifier> {
    match list {
        Symbol::Plus(element, None) | Symbol::Star(element, None) => match element.as_ref() {
            Symbol::Group(symbols) => sequence_element_type(grammar, symbols)
                .into_iter()
                .collect(),
            element => element_types(grammar, element),
        },
        Symbol::Plus(element, Some(_)) | Symbol::Star(element, Some(_)) => {
            element_types(grammar, element)
        }
        _ => vec![],
    }
}

/// Returns the type of the one named symbol of a sequence, as for the element
/// of `("," A)+`. A sequence has a named symbol when it has exactly one
/// parse-tree symbol that is not a literal terminal, and that symbol refers to
/// a rule. A sequence with several such symbols, as in `(Comma A)+` or
/// `(A B?)+`, does not have one, and neither does a sequence whose one such
/// symbol becomes a derived nonterminal, as in `("," A?)+`.
fn sequence_element_type(grammar: &Grammar, symbols: &[Symbol]) -> Option<Identifier> {
    let is_literal = |symbol: &Symbol| {
        symbol.as_identifier().is_some_and(|ident| {
            matches!(grammar.definition(ident.resolve()), Definition::Terminal(t) if t.is_literal())
        })
    };
    let named: Vec<_> = symbols
        .iter()
        .filter(|s| s.is_parse_tree_symbol() && !is_literal(s))
        .collect();
    match named[..] {
        [symbol] => symbol.as_identifier().cloned(),
        _ => None,
    }
}

/// The symbol of a one-symbol group, or the symbol itself. The operand of
/// `(A+)?` is the list `A+`, as it is for `A+?`.
fn ungroup_single(symbol: &Symbol) -> &Symbol {
    match symbol {
        Symbol::Group(symbols) if symbols.len() == 1 => &symbols[0],
        symbol => symbol,
    }
}

/// The number of parse-tree symbols of an optional when it is present.
fn opt_symbol_count(grammar: &Grammar, nonterminal: &Nonterminal) -> usize {
    grammar.alternatives(nonterminal)[0]
        .symbols
        .iter()
        .filter(|s| s.is_parse_tree_symbol())
        .count()
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
