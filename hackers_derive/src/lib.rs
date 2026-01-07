extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit};

#[proc_macro_derive(DeriveFieldInfo, attributes(fieldinfo))]
pub fn field_info_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(named_fields) = &data_struct.fields {
            &named_fields.named
        } else {
            panic!("FieldInfo can only be derived for named fields.");
        }
    } else {
        panic!("FieldInfo can only be derived for structs.");
    };

    let field_infos = fields.iter().filter_map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;

        let mut field_name = ident.to_string();
        let mut skip = false;

        for attr in &f.attrs {
            if attr.path().is_ident("fieldinfo") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        skip = true;
                    } else if meta.path.is_ident("name") {
                        if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                            field_name = s.value();
                        }
                    }
                    Ok(())
                });
            }
        }

        if skip {
            None
        } else {
            let type_str = quote!(#ty).to_string();
            let interpret_expr = if type_str.starts_with("[c_char;") {
                quote! {
                    Some({
                        let raw_bytes: &[u8] = unsafe {
                            std::slice::from_raw_parts(
                                self.#ident.as_ptr() as *const u8,
                                self.#ident.len()
                            )
                        };
                        let nul_pos = raw_bytes.iter()
                            .position(|&b| b == 0)
                            .unwrap_or(raw_bytes.len());
                        format!("{}",
                            std::str::from_utf8(&raw_bytes[..nul_pos])
                                .unwrap_or("<invalid utf8>")
                        )
                    })
                }
            } else {
                quote! { Some(format!("{:?}", self.#ident)) }
            };

            Some(quote! {
                hackers::FieldMeta {
                    name: #field_name.into(),
                    offset: memoffset::offset_of!(#name, #ident),
                    size: std::mem::size_of::<#ty>(),
                    type_name: stringify!(#ty),
                    interpret: #interpret_expr,
                }
            })
        }
    });

    let expanded = quote! {
        impl hackers::FieldInfo for #name {
            fn get_field_info(&self) -> Vec<hackers::FieldMeta> {
                vec![
                    #(#field_infos),*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(HackSettings, attributes(setting, hack_settings))]
pub fn hack_settings_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut crate_path = quote!(hackers);
    for attr in &input.attrs {
        if attr.path().is_ident("hack_settings") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate_name") {
                    if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                        let path_str = s.value();
                        let path: syn::Path = syn::parse_str(&path_str).unwrap();
                        crate_path = quote!(#path);
                    }
                }
                Ok(())
            });
        }
    }

    let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(named_fields) = &data_struct.fields {
            &named_fields.named
        } else {
            panic!("HackSettings can only be derived for named fields.");
        }
    } else {
        panic!("HackSettings can only be derived for structs.");
    };

    let mut get_settings_entries = Vec::new();
    let mut apply_settings_entries = Vec::new();

    for f in fields {
        let ident = f.ident.as_ref().unwrap();
        let field_name_str = ident.to_string();

        let mut is_setting = false;
        for attr in &f.attrs {
            if attr.path().is_ident("setting") {
                is_setting = true;
                break;
            }
        }

        if is_setting {
            get_settings_entries.push(quote! {
                map.insert(
                    #field_name_str.to_string(),
                    #crate_path::serde_json::to_value(&self.#ident).unwrap_or(#crate_path::serde_json::Value::Null)
                );
            });

            apply_settings_entries.push(quote! {
                if let Some(val) = settings.get(#field_name_str) {
                    if let Ok(v) = #crate_path::serde_json::from_value(val.clone()) {
                        self.#ident = v;
                    }
                }
            });
        }
    }

    let expanded = quote! {
        impl #crate_path::HackSettings for #name {
            fn get_settings(&self) -> std::collections::HashMap<String, #crate_path::serde_json::Value> {
                let mut map = std::collections::HashMap::new();
                #(#get_settings_entries)*
                map
            }

            fn apply_settings(&mut self, settings: &std::collections::HashMap<String, #crate_path::serde_json::Value>) {
                #(#apply_settings_entries)*
            }
        }
    };

    TokenStream::from(expanded)
}
