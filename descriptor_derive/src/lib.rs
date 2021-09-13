use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, Fields, Ident, Item, ItemEnum, ItemStruct, Type, TypePath};

use crate::parse::{DescriptorFieldAttr, DescriptorStructAttr};

mod parse;

#[derive(Clone)]
struct StructField {
    ident: Ident,
    typ: Type,
    field_name: String,
    attr: DescriptorFieldAttr,
}

#[proc_macro_derive(Descriptor, attributes(descriptor))]
#[proc_macro_error]
pub fn descriptor(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Item);
    match input {
        Item::Enum(item) => generate_enum_decriptor(item),
        Item::Struct(item) => generate_struct_decriptor(item),
        _ => abort! {input, "not implemented for kind of input"},
    }
}

/// The purpose of the function is to implement the decriptor trait automatically
/// User can provide some attribute in order to customize the Description/Table generated
/// See parse.rs to check all possible attributes
/// The decriptor trait has 6 method:
/// `to_field` is the final call to have a String result of a field
///     We generate a `match` on field_name wanted, and forward it to inner struct if a dot exist
/// `default_headers` will return all headers by default, or just the one provided by the user
/// `headers` will generate the list of header recursively
/// `header_name` a method to get an header overrided name
/// `struct_pad` internal method in order to get the padding to apply for fields
/// `describe` will call describe on all fields in order to decriptor the description.
/// ```
fn generate_struct_decriptor(input: ItemStruct) -> proc_macro::TokenStream {
    let name = &input.ident;

    let decriptor_struct_attributes = parse::extract_struct_attributes(&input.attrs);
    let fields = extract_field(&input);

    let describe = describe_method_for_struct(&fields, &decriptor_struct_attributes);
    let default_headers = default_headers_for_struct(&fields, &decriptor_struct_attributes);
    let headers = headers_for_struct(&fields, &decriptor_struct_attributes);
    let header_name_func = rename_headers_for_struct(&fields, &decriptor_struct_attributes);
    let to_field = to_field_for_struct(&fields, &decriptor_struct_attributes);
    let pad_struct = pad_struct(&fields);

    generate_trait(
        name,
        describe,
        to_field,
        Some(pad_struct),
        Some(default_headers),
        Some(headers),
        Some(header_name_func),
    )
    .into()
}

fn pad_struct(fields: &[StructField]) -> TokenStream {
    let pad = match fields
        .iter()
        .map(|field| field.field_name.to_case(Case::Title).len())
        .max()
    {
        None => 0,
        Some(x) => x + 1,
    };

    let mut max_pad = quote! {
        let pad = #pad;
    };

    for field in fields {
        if field.attr.flatten {
            let typ = &field.typ;
            max_pad.extend(quote! {
                let pad = pad.max(#typ::struct_pad());
            })
        }
    }

    quote! {
        #max_pad
        pad
    }
}

fn extract_field(input: &ItemStruct) -> Vec<StructField> {
    match &input.fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|field| {
                let ident = match field.ident.as_ref() {
                    None => abort! {input.ident, "no identifier on field"},
                    Some(ident) => ident,
                };

                StructField {
                    ident: ident.clone(),
                    typ: field.ty.clone(),
                    field_name: ident.to_string(),
                    attr: parse::extract_field_attributes(&field.attrs),
                }
            })
            .filter(|x| !x.attr.skip)
            .collect::<Vec<StructField>>(),
        _ => abort! {input.ident, "not implemented for unnamed struct"},
    }
}

// Generate the to_field method implementation for the struct
fn to_field_for_struct(fields: &[StructField], struct_attributes: &DescriptorStructAttr) -> TokenStream {
    let mut match_to_field = quote!();

    fields
        .iter()
        .map(|field| {
            let field_name = &field.field_name;

            let value = field_getter(
                &field,
                quote! {
                    to_field(_child)
                },
            );

            quote! {
                #field_name => {#value},
            }
        })
        .for_each(|ts| match_to_field.extend(ts));

    let fallback = if let Some(additional_struct) = &struct_attributes.additional_struct {
        quote! {
            _ => {
                Into::<#additional_struct>::into(self).to_field(field_name)
            },
        }
    } else {
        quote! {
            _ => "field not found".to_string(),
        }
    };

    let return_value = if let Some(map) = &struct_attributes.map {
        quote! {
            #map(&self, value)
        }
    } else {
        quote! {
            value
        }
    };

    let func = quote! {
        let (field, _child) = descriptor::get_keys(field_name);

        let value = match field {
            #match_to_field
            #fallback
        };

        #return_value
    };

    func
}

