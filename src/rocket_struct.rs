use crate::rocket_attribute::{ResponseAttribute, RocketAttribute};

/// there are at least 2 cases where we would be interested in structs:
///   1. request guards or parameter guards
///   2. when it's being used in the response
/// in the first scenario, it could be any struct
/// in the second scenario, it should derive Responder and also have a response
/// attrbute with status/content_type information

#[derive(Debug, PartialEq)]
pub struct RocketStruct {
    ident: String,
    // so for unnamed ill just go with (0, type), (1, type) like a fake array
    fields: Vec<(String, String)>,
    response: Option<ResponseAttribute>,
}

impl RocketStruct {
    pub fn parse_struct(s: &syn::ItemStruct) -> Self {
        let attrs = RocketAttribute::from_struct(s);

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
            response: attrs.into_iter().find_map(|attr| {
                if let RocketAttribute::Response(response) = attr {
                    Some(response)
                } else {
                    None
                }
            }),
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
                ],
                response: None,
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
                ],
                response: None,
            },
            "Parses struct properly"
        );
    }

    #[test]
    fn parses_structs_with_response_attribute() {
        let result = RocketStruct::parse_struct(
            &syn::parse_str(
                "
                #[response(status=404)]
                struct MyResponse(String);
                ",
            )
            .unwrap(),
        );
        assert_eq!(
            result,
            RocketStruct {
                ident: "MyResponse".to_string(),
                fields: vec![("0".to_string(), "String".to_string())],
                response: Some(ResponseAttribute {
                    status: 404,
                    content_type: None
                }),
            },
            "Parses struct properly"
        );
    }
}
