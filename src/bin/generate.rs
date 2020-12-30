use quote::quote;

enum Phrase {
    Terminal(String),
    Nonterminal(String),
}

struct Clause {
    phrases: Vec<Phrase>,
}

struct Rule {
    name: String,
    clauses: Vec<Clause>,
}

struct Grammar {
    rules: Vec<Rule>,
}

fn generate(grammar: &Grammar) -> proc_macro2::TokenStream {
    let rule_names: Vec<_> = grammar
        .rules
        .iter()
        .map(|rule| proc_macro2::Ident::new(&rule.name, proc_macro2::Span::call_site()))
        .collect();
    let bodies: Vec<_> = {
        let mut bodies = Vec::with_capacity(grammar.rules.len());

        for rule in grammar.rules.iter() {
            let mut body = quote! { false };
            for clause in rule.clauses.iter() {
                let mut part = quote! { true };

                for phrase in clause.phrases.iter() {
                    part.extend(match phrase {
                        Phrase::Terminal(s) => quote! {
                            && input.consume(#s, emitter)
                        },
                        Phrase::Nonterminal(name) => {
                            let parser =
                                proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
                            quote! {
                                && input.parse::<#parser>(emitter)
                            }
                        }
                    });
                }

                body.extend(quote! {
                    || ({
                        let backtrack = *input;
                        let result = #part;
                        if !result {
                            *input = backtrack;
                        }
                        result
                    })
                });
            }

            bodies.push(body);
        }

        bodies
    };
    let raw_kinds: Vec<_> = (1..=grammar.rules.len())
        .map(|i| {
            use std::convert::TryInto;
            let i: u16 = i.try_into().unwrap();
            quote! { #i }
        })
        .collect();

    quote! {
        use peglog::{Emitter, Input, Parser};

        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum SyntaxKind {
            Token,
            #(#rule_names,)*
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct Language;
        impl rowan::Language for Language {
            type Kind = SyntaxKind;
            fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
                match raw.0 {
                    0 => SyntaxKind::Token,
                    #(#raw_kinds => SyntaxKind::#rule_names,)*
                    _ => unreachable!(),
                }
            }
            fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
                rowan::SyntaxKind(match kind {
                    SyntaxKind::Token => 0,
                    #(SyntaxKind::#rule_names => #raw_kinds,)*
                })
            }
        }
        impl peglog::Language for Language {
            const TOKEN: Self::Kind = SyntaxKind::Token;
        }

        #(
        pub struct #rule_names;
        impl Parser for #rule_names {
            type Language = Language;
            const KIND: SyntaxKind = SyntaxKind::#rule_names;

            fn parse(input: &mut Input<'_>, emitter: &mut Emitter) -> bool {
                #bodies
            }
        }
        )*
    }
}

fn main() {
    // 0: T = "a" | "b" T
    // let grammar = Grammar {
    //     rules: vec![Rule {
    //         name: "T".into(),
    //         clauses: vec![
    //             Clause {
    //                 phrases: vec![Phrase::Terminal("a".into())],
    //             },
    //             Clause {
    //                 phrases: vec![
    //                     Phrase::Terminal("b".into()),
    //                     Phrase::Nonterminal("T".into()),
    //                 ],
    //             },
    //         ],
    //     }],
    // };

    // https://www.slideshare.net/chiguri/pegexpression
    // S = "a" S "a" | "b" S "b" | ""
    // or, equivalently
    // S = Sa | Sb | Empty
    // Sa = "a" S "a"
    // Sb = "b" Sb "b"
    // Empty = ""
    let grammar = Grammar {
        rules: vec![
            Rule {
                name: "S".into(),
                clauses: vec![
                    Clause {
                        phrases: vec![Phrase::Nonterminal("Sa".into())],
                    },
                    Clause {
                        phrases: vec![Phrase::Nonterminal("Sb".into())],
                    },
                    Clause {
                        phrases: vec![Phrase::Nonterminal("Empty".into())],
                    },
                ],
            },
            Rule {
                name: "Sa".into(),
                clauses: vec![Clause {
                    phrases: vec![
                        Phrase::Terminal("a".into()),
                        Phrase::Nonterminal("S".into()),
                        Phrase::Terminal("a".into()),
                    ],
                }],
            },
            Rule {
                name: "Sb".into(),
                clauses: vec![Clause {
                    phrases: vec![
                        Phrase::Terminal("b".into()),
                        Phrase::Nonterminal("S".into()),
                        Phrase::Terminal("b".into()),
                    ],
                }],
            },
            Rule {
                name: "Empty".into(),
                clauses: vec![Clause {
                    phrases: vec![Phrase::Terminal("".into())],
                }],
            },
        ],
    };

    println!("{}", generate(&grammar));
}
