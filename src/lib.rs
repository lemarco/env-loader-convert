use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use std::str::FromStr;
struct KeyValue {
    key: String,
    value: String,
    validation: String,
}

impl FromStr for KeyValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(());
        }
        let key = parts[0].trim().to_string();
        let value_and_validation: Vec<&str> = parts[1].trim().split("=>").collect();
        if value_and_validation.len() != 2 {
            return Ok(KeyValue {
                key,
                value: value_and_validation[0].trim().to_string(),
                validation: "".to_string(),
            });
        }
        let value = value_and_validation[0].trim().to_string();
        let mut validation: [String; 4] = [
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        ];
        value_and_validation[1].split(',').for_each(|func_literal| {
            // let literal = func_literal.to_string().trim();
            if func_literal.starts_with("min(") {
                let re = Regex::new(r"\d+").unwrap();
                if let Some(capture) = re.find(func_literal) {
                    let num: i32 = capture.as_str().parse().unwrap();
                    validation[0].push_str(&format!("{num}"));
                } else {
                    panic!()
                }
            } else if func_literal.starts_with("max(") {
                let re = Regex::new(r"\d+").unwrap();
                if let Some(capture) = re.find(func_literal) {
                    let num: i32 = capture.as_str().parse().unwrap();
                    validation[1].push_str(&format!("{num}"));
                } else {
                    panic!()
                }
            } else if func_literal == "notEmpty" {
                validation[2].push('1');
            } else if func_literal == "optional" {
                validation[3].push('1');
            } else {
                panic!();
            }
        });

        Ok(KeyValue {
            key,
            value,
            validation: validation.join(","),
        })
    }
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
             value,
             validation,
         }| {
            let key_str = key.as_str();
            let value_enum = match value.to_ascii_lowercase().as_str() {
                "int" | "integer" | "i32" => quote! { Value::Int },
                "str" | "string" => quote! { Value::Str },
                "long" | "i64" => quote! { Value::Long },
                "bool" | "boolean" => quote! { Value::Bool },
                "float" | "f32" => quote! { Value::Float },
                _ => panic!("Unsupported type: {:?}", value),
            };
            quote! { (#key_str, #value_enum, #validation) }
        },
    );

    let output = quote! { [ #( #result ),* ] };

    output.into()
}
