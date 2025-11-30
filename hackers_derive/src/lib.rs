extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Lit};

#[proc_macro_derive(FieldInfo, attributes(fieldinfo))]
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
        impl hackers::FieldInfoTrait for #name {  // <-- Use FieldInfoTrait
            fn get_field_info(&self) -> Vec<hackers::FieldMeta> {
                vec![
                    #(#field_infos),*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}