use proc_macro::TokenStream;
use quote::quote;
use std::str::FromStr;

struct KeyValue {
    key: String,
    ty: String,
    constraints: Vec<Constraint>,
}
#[derive(Debug)]
enum Constraint {
    Min(i32),
    Max(i32),
    NotEmpty,
    Optional,
}

impl FromStr for KeyValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(());
        }
        let key = parts[0].trim().to_string();
        let constraints: Vec<Constraint> = parts[1]
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<Constraint>, _>>()
            .unwrap_or_else(|_| panic!("Invalid input: {:?}", parts[1]));

        let ty =
            constraints
                .iter()
                .rev()
                .fold(
                    parts.pop().unwrap().to_string(),
                    |acc, constraint| match constraint {
                        Constraint::Min(_) | Constraint::Max(_) => acc,
                        _ => {
                            let constraint_str = match constraint {
                                Constraint::NotEmpty => "notEmpty",
                                Constraint::Optional => "optional",
                                _ => unreachable!(),
                            };
                            format!("{} => {}", constraint_str, acc)
                        }
                    },
                );

        Ok(KeyValue {
            key,
            ty,
            constraints,
        })
    }
}

impl FromStr for Constraint {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.starts_with("min(") && s.ends_with(')') {
            let val = s[4..s.len() - 1]
                .parse()
                .unwrap_or_else(|_| panic!("Invalid constraint: {:?}", s));
            Ok(Constraint::Min(val))
        } else if s.starts_with("max(") && s.ends_with(')') {
            let val = s[4..s.len() - 1]
                .parse()
                .unwrap_or_else(|_| panic!("Invalid constraint: {:?}", s));
            Ok(Constraint::Max(val))
        } else if s == "notEmpty" {
            Ok(Constraint::NotEmpty)
        } else if s == "optional" {
            Ok(Constraint::Optional)
        } else {
            panic!("Invalid constraint: {:?}", s);
        }
    }
}

#[proc_macro]
pub fn convert_values(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let input = input.trim();
    let input = input.strip_prefix('{').unwrap_or(input);
    let input = input.strip_suffix('}').unwrap_or(input);
    let input = input.trim();

    let key_values: Vec<KeyValue> = input
        .split(',')
        .map(|s| s.trim().parse())
        .collect::<Result<Vec<KeyValue>, _>>()
        .unwrap_or_else(|_| panic!("Invalid input: {:?}", input));

    let result = key_values.iter().map(
        |KeyValue {
             key,
             ty,
             constraints,
         }| {
            let key_str = key.as_str();
            let value_enum = match ty.as_str() {
                "int" => {
                    validate_int_constraints(constraints);
                    quote! { Value::Int }
                }
                "str" => {
                    validate_str_constraints(constraints);
                    quote! { Value::Str }
                }
                "long" => {
                    validate_long_constraints(constraints);
                    quote! { Value::Long }
                }
                "bool" => {
                    validate_bool_constraints(constraints);
                    quote! { Value::Bool }
                }
                _ => panic!("Unsupported type: {:?}", ty),
            };
            let constraints_tokens = constraints.iter().map(|constraint| match constraint {
                Constraint::Min(val) => quote! { Constraint::Min(#val) },
                Constraint::Max(val) => quote! { Constraint::Max(#val) },
                Constraint::NotEmpty => quote! { Constraint::NotEmpty },
                Constraint::Optional => quote! { Constraint::Optional },
            });
            quote! { (#key_str, #value_enum, &[ #( #constraints_tokens ),* ]) }
        },
    );

    let output = quote! { [ #( #result ),* ] };

    output.into()
}

fn validate_int_constraints(constraints: &[Constraint]) {
    for constraint in constraints {
        match constraint {
            Constraint::Min(_) | Constraint::Max(_) | Constraint::Optional => {}
            _ => panic!("Invalid constraint for int: {:?}", constraint),
        }
    }
}

fn validate_str_constraints(constraints: &[Constraint]) {
    for constraint in constraints {
        match constraint {
            Constraint::Min(_)
            | Constraint::Max(_)
            | Constraint::Optional
            | Constraint::NotEmpty => {}
            _ => panic!("Invalid constraint for str: {:?}", constraint),
        }
    }
}

fn validate_long_constraints(constraints: &[Constraint]) {
    for constraint in constraints {
        match constraint {
            Constraint::Min(_) | Constraint::Max(_) | Constraint::Optional => {}
            _ => panic!("Invalid constraint for long: {:?}", constraint),
        }
    }
}

fn validate_bool_constraints(constraints: &[Constraint]) {
    for constraint in constraints {
        match constraint {
            Constraint::Optional => {}
            _ => panic!("Invalid constraint for bool: {:?}", constraint),
        }
    }
}

enum Value {
    Str,
    Int,
    Long,
    Bool,
}

// enum Constraint {
//     Min(i32),
//     Max(i32),
//     NotEmpty,
//     Optional,
// }
