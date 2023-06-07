use proc_macro::TokenStream;
use quote::quote;
use std::str::FromStr;
mod validators;
use validators::{validate_bool_constraints, validate_num_constraints, Constraint};
struct KeyValue {
    key: String,
    ty: String,
    constraints: Option<Vec<Constraint>>,
}

impl FromStr for KeyValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("=>").collect();

        let kv_parts: Vec<&str> = parts[0].split(':').collect();

        if kv_parts.len() != 2 {
            return Err(());
        }
        let key = kv_parts[0].trim().to_string();
        let ty = kv_parts[1].trim().to_string();

        if parts.len() != 2 {
            return Ok(KeyValue {
                key,
                ty,
                constraints: None,
            });
        }
        let splitted_constraints = parts[1].split_ascii_whitespace().map(|cons| cons.trim());
        let mut constraints = vec![];

        for constraint in splitted_constraints {
            if constraint.starts_with("min(") && constraint.ends_with(')') {
                let value = constraint[4..constraint.len() - 1].trim().parse::<i64>();
                if value.is_err() {
                    panic!("Wrong value for constraint Min provided");
                }
                constraints.push(Constraint::Min(value.unwrap()));
            } else if constraint.starts_with("max(") && constraint.ends_with(')') {
                let value = constraint[4..constraint.len() - 1].trim().parse::<i64>();
                if value.is_err() {
                    panic!("Wrong value for constraint Max provided");
                }
                constraints.push(Constraint::Max(value.unwrap()));
            } else if constraint.to_ascii_lowercase() == "notempty" {
                constraints.push(Constraint::NotEmpty);
            } else if constraint.to_ascii_lowercase() == "optional" {
                constraints.push(Constraint::Optional);
            } else {
                panic!("Wrong constraint provided");
            }
        }

        Ok(KeyValue {
            key,
            ty,
            constraints: Some(constraints),
        })
    }
}

fn generate_mask(constraints: &Option<Vec<Constraint>>) -> String {
    let constraints = constraints.as_ref().unwrap();
    let mut mask = [
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
    ];
    for cons in constraints {
        match cons {
            Constraint::Max(val) => mask[0].push_str(&format!("{}", val)),
            Constraint::Min(val) => mask[1].push_str(&format!("{}", val)),
            Constraint::Optional => mask[2].push('1'),
            Constraint::NotEmpty => mask[3].push('1'),
        }
    }

    mask.join(",")
}

#[proc_macro]
pub fn convert(input: TokenStream) -> TokenStream {
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
            let typing = match ty.to_ascii_lowercase().as_str() {
                "int" | "integer" | "i32" => {
                    if constraints.is_some() {
                        validate_num_constraints(key, constraints.as_ref().unwrap());
                    }

                    quote! { Value::Int }
                }
                "str" | "string" => {
                    quote! { Value::Str }
                }
                "long" | "i64" => {
                    if constraints.is_some() {
                        validate_num_constraints(key, constraints.as_ref().unwrap());
                    }
                    quote! {Value::Long }
                }
                "bool" | "boolean" => {
                    if constraints.is_some() {
                        validate_bool_constraints(key, constraints.as_ref().unwrap());
                    }
                    quote! { Value::Bool }
                }
                _ => panic!("Unsupported type: {:?}", ty),
            };

            if constraints.is_none() {
                quote! { (#key_str, #typing, "".to_string() ) }
            } else {
                let mask = generate_mask(constraints);
                quote! { (#key_str, #typing, #mask.to_string() ) }
            }
        },
    );

    let output = quote! { [ #( #result ),* ] };

    output.into()
}
