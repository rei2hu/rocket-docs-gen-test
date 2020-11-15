use crate::rocket_attribute::ResponseAttribute;

/// we want enums that derive Responder because those are ones that can be used
/// in responses. the variants will have response attributes with status and
/// content_type information

#[derive(Debug)]
pub struct RocketEnum {
    ident: String,
    variants: Vec<RocketVariant>,
}

#[derive(Debug)]
struct RocketVariant {
    ident: String,
    response: ResponseAttribute,
    fields: Vec<(String, String)>,
}

impl RocketEnum {
    pub fn parse_enum(enm: &syn::ItemEnum) -> Self {
        RocketEnum {
            ident: "".to_string(),
            variants: vec![],
        }
    }
}
