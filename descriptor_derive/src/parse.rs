use proc_macro_error::{abort, ResultExt};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{self, Attribute, Expr, Ident, LitStr, Token};

pub struct DescriptorAttr {
    ident: Ident,
    attribute: String,
    expr: Option<Expr>,
    value: Option<String>,
}

#[derive(Clone)]
pub struct DescriptorStructAttr {
    pub into: Option<Expr>,
    pub headers: Option<Expr>,
    pub map: Option<Expr>,
    pub extra_fields: Option<Expr>,
}

#[derive(Clone)]
pub struct DescriptorFieldAttr {
    pub skip_header: bool,
    pub skip_description: bool,
    pub skip: bool,
    pub output_table: bool,
    pub resolve_option: bool,
    pub into: Option<Expr>,
    pub map: Option<Expr>,
    pub rename_description: Option<String>,
    pub rename_header: Option<String>,
    pub flatten: bool,
}

impl Parse for DescriptorAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let name_str = ident.to_string();

        if input.peek(Token![=]) {
            // `name = value` attributes.
            let assign_token = input.parse::<Token![=]>()?; // skip '='

            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Ok(Self {
                    ident,
                    attribute: name_str,
                    expr: None,
                    value: Some(lit.value()),
                })
            } else {
                match input.parse::<Expr>() {
                    Ok(expr) => Ok(Self {
                        ident,
                        attribute: name_str,
                        expr: Some(expr),
                        value: None,
                    }),
                    Err(_) => abort! {
                        assign_token,
                        "expected `string literal` or `expression` after `=`"
                    },
                }
            }
        } else {
            // Attributes represented with a sole identifier.
            Ok(Self {
                ident,
                attribute: name_str,
                expr: None,
                value: None,
            })
        }
    }
}

pub fn parse_attributes(all_attrs: &[Attribute]) -> Vec<DescriptorAttr> {
    all_attrs
        .iter()
        .filter(|attr| attr.path.is_ident("descriptor"))
        .flat_map(|attr| {
            attr.parse_args_with(Punctuated::<DescriptorAttr, Token![,]>::parse_terminated)
                .unwrap_or_abort()
        })
        .collect()
}

pub fn extract_struct_attributes(all_attrs: &[Attribute]) -> DescriptorStructAttr {
    let mut struct_attr = DescriptorStructAttr {
        into: None,
        headers: None,
        map: None,
        extra_fields: None,
    };

    for attr in parse_attributes(all_attrs) {
        let DescriptorAttr {
            ident,
            attribute,
            expr,
            value,
        } = attr;
        match (attribute.as_str(), expr, value, ident) {
            ("into", Some(expr), ..) => struct_attr.into = Some(expr),
            ("into", _, _, ident) => {
                abort! {ident,"expected `string literal` or `expression` after `=`"}
            }
            ("map", Some(expr), ..) => struct_attr.map = Some(expr),
            ("map", _, _, ident) => {
                abort! {ident,"expected `string literal` or `expression` after `=`"}
            }
            ("extra_fields", Some(expr), ..) => struct_attr.extra_fields = Some(expr),
            ("extra_fields", _, _, ident) => {
                abort! {ident,"expected `string literal` or `expression` after `=`"}
            }
            ("default_headers", Some(expr), ..) => struct_attr.headers = Some(expr),
            ("default_headers", _, _, ident) => {
                abort! {ident,"expected `string literal` or `expression` after `=`"}
            }
            (.., ident) => abort! {ident,"unknown parameter"},
        }
    }

    struct_attr
}

pub fn extract_field_attributes(all_attrs: &[Attribute]) -> DescriptorFieldAttr {
    let mut field_attribute = DescriptorFieldAttr {
        skip_header: false,
        skip_description: false,
        skip: false,
        output_table: false,
        flatten: false,
        resolve_option: false,
        rename_header: None,
        rename_description: None,
        map: None,
        into: None,
    };

    for attr in parse_attributes(all_attrs) {
        let DescriptorAttr {
            ident,
            attribute,
            expr,
            value,
        } = attr;
        match (attribute.as_str(), expr, value, ident) {
            ("skip_header", None, None, ..) => field_attribute.skip_header = true,
            ("skip_header", _, _, ident) => {
                abort! {ident,"not expected `string literal` or `expression` after `=`"}
            }
            ("skip_description", None, None, ..) => field_attribute.skip_description = true,
            ("skip_description", _, _, ident) => {
                abort! {ident,"not expected `string literal` or `expression` after `=`"}
            }
            ("skip", None, None, ..) => field_attribute.skip = true,
            ("skip", _, _, ident) => {
                abort! {ident,"not expected `string literal` or `expression` after `=`"}
            }
            ("output_table", None, None, ..) => field_attribute.output_table = true,
            ("output_table", _, _, ident) => {
                abort! {ident,"not expected `string literal` or `expression` after `=`"}
            }
            ("map", Some(expr), None, ..) => field_attribute.map = Some(expr),
            ("map", _, _, ident) => {
                abort! {ident,"expected `string literal` or `expression` after `=`"}
            }
            ("flatten", None, None, ..) => field_attribute.flatten = true,
            ("flatten", _, _, ident) => {
                abort! {ident,"not expected `string literal` or `expression` after `=`"}
            }
            ("resolve_option", None, None, ..) => field_attribute.resolve_option = true,
            ("resolve_option", _, _, ident) => {
                abort! {ident,"not expected `string literal` or `expression` after `=`"}
            }
            ("rename_description", None, Some(val), ..) => {
                field_attribute.rename_description = Some(val)
            }
            ("rename_description", _, _, ident) => {
                abort! {ident,"expected `string literal` or `expression` after `=`"}
            }
            ("rename_header", None, Some(val), ..) => field_attribute.rename_header = Some(val),
            ("rename_header", _, _, ident) => {
                abort! {ident,"expected `string literal` or `expression` after `=`"}
            }
            ("into", Some(expr), ..) => field_attribute.into = Some(expr),
            ("into", _, _, ident) => {
                abort! {ident,"expected `string literal` or `expression` after `=`"}
            }
            (.., ident) => abort! {ident,"unknown parameter"},
        }
    }

    field_attribute
}
