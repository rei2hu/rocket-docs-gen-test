use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum RocketAttribute {
    DeriveResponder,
    Response(ResponseAttribute),
    Route(RouteAttribute),
}

#[derive(Debug, PartialEq)]
pub struct RouteAttribute {
    pub method: String,
    pub path: String,
    pub rank: Option<i32>,
    pub format: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct ResponseAttribute {
    pub status: u32,
    pub content_type: Option<String>,
}

impl RocketAttribute {
    pub fn from_attributes(attrs: &Vec<syn::Attribute>) -> Vec<Self> {
        fn nested_kv_to_hashmap(
            nested: &syn::punctuated::Punctuated<syn::NestedMeta, syn::token::Comma>,
        ) -> HashMap<String, String> {
            nested
                .pairs()
                .filter_map(|pair| {
                    if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = pair.value() {
                        nv.path.get_ident().map(|ident| {
                            (
                                ident.to_string(),
                                crate::ast_formatting::format_lit(&nv.lit),
                            )
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }

        attrs
            .iter()
            .filter_map(|attr| {
                attr.parse_meta().ok().and_then(|meta| match meta {
                    syn::Meta::List(l) => {
                        match l.path.get_ident().map(|ident| ident.to_string()) {
                            // #[derive(.., Responder, ..)]
                            Some(ref id)
                                if id == "derive"
                                    && l.nested.iter().any(|meta| {
                                        // ugly
                                        if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = meta {
                                            if path.segments.pairs().any(|segment| {
                                                segment.value().ident.to_string() == "Responder"
                                            }) {
                                                true
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    }) =>
                            {
                                Some(RocketAttribute::DeriveResponder)
                            }
                            // #[response(status = num, content_type = "idk")
                            Some(ref id) if id == "response" => {
                                let map = nested_kv_to_hashmap(&l.nested);

                                Some(RocketAttribute::Response(ResponseAttribute {
                                    status: map
                                        .get("status")
                                        .and_then(|val| val.parse().ok())
                                        .unwrap(),
                                    content_type: map
                                        .get("content_type")
                                        .map(|val| val.to_string()),
                                }))
                            }

                            // TODO: support for #[route(...)]
                            // #[get/post/etc("path", rank = 1, etc..)]
                            Some(ref id)
                                if id == "get"
                                    || id == "post"
                                    || id == "put"
                                    || id == "delete"
                                    || id == "head"
                                    || id == "options"
                                    || id == "patch" =>
                            {
                                let map = nested_kv_to_hashmap(&l.nested);
                                Some(RocketAttribute::Route(RouteAttribute {
                                    method: id.to_string(),
                                    path: l
                                        .nested
                                        .iter()
                                        .find_map(|kv| {
                                            if let syn::NestedMeta::Lit(syn::Lit::Str(str)) = kv {
                                                Some(str.value())
                                            } else {
                                                None
                                            }
                                        })
                                        .unwrap(),
                                    rank: map.get("rank").and_then(|val| val.parse().ok()),
                                    format: map.get("format").map(|val| val.to_string()),
                                    data: map.get("data").map(|val| val.to_string()),
                                }))
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                })
            })
            .collect()
    }

    pub fn from_struct(strct: &syn::ItemStruct) -> Vec<Self> {
        Self::from_attributes(&strct.attrs)
    }

    pub fn from_fn(function: &syn::ItemFn) -> Vec<Self> {
        Self::from_attributes(&function.attrs)
    }

    pub fn from_enum(enm: &syn::ItemEnum) -> Vec<Self> {
        Self::from_attributes(&enm.attrs)
    }

    pub fn from_variant(var: &syn::Variant) -> Vec<Self> {
        Self::from_attributes(&var.attrs)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let x = RocketAttribute::from_fn(
            &syn::parse_str(
                "
                #[post(\"my/path\", data = \"<something>\")]
                fn x() {}
                ",
            )
            .unwrap(),
        );

        assert_eq!(
            x,
            vec![RocketAttribute::Route(RouteAttribute {
                method: "post".to_string(),
                path: "my/path".to_string(),
                rank: None,
                format: None,
                data: Some("<something>".to_string())
            })],
            "Parses attributes on a function correctly"
        )
    }

    #[test]
    fn test_1() {
        let x = RocketAttribute::from_struct(
            &syn::parse_str(
                "
                #[derive(Responder)]
                #[response(status =400, content_type = \"application/json\")]
                struct MyResponse {
                    body: String,
                    head1: Option<CustomType>,
                }
                ",
            )
            .unwrap(),
        );

        assert_eq!(
            x,
            vec![
                RocketAttribute::DeriveResponder,
                RocketAttribute::Response(ResponseAttribute {
                    status: 400,
                    content_type: Some("application/json".to_string())
                })
            ],
            "Parses attributes on a struct properly"
        )
    }
}
