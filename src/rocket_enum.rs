/// we want enums that derive Responder because those are ones that can be used
/// in responses. the variants will have response attributes with status and
/// content_type information

pub struct RocketEnum {
    ident: String,
    variants: Vec<RocketVariant>,
}

struct RocketVariant {
    ident: String,
    fields: Vec<(String, String)>,

    // from attribute
    status: u8,
    content_type: Option<String>,
}

impl RocketEnum {
    pub fn parse_enum(enm: syn::ItemEnum) -> Self {
        unimplemened!()
    }
}