// Generate the rename_header method implementation for the struct
fn rename_headers_for_struct(
    fields: &[StructField],
    struct_attributes: &DescriptorStructAttr,
) -> TokenStream {
    let mut rename_headers = quote!();

    fields
        .iter()
        .map(|field| {
            let typ = &field.typ;
            let field_name = &field.field_name;

            match &field.attr.rename_header {
                Some(rename) => quote! {
                    #field_name => Some(#rename.to_string()),
                },
                None => {
                    if let Some(into) = &field.attr.into {
                        quote! {
                            #field_name => <#into>::header_name(_child),
                        }
                    } else {
                        quote! {
                            #field_name => <#typ>::header_name(_child),
                        }
                    }
                }
            }
        })
        .for_each(|ts| rename_headers.extend(ts));

    if let Some(additional_struct) = &struct_attributes.additional_struct {
        rename_headers.extend(quote! {
            stringify!(#additional_struct) => <#additional_struct>::header_name(_child),
        });
    }

    let func = quote! {
        let (header, _child) = descriptor::get_keys(header);
        match header {
            #rename_headers
            _ => None,
        }
    };
    func
}

// Will generate the header function, we list all possible fields recursively
fn headers_for_struct(fields: &[StructField], struct_attributes: &DescriptorStructAttr) -> TokenStream {
    let mut headers = quote! {
        let mut headers = Vec::new();
    };

    for field in fields.iter() {
        let typ = &field.typ;
        let field_name = &field.field_name;

        headers.extend(if let Some(into) = &field.attr.into {
            quote! {
                let mut fields = <#into>::default_headers()
            }
        } else {
            quote! {
                let mut fields = <#typ>::default_headers()
            }
        });

        headers.extend(quote! {
                .into_iter()
                .map(|x| format!("{}.{}", #field_name, x).to_string())
                .collect::<Vec<String>>();

                if fields.is_empty() {
                    headers.push(#field_name.to_string());
                } else {
                    headers.append(&mut fields);
                }
        });
    }

    if let Some(additional_struct) = &struct_attributes.additional_struct {
        headers.extend(quote! {
            let mut fields = <#additional_struct>::default_headers();
            headers.append(&mut fields);
        })
    }

    headers.extend(quote! {
        // return headers at end of function
        headers
    });

    headers
}

// Generate the default_headers method implementation for the struct
fn default_headers_for_struct(
    fields: &[StructField],
    struct_attributes: &DescriptorStructAttr,
) -> TokenStream {
    match struct_attributes.headers.as_ref() {
        Some(headers) => {
            quote! {
                #headers.iter().map(|x| x.to_string()).collect::<Vec<String>>()
            }
        }
        None => {
            let mut slice = quote! {};

            fields
                .iter()
                .filter(|x| x.attr.skip_header)
                .map(|x| x.field_name.to_string())
                .for_each(|x| {
                    slice.extend(quote! {
                        #x,
                    })
                });

            quote! {
                const SKIP : &'static[&'static str] = &[#slice];
                Self::headers()
                    .into_iter()
                    .filter(|x| !SKIP.contains(&x.as_str()))
                    .map(|x|x.to_string())
                    .collect::<Vec<_>>()
            }
        }
    }
}

// Generate the describe method implementation for the struct
fn describe_method_for_struct(
    fields: &[StructField],
    struct_attributes: &DescriptorStructAttr,
) -> TokenStream {
    match struct_attributes.into.as_ref() {
        Some(into) => {
            quote! {
                Into::<#into>::into(self).describe(writer, ctx.clone())
            }
        }
        None => {
            let mut describe = quote!();

            fields
                .iter()
                .filter(|x| !x.attr.skip_description)
                .enumerate()
                .map(|(i, x)| describe_field(x, i == 0))
                .for_each(|value| describe.extend(value));

            if let Some(additional_struct) = &struct_attributes.additional_struct {
                describe.extend(quote! {
                    Into::<#additional_struct>::into(self).describe(writer, ctx.pad(Self::struct_pad()))?;
                })
            }

            describe.extend(quote!(Ok(())));
            describe
        }
    }
}

// Will generate the describe for a specific field
fn describe_field(field: &StructField, first_field: bool) -> TokenStream {
    let title_name = field.field_name.to_case(Case::Title);
    let ident = &field.ident;

    if field.attr.flatten {
        quote! {
            self.#ident.describe(writer, ctx.pad(Self::struct_pad()))?;
        }
    } else {
        let title = quote! {
            ctx.write_title(writer, #title_name, #first_field)?;
        };

        let value = if field.attr.output_table {
            quote! {
                ctx.describe_table(&self.#ident, writer)?;
            }
        } else {
            field_getter(
                &field,
                quote! {
                    describe(writer, ctx.indent(Self::struct_pad(), #title_name.len()))?;
                },
            )
        };

        quote! {
            #title
            #value
        }
    }
}

// A helper function that handle all the code to map/into/resolve_option
// Need a method to call after the getter
fn field_getter(field: &StructField, method: TokenStream) -> TokenStream {
    let ident = &field.ident;

    let value = match (&field.attr.map, &field.attr.into) {
        (Some(func), _) => {
            if field.attr.method {
                quote! {
                    #ident.#func()
                }
            } else {
                quote! {
                    #func(#ident)
                }
            }
        }
        (_, Some(into)) => {
            quote! {
                Into::<#into>::into(#ident)
            }
        }
        (_, _) => {
            quote! {
                #ident
            }
        }
    };

    if path_is_option(&field.typ) && field.attr.resolve_option {
        quote! {
            if let Some(#ident) = &self.#ident {
                #value.#method
            } else {
                self.#ident.#method
            }
        }
    } else {
        quote! {
            let #ident = &self.#ident;
            #value.#method
        }
    }
}

// Generate decriptor Trait impl for Enum.
fn generate_enum_decriptor(input: ItemEnum) -> proc_macro::TokenStream {
    let enum_name = &input.ident;

    let mut match_fields = quote! {};

    for variant in input.variants {
        let name = variant.ident;
        let field_attributes = parse::extract_field_attributes(&variant.attrs);

        let value = if let Some(rename) = field_attributes.rename_description {
            quote!(#rename)
        } else {
            quote!(stringify!(#name))
        };

        match_fields.extend(quote! {
            #enum_name::#name => #value.to_string(),
        })
    }

    let to_field = quote! {
        match self {
            #match_fields
        }
    };

    let describe = quote! {
        ctx.write_value(writer, self.to_field(""))
    };
    generate_trait(&input.ident, describe, to_field, None, None, None, None).into()
}

fn generate_trait(
    name: &Ident,
    describe: TokenStream,
    to_field: TokenStream,
    pad: Option<TokenStream>,
    default_headers: Option<TokenStream>,
    headers: Option<TokenStream>,
    header_name: Option<TokenStream>,
) -> TokenStream {
    let default_headers = match &default_headers {
        None => quote! {},
        Some(headers) => quote! {
            fn default_headers() -> Vec<String> {
                #headers
            }
        },
    };
    let headers = match &headers {
        None => quote! {},
        Some(headers) => quote! {
            fn headers() -> Vec<String> {
                #headers
            }
        },
    };

    let header_name = match &header_name {
        None => quote! {},
        Some(header_name) => quote! {
            fn header_name(header: &str) -> Option<String> {
                #header_name
            }
        },
    };

    let pad = match &pad {
        None => quote! {},
        Some(pad) => quote! {
            fn struct_pad() -> usize {
                #pad
            }
        },
    };

    quote! {
        impl descriptor::Describe for #name {
            fn describe<W>(&self, writer: &mut W, ctx: descriptor::Context) -> std::io::Result<()>
            where
                W: std::io::Write,
            {
                #describe
            }

            #header_name
            #headers
            #default_headers
            #pad

            fn to_field(&self, field_name: &str) -> String {
                #to_field
            }
        }
    }
}

fn path_is_option(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            path.leading_colon.is_none()
                && path.segments.len() == 1
                && path.segments.iter().next().unwrap().ident == "Option"
        }
        _ => false,
    }
}
