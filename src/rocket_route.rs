/// the only functions we are interested in are ones with the route attributes.
/// these represent the routes that are exposed.

#[derive(Debug, PartialEq)]
pub struct RocketRoute {
    ident: String,
    handler: Function,

    // from attribute
    method: String,
    path: String,
    rank: Option<i32>,
    format: Option<String>,
    data: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct Function {
    args: Vec<(String, String)>,
    ret: String,
}

impl RocketRoute {
    // each attribute defines its own route, it seems like you can only put
    // one route attribute on a function anyways, so really returning a
    // vector isnt an actual case since the length should always be 1
    pub fn parse_fn(function: &syn::ItemFn) -> Vec<Self> {
        let handler = Function {
            args: function
                .sig
                .inputs
                .iter()
                .filter_map(|kv| match kv {
                    syn::FnArg::Receiver(_) => None,
                    syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => Some((
                        crate::ast_formatting::format_pat(pat),
                        crate::ast_formatting::format_type(ty),
                    )),
                })
                .collect(),
            ret: crate::ast_formatting::format_ret_type(&function.sig.output),
        };

        // TODO: support for #[route(...)]
        fn is_valid_route_attribute(ident: &syn::Ident) -> bool {
            let str = crate::ast_formatting::format_idnt(ident);
            str == "get"
                || str == "put"
                || str == "post"
                || str == "delete"
                || str == "head"
                || str == "options"
                || str == "patch"
        }

        function
            .attrs
            .iter()
            .filter_map(|attr| {
                attr.parse_meta().ok().and_then(|meta| match meta {
                    syn::Meta::List(l)
                        if l.path.get_ident().map_or(false, is_valid_route_attribute) =>
                    {
                        let pairs = l
                            .nested
                            .iter()
                            .filter_map(|kv| match kv {
                                // assume the only one that isn't k=v is the path
                                syn::NestedMeta::Lit(syn::Lit::Str(str)) => {
                                    Some(("path".to_string(), str.value()))
                                }
                                // gather up k=v into tuples
                                // anyway to consolidate these 2 cases?
                                syn::NestedMeta::Meta(syn::Meta::NameValue(
                                    syn::MetaNameValue {
                                        path,
                                        lit: syn::Lit::Int(int),
                                        ..
                                    },
                                )) => path.get_ident().map(|ident| {
                                    (crate::ast_formatting::format_idnt(ident), int.to_string())
                                }),

                                syn::NestedMeta::Meta(syn::Meta::NameValue(
                                    syn::MetaNameValue {
                                        path,
                                        lit: syn::Lit::Str(str),
                                        ..
                                    },
                                )) => path.get_ident().map(|ident| {
                                    (crate::ast_formatting::format_idnt(ident), str.value())
                                }),
                                _ => None,
                            })
                            .collect::<Vec<(String, String)>>();

                        // so theres a case where you have something like #[post
                        // (not="a path")]. i dont think that's valid because
                        // the compiler should complain about the attribute
                        // if pairs.len() < 1 {
                        //     return None;
                        // }

                        l.path.get_ident().map(|ident| RocketRoute {
                            ident: crate::ast_formatting::format_idnt(&function.sig.ident),
                            method: crate::ast_formatting::format_idnt(ident),

                            // want a better way to do this like reducing into
                            // a struct and spreading it inside somehow
                            path: pairs
                                .iter()
                                .find(|(key, _)| key == "path")
                                .unwrap()
                                .1
                                .to_owned(),
                            rank: pairs
                                .iter()
                                .find(|(key, _)| key == "rank")
                                .and_then(|pair| pair.1.parse().ok()),
                            format: pairs
                                .iter()
                                .find(|(key, _)| key == "format")
                                .map(|pair| pair.1.to_owned()),
                            data: pairs
                                .iter()
                                .find(|(key, _)| key == "data")
                                .map(|pair| pair.1.to_owned()),
                            handler: handler.clone(),
                        })
                    }
                    _ => None,
                })
            })
            .collect::<Vec<RocketRoute>>()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_function_with_rocket_attribute() {
        let result = RocketRoute::parse_fn(
            &syn::parse_str(
                "
                #[post(\"/some/path\", format = \"application/json\")]
                fn my_fn(arg1: String, arg2: CustomType) -> Result<User, Error> {

                }
                ",
            )
            .unwrap(),
        );

        assert_eq!(
            result,
            vec![RocketRoute {
                ident: "my_fn".to_string(),
                method: "post".to_string(),
                path: "/some/path".to_string(),
                rank: None,
                format: Some("application/json".to_string()),
                data: None,
                handler: Function {
                    args: vec![
                        ("arg1".to_string(), "String".to_string()),
                        ("arg2".to_string(), "CustomType".to_string())
                    ],
                    ret: "Result < User , Error >".to_string()
                }
            }],
            "Parses a function with a route attribute properly"
        )
    }

    #[test]
    fn ignores_non_rocket_attributes() {
        let result = RocketRoute::parse_fn(
            &syn::parse_str(
                "
                #[not_rocket(\"/some/path\", format = \"application/json\")]
                fn my_fn(arg1: String, arg2: CustomType) -> Result<User, Error> {

                }
                ",
            )
            .unwrap(),
        );
        assert_eq!(
            result.len(),
            0,
            "Filters out functions that do not have a route attribute"
        );
    }

    #[test]
    fn handles_function_with_at_least_one_route_attribute() {
        let result = RocketRoute::parse_fn(
            &syn::parse_str(
                "
                #[private]
                #[not_rocket(\"/some/path\", format = \"application/json\")]
                #[head(\"/\", rank=12)]
                fn my_fn2(arg14: String, arg2: Option<Auth>) -> i32 {

                }
                ",
            )
            .unwrap(),
        );
        assert_eq!(
            result,
            vec![RocketRoute {
                ident: "my_fn2".to_string(),
                method: "head".to_string(),
                path: "/".to_string(),
                rank: Some(12),
                format: None,
                data: None,
                handler: Function {
                    args: vec![
                        ("arg14".to_string(), "String".to_string()),
                        ("arg2".to_string(), "Option < Auth >".to_string())
                    ],
                    ret: "i32".to_string()
                }
            }],
            "Parses functions that have at least one route attribute"
        );
    }
}
