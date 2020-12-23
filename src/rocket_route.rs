use crate::rocket_attribute::{RocketAttribute, RouteAttribute};

/// the only functions we are interested in are ones with the route attributes.
/// these represent the routes that are exposed.

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct RocketRoute {
    ident: String,
    handler: Function,
    route: RouteAttribute,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
struct Function {
    args: Vec<(String, String)>,
    ret: String,
}

impl RocketRoute {
    pub fn parse_fn(function: &syn::ItemFn) -> Option<Self> {
        let attrs = RocketAttribute::from_fn(function);

        // a function should have at least 1 route attribute to be important
        // there can only be 1 route attribute per fn
        if let Some(route_attr) = attrs.into_iter().find_map(|attr| {
            if let RocketAttribute::Route(route_attr) = attr {
                Some(route_attr)
            } else {
                None
            }
        }) {
            Some(RocketRoute {
                ident: crate::ast_formatting::format_idnt(&function.sig.ident),
                handler: Function {
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
                },
                route: route_attr,
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
            Some(RocketRoute {
                ident: "my_fn".to_string(),
                route: RouteAttribute {
                    method: "post".to_string(),
                    path: "/some/path".to_string(),
                    rank: None,
                    format: Some("application/json".to_string()),
                    data: None,
                },
                handler: Function {
                    args: vec![
                        ("arg1".to_string(), "String".to_string()),
                        ("arg2".to_string(), "CustomType".to_string())
                    ],
                    ret: "Result < User , Error >".to_string()
                }
            }),
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
        assert!(
            result.is_none(),
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
            Some(RocketRoute {
                ident: "my_fn2".to_string(),
                route: RouteAttribute {
                    method: "head".to_string(),
                    path: "/".to_string(),
                    rank: Some(12),
                    format: None,
                    data: None,
                },
                handler: Function {
                    args: vec![
                        ("arg14".to_string(), "String".to_string()),
                        ("arg2".to_string(), "Option < Auth >".to_string())
                    ],
                    ret: "i32".to_string()
                }
            }),
            "Parses functions that have at least one route attribute"
        );
    }
}
