/// some general formatting for various ast nodes. might update this eventually
/// for better output or use type aliases so im not dumping String everywhere
/// and getting confused

pub fn format_ret_type(ty: &syn::ReturnType) -> String {
    match ty {
        syn::ReturnType::Default => "()".to_string(),
        syn::ReturnType::Type(_, box ty) => format_type(ty),
    }
}

pub fn format_type(ty: &syn::Type) -> String {
    format!("{}", quote!(#ty))
}

pub fn format_pat(pat: &syn::Pat) -> String {
    format!("{}", quote!(#pat))
}

pub fn format_idnt(ident: &syn::Ident) -> String {
    ident.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_default_type() {
        assert_eq!(
            format_type(&syn::parse_str("()").unwrap()),
            "()",
            "Formatting the unit type should always return ()"
        );
    }

    #[test]
    fn format_default_return_type() {
        assert_eq!(
            format_ret_type(&syn::parse_str("").unwrap()),
            "()",
            "Formatting an unspecified return type should always return ()"
        );
    }

    #[test]
    fn format_option_type() {
        assert_eq!(
            format_type(&syn::parse_str("Option<i32>").unwrap()),
            "Option < i32 >",
            "Formatting an option should work"
        )
    }
}
