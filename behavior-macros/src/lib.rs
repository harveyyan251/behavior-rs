extern crate proc_macro;
extern crate quote;
extern crate serde;
extern crate serde_json;
extern crate syn;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, GenericArgument, PathArguments};

// TODO: 重构所有宏

#[proc_macro_derive(TreeNodeStatus)]
pub fn derive_tree_node_status(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let gen = quote! {
        impl #impl_generics TreeNodeStatus for #struct_name #ty_generics #where_clause {
            fn get_status(&self) -> Status {
                self.base.get_status()
            }

            fn set_status(&mut self, status: Status) {
                self.base.set_status(status);
            }

            fn reset_status(&mut self) {
                self.base.reset_status();
            }

            fn is_running(&self) -> bool {
                self.base.is_running()
            }

            fn is_completed(&self) -> bool {
                self.base.is_completed()
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(EditorEnumDataGenerator)]
pub fn derive_editor_enum_data_generator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // 确保输入是一个枚举
    let gen = match &input.data {
        Data::Enum(data_enum) => {
            // 创建变体和对应值的映射
            let variants = data_enum.variants.iter().map(|variant| {
                let var_name = &variant.ident;
                let var_value = variant
                    .discriminant
                    .as_ref()
                    .and_then(|(_, expr)| {
                        if let syn::Expr::Lit(inner) = expr {
                            Some(quote! { #inner })
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| quote! { 0 }); // 默认值为 0

                // 生成不带前缀的变体名称和对应值
                quote! {
                    stringify!(#var_name): #var_value
                }
            });

            // 生成代码
            quote! {
                impl EditorEnumDataGenerator for #struct_name {
                    fn generate_editor_enum_data() -> serde_json::Value {
                        serde_json::json!({
                            stringify!(#struct_name): {
                                #(#variants), *
                            }
                        })
                    }
                }
                // impl #struct_name {
                //     pub fn gen_import_data() -> serde_json::Value {
                //         serde_json::json!({
                //             stringify!(#struct_name): {
                //                 #(#variants), *
                //             }
                //         })
                //     }
                // }
            }
        }
        _ => {
            panic!("Expected an enum; found something else.");
        }
    };
    gen.into()
}

#[proc_macro_derive(EditorNodeDataGenerator)]
pub fn derive_editor_node_data_generator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let meta_field: Vec<proc_macro2::TokenStream> = if let Data::Struct(data) = &input.data {
        if let Fields::Named(ref fields) = data.fields {
            fields
                .named
                .iter()
                .filter_map(|field: &Field| {
                    let field_type = &field.ty;
                    if is_metadata_cell(field_type) {
                        let field_name = &field.ident;
                        let inner_types = extract_meta_inner_types(field_type);
                        let inner_type = inner_types[0].clone();
                        if inner_types.len() >= 2 {
                            let enum_ref_type = inner_types[1].clone();
                            Some(quote! {
                                {
                                    "name": stringify!(#field_name),
                                    "ty": stringify!(#inner_type).replace(" ", ""),
                                    "enum_ref": stringify!(#enum_ref_type).replace(" ", ""),
                                }
                            })
                        } else {
                            Some(quote! {
                                {
                                    "name": stringify!(#field_name),
                                    "ty": stringify!(#inner_type).replace(" ", ""),
                                }
                            })
                        }
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let bb_ref_field: Vec<proc_macro2::TokenStream> = if let Data::Struct(data) = &input.data {
        if let Fields::Named(ref fields) = data.fields {
            fields
                .named
                .iter()
                .filter_map(|field: &Field| {
                    let field_type = &field.ty;
                    if is_blackboard_cell(field_type) {
                        let field_name = &field.ident;
                        let inner_ty = extract_inner_type(field_type);
                        Some(quote! {
                            {
                                "name": stringify!(#field_name),
                                "ty": stringify!(#inner_ty).replace(" ", ""),
                            }
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let dyn_ref_field: Vec<proc_macro2::TokenStream> = if let Data::Struct(data) = &input.data {
        if let Fields::Named(ref fields) = data.fields {
            fields
                .named
                .iter()
                .filter_map(|field: &Field| {
                    let field_type = &field.ty;
                    if is_dynamic_cell(field_type) {
                        let field_name = &field.ident;
                        let inner_ty = extract_inner_type(field_type);
                        Some(quote! {
                            {
                                "name": stringify!(#field_name),
                                "ty": stringify!(#inner_ty).replace(" ", ""),
                            }
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let gen = quote! {
        impl EditorNodeDataGenerator for #struct_name {
            fn generate_editor_node_data() -> serde_json::Value {
                serde_json::json!({
                    "name": stringify!(#struct_name),
                    "metadata": [#(#meta_field),*],
                    "refs": [#(#bb_ref_field),*],
                    "dyn_refs": [#(#dyn_ref_field),*],
                })
            }
        }
    };

    gen.into()
}

fn is_metadata_cell(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        return type_path
            .path
            .segments
            .iter()
            .any(|seg| seg.ident == "MetaDataCell");
    }
    false
}

fn is_blackboard_cell(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        return type_path
            .path
            .segments
            .iter()
            .any(|seg| seg.ident == "BlackBoardCell");
    }
    false
}

fn is_dynamic_cell(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        return type_path
            .path
            .segments
            .iter()
            .any(|seg| seg.ident == "DynamicCell");
    }
    false
}

fn extract_inner_type(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                    return inner_type.clone();
                }
            }
        }
    }
    ty.clone()
}

fn extract_meta_inner_types(ty: &syn::Type) -> Vec<syn::Type> {
    let mut inner_types = Vec::new();
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                for arg in &args.args {
                    if let syn::GenericArgument::Type(inner_type) = arg {
                        inner_types.push(inner_type.clone());
                    }
                    if inner_types.len() == 2 {
                        break; // 限制到前两个参数
                    }
                }
            }
        }
    }

    inner_types
}
