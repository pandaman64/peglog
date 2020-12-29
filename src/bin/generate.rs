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
    use std::convert::TryInto;

    let mut ret = quote! {
        use peglog::{Emitter, Input, Parser};
    };

    for (id, rule) in grammar.rules.iter().enumerate() {
        let id: u16 = id.try_into().unwrap();
        let name = proc_macro2::Ident::new(&rule.name, proc_macro2::Span::call_site());
        let mut body = quote! { false };
        for clause in rule.clauses.iter() {
            let mut part = quote! { true };

            for phrase in clause.phrases.iter() {
                part.extend(match phrase {
                    Phrase::Terminal(s) => quote! {
                        && input.consume(#s, emitter)
                    },
                    Phrase::Nonterminal(name) => {
                        let parser = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
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

        ret.extend(quote! {
            pub struct #name;
            impl Parser for #name {
                const ID: u16 = #id;

                fn parse(input: &mut Input<'_>, emitter: &mut Emitter) -> bool {
                    #body
                }
            }
        });
    }

    ret
}

fn main() {
    // 0: T = "a" | "b" T
    let grammar = Grammar {
        rules: vec![Rule {
            name: "T".into(),
            clauses: vec![
                Clause {
                    phrases: vec![Phrase::Terminal("a".into())],
                },
                Clause {
                    phrases: vec![
                        Phrase::Terminal("b".into()),
                        Phrase::Nonterminal("T".into()),
                    ],
                },
            ],
        }],
    };

    println!("{}", generate(&grammar));
}
