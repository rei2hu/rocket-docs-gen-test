#![feature(box_patterns, bindings_after_at, never_type)]

use std::error::Error;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = File::open("../test.rs")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let ast = syn::parse_file(&content)?;

    traverse_file(ast);

    Ok(())
}

fn traverse_file(ast: syn::File) {
    traverse_items(&ast.items, 0);
}

fn traverse_items(items: &Vec<syn::Item>, depth: u32) {
    items.iter().for_each(|item| traverse_item(item, depth));
}

#[derive(Debug)]
struct RocketAttribute {
    method: String,
    path: String,
    rank: Option<i32>,
    format: Option<String>,
    data: Option<String>,
}

#[derive(Debug)]
struct RocketFunctionArgument {
    name: String,
    arg_type: String,
}

#[derive(Debug)]
struct RocketFunction {
    args: Vec<RocketFunctionArgument>,
    return_type: String,
}

#[derive(Debug)]
struct RocketEndpoint {
    function: RocketFunction,
    attributes: Vec<RocketAttribute>,
}

fn to_rocket_attr(attribute: &syn::Attribute) -> Option<RocketAttribute> {
    attribute.parse_meta().ok().and_then(|meta| match meta {
        syn::Meta::Path(_) => None,
        syn::Meta::NameValue(_) => None,
        syn::Meta::List(l) => {
            let pairs = l
                .nested
                .pairs()
                .map(|pair| pair.into_tuple().0)
                .filter_map(|kv| match kv {
                    // assume the only one that isn't k=v is the path
                    syn::NestedMeta::Lit(syn::Lit::Str(str)) => {
                        Some(("path".to_string(), str.value()))
                    }
                    // gather up k=v into tuples
                    // anyway to consolidate these 2 cases?
                    syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                        path,
                        lit: syn::Lit::Int(int),
                        ..
                    })) => path
                        .get_ident()
                        .map(|ident| (ident.to_string(), int.to_string())),

                    syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                        path,
                        lit: syn::Lit::Str(str),
                        ..
                    })) => path
                        .get_ident()
                        .map(|ident| (ident.to_string(), str.value())),
                    _ => None,
                })
                .collect::<Vec<(String, String)>>();

            // so theres a case where you have something like post(not="a path")
            // not sure if that's even valid, but the unwrap below runs into it
            // so yeah
            if pairs.len() < 1 {
                return None;
            }

            l.path.get_ident().map(|ident| RocketAttribute {
                method: ident.to_string(),
                path: pairs
                    .iter()
                    .find(|pair| pair.0 == "path")
                    .unwrap()
                    .1
                    .to_owned(),
                rank: pairs
                    .iter()
                    .find(|pair| pair.0 == "rank")
                    .and_then(|pair| pair.1.parse().ok()),
                format: pairs
                    .iter()
                    .find(|pair| pair.0 == "format")
                    .map(|pair| pair.1.to_owned()),
                data: pairs
                    .iter()
                    .find(|pair| pair.0 == "data")
                    .map(|pair| pair.1.to_owned()),
            })
        }
    })
}

fn traverse_item(item: &syn::Item, depth: u32) {
    match item {
        // fn x { }
        syn::Item::Fn(
            function
            @
            syn::ItemFn {
                attrs,
                block: box syn::Block { stmts, .. },
                ..
            },
        ) => {
            let rocket_attrs = attrs
                .iter()
                .filter_map(to_rocket_attr)
                .collect::<Vec<RocketAttribute>>();

            if rocket_attrs.len() > 0 {
                let args = function
                    .sig
                    .inputs
                    .pairs()
                    .map(|pair| pair.into_tuple().0)
                    // assuming no receiver types i.e. self
                    .filter_map(|arg| {
                        if let syn::FnArg::Typed(arg) = arg {
                            Some(arg)
                        } else {
                            None
                        }
                    });

                // let return_type = function.sig.output;

                let endpoint = RocketEndpoint {
                    function: RocketFunction {
                        args: vec![],
                        return_type: "".to_string(),
                    },
                    attributes: rocket_attrs,
                };
                println!("{:#?}", function);
                println!("{:#?}", endpoint);
            }

            stmts
                .iter()
                .filter(|stmt| {
                    if let syn::Stmt::Item(..) = stmt {
                        true
                    } else {
                        false
                    }
                })
                .for_each(|item| {
                    if let syn::Stmt::Item(item) = item {
                        traverse_item(item, depth + 1)
                    } else {
                        panic!("filtered to items but found a non item?")
                    }
                })
        }

        // mod x or mod x { }
        syn::Item::Mod(item_mod) => {
            println!("mod {} ({})", item_mod.ident, depth,);
            if let syn::ItemMod {
                content: Some((_, items)),
                ..
            } = item_mod
            {
                traverse_items(items, depth + 1)
            }
        }

        // struct x { }
        syn::Item::Struct(strct) => println!("struct {} ({})", strct.ident, depth),
        _ => (),
    };
}
