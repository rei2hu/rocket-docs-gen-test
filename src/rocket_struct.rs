#[derive(Debug, PartialEq)]
pub struct RocketStruct {
    ident: String,
    // so for unnamed ill just go with (0, type), (1, type) like a fake array
    fields: Vec<(String, String)>,
}

impl RocketStruct {
    pub fn parse_struct(s: &syn::ItemStruct) -> Self {
        let fields = match s.fields.to_owned() {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .map(|field| {
                    (
                        crate::ast_formatting::format_idnt(&field.ident.to_owned().unwrap()),
                        crate::ast_formatting::format_type(&field.ty),
                    )
                })
                .collect(),
            syn::Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    (
                        idx.to_string(),
                        crate::ast_formatting::format_type(&field.ty),
                    )
                })
                .collect(),
            syn::Fields::Unit => vec![],
        };

        RocketStruct {
            ident: crate::ast_formatting::format_idnt(&s.ident),
            fields,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_structs_with_named_fields() {
        let result = RocketStruct::parse_struct(
            &syn::parse_str(
                "
                pub struct MyStruct {
                    field1: i32,
                    field2: AnotherStruct,
                    field3: (i32, u8),
                }
                ",
            )
            .unwrap(),
        );
        assert_eq!(
            result,
            RocketStruct {
                ident: "MyStruct".to_string(),
                fields: vec![
                    ("field1".to_string(), "i32".to_string()),
                    ("field2".to_string(), "AnotherStruct".to_string()),
                    ("field3".to_string(), "(i32 , u8)".to_string())
                ]
            },
            "Parses struct properly"
        );
    }

    #[test]
    fn parses_structs_with_unnamed_fields() {
        let result = RocketStruct::parse_struct(
            &syn::parse_str(
                "
                struct Point(i32, i32);
                ",
            )
            .unwrap(),
        );
        assert_eq!(
            result,
            RocketStruct {
                ident: "Point".to_string(),
                fields: vec![
                    ("0".to_string(), "i32".to_string()),
                    ("1".to_string(), "i32".to_string())
                ]
            },
            "Parses struct properly"
        );
    }
}
