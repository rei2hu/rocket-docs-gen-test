#![feature(box_patterns, bindings_after_at, never_type)]

#[macro_use]
extern crate quote;

mod ast_formatting;
mod rocket_route;
mod rocket_struct;

fn main() {
    let content = "
    #![feature(proc_macro_hygiene, decl_macro)]
    #[macro_use] extern crate rocket;

    #[get(\"/hello/<name>/<age>\")]
    fn hello(name: String, age: u8) -> String {
        format!(\"Hello, {} year old named {}!\", age, name)
    }

    #[derive(Responder)]
    pub enum LoginResponse {
        #[response(status = 200, content_type = \"text/plain\")]
        Success(i32),
        #[response(status = 401, content_type = \"text/plain\")]
        Failure(String),
    }

    struct LoginData {
        username: String,
        password: String,
    }

    #[post(\"/login\", format=\"application/json\", data = \"<data>\")]
    fn login(user: User, data: LoginData) -> LoginResponse {
        if username == \"a\" && password == \"b\" {
            LoginResponse::Success(user.id)
        } else {
            LoginResponse::Failure(\"Bad login\")
        }
    }

    fn main() {
        rocket::ignite().mount(\"/\", routes![hello]).launch();
    }
    "
    .to_string();

    let ast = syn::parse_str(&content).unwrap();
    traverse_file(ast);
}

fn traverse_file(ast: syn::File) {
    traverse_items(&ast.items, 0);
}

fn traverse_items(items: &Vec<syn::Item>, depth: u32) {
    items.iter().for_each(|item| traverse_item(item, depth));
}

fn traverse_item(item: &syn::Item, depth: u32) {
    match item {
        // fn x { }
        syn::Item::Fn(
            function
            @
            syn::ItemFn {
                block: box syn::Block { stmts, .. },
                ..
            },
        ) => {
            let x = rocket_route::RocketRoute::parse_fn(function);
            if x.len() > 0 {
                println!("{:#?}", x);
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
            if let syn::ItemMod {
                content: Some((_, items)),
                ..
            } = item_mod
            {
                traverse_items(items, depth + 1)
            }
        }

        // struct x { }
        syn::Item::Struct(strct) => {
            let x = rocket_struct::RocketStruct::parse_struct(strct);
            println!("{:#?}", x);
        }
        _ => (),
    };
}
