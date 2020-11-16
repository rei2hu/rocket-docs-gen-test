use crate::rocket_attribute::{ResponseAttribute, RocketAttribute};

/// we want enums that derive Responder because those are ones that can be used
/// in responses. the variants will have response attributes with status and
/// content_type information

#[derive(Debug, PartialEq)]
pub struct RocketEnum {
    ident: String,
    variants: Vec<RocketVariant>,
}

#[derive(Debug, PartialEq)]
struct RocketVariant {
    ident: String,
    response: ResponseAttribute,
    fields: Vec<(String, String)>,
}

impl RocketEnum {
    pub fn parse_enum(enm: &syn::ItemEnum) -> Option<Self> {
        let attrs = RocketAttribute::from_enum(enm);

        if let Some(_) = attrs.into_iter().find_map(|attr| {
            if let RocketAttribute::DeriveResponder = attr {
                Some(())
            } else {
                None
            }
        }) {
            Some(RocketEnum {
                ident: crate::ast_formatting::format_idnt(&enm.ident),
                variants: enm
                    .variants
                    .pairs()
                    .filter_map(|variant| {
                        let variant = variant.value();
                        let attrs = RocketAttribute::from_variant(variant);

                        if let Some(res_attr) = attrs.into_iter().find_map(|attr| {
                            if let RocketAttribute::Response(res_attr) = attr {
                                Some(res_attr)
                            } else {
                                None
                            }
                        }) {
                            let fields = match variant.fields.to_owned() {
                                syn::Fields::Named(fields) => fields
                                    .named
                                    .iter()
                                    .map(|field| {
                                        (
                                            crate::ast_formatting::format_idnt(
                                                &field.ident.to_owned().unwrap(),
                                            ),
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

                            Some(RocketVariant {
                                ident: crate::ast_formatting::format_idnt(&variant.ident),
                                response: res_attr,
                                fields,
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_1() {
        let x = RocketEnum::parse_enum(
            &syn::parse_str(
                "
            #[derive(Responder)]
            enum MyResponseEnum {
                #[response(status = 200, content_type = \"application/json\")]
                GoodStuff(i32, String),
                #[response(status = 400)]
                BadRequest(i32, i32),
                #[response(status = 500, content_type = \"text\")]
                InternalError { body: String, header1: i32 },
            }
            ",
            )
            .unwrap(),
        );
        assert_eq!(
            x,
            Some(RocketEnum {
                ident: "MyResponseEnum".to_string(),
                variants: vec![
                    RocketVariant {
                        ident: "GoodStuff".to_string(),
                        response: ResponseAttribute {
                            status: 200,
                            content_type: Some("application/json".to_string())
                        },
                        fields: vec![
                            (0.to_string(), "i32".to_string()),
                            (1.to_string(), "String".to_string())
                        ]
                    },
                    RocketVariant {
                        ident: "BadRequest".to_string(),
                        response: ResponseAttribute {
                            status: 400,
                            content_type: None,
                        },
                        fields: vec![
                            (0.to_string(), "i32".to_string()),
                            (1.to_string(), "i32".to_string())
                        ]
                    },
                    RocketVariant {
                        ident: "InternalError".to_string(),
                        response: ResponseAttribute {
                            status: 500,
                            content_type: Some("text".to_string())
                        },
                        fields: vec![
                            ("body".to_string(), "String".to_string()),
                            ("header1".to_string(), "i32".to_string())
                        ]
                    }
                ],
            })
        );
    }
}
