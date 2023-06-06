use proc_macro::TokenStream;
use quote::quote;
use std::str::FromStr;

enum Value {
    Str,
    Int,
    Long,
    Bool,
}

enum Constraint {
    Min(i32),
    Max(i32),
    NotEmpty,
    Optional,
}

struct KeyValue {
    key: String,
    value: Value,
    constraints: Vec<Constraint>,
}

impl FromStr for KeyValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').map(|part| part.trim()).collect();

        if parts.len() < 2 {
            return Err(());
        }

        let key = parts[0].to_string();
        let value = match parts[1] {
            "int" => Value::Int,
            "str" => Value::Str,
            "long" => Value::Long,
            "bool" => Value::Bool,
            _ => panic!("Invalid value type: {}", parts[1]),
        };

        let constraints: Vec<Constraint> = parts[2..]
            .iter()
            .map(|&part| {
                let constraint_parts: Vec<&str> = part.split_whitespace().collect();
                match constraint_parts.as_slice() {
                    ["min", val] => Constraint::Min(val.parse().unwrap()),
                    ["max", val] => Constraint::Max(val.parse().unwrap()),
                    ["notEmpty"] => Constraint::NotEmpty,
                    ["optional"] => Constraint::Optional,
                    _ => panic!("Invalid constraint: {}", part),
                }
            })
            .collect();

        Ok(KeyValue {
            key,
            value,
            constraints,
        })
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
             value,
             constraints,
         }| {
            let key_str = key.as_str();
            let value_enum = match value {
                Value::Int => quote! { Value::Int },
                Value::Str => quote! { Value::Str },
                Value::Long => quote! { Value::Long },
                Value::Bool => quote! { Value::Bool },
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
