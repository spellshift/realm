use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, FnArg, ItemStruct, ItemTrait, Lit, Meta, NestedMeta, Signature, TraitItem, Type, TypeReference};

#[proc_macro_attribute]
pub fn eldritch_library(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut trait_def = parse_macro_input!(item as ItemTrait);
    let trait_name = &trait_def.ident;
    let (impl_generics, ty_generics, where_clause) = trait_def.generics.split_for_impl();

    let lib_name = parse_macro_input!(attr as Lit);
    let lib_name_str = if let Lit::Str(lit) = lib_name {
        lit.value()
    } else {
        panic!("Expected string literal for library name");
    };

    // Inject supertraits
    trait_def.supertraits.push(parse_quote!(core::fmt::Debug));
    trait_def.supertraits.push(parse_quote!(core::marker::Send));
    trait_def.supertraits.push(parse_quote!(core::marker::Sync));

    let mut method_dispatches = Vec::new();
    let mut method_names = Vec::new();

    for item in &mut trait_def.items {
        if let TraitItem::Method(method) = item {
            // Check for eldritch_method attribute
            let mut is_eldritch = false;
            let mut rename = None;

            for attr in &method.attrs {
                if attr.path.is_ident("eldritch_method") {
                    is_eldritch = true;
                    if let Ok(Meta::List(meta)) = attr.parse_meta() {
                        if let Some(NestedMeta::Lit(Lit::Str(lit))) = meta.nested.first() {
                            rename = Some(lit.value());
                        }
                    }
                }
            }

            if is_eldritch {
                let method_name = &method.sig.ident;
                let bind_name = rename.unwrap_or_else(|| method_name.to_string());
                let (args_parsing, arg_names) = generate_args_parsing(&method.sig);

                method_dispatches.push(quote! {
                    #bind_name => {
                        #args_parsing
                        let result = self.#method_name(#arg_names);
                        eldritchv2::conversion::IntoEldritchResult::into_eldritch_result(result)
                    }
                });
                method_names.push(bind_name);
            }
        }
    }

    let adapter_name = syn::Ident::new(&format!("{}EldritchAdapter", trait_name), trait_name.span());

    // Generate helper trait and its implementation for T: Trait
    let expanded = quote! {
        #trait_def

        pub trait #adapter_name {
            fn _eldritch_type_name(&self) -> &str;
            fn _eldritch_method_names(&self) -> alloc::vec::Vec<alloc::string::String>;
            fn _eldritch_call_method(
                &self,
                name: &str,
                _eldritch_args: &[eldritchv2::Value],
                _eldritch_kwargs: &alloc::collections::BTreeMap<String, eldritchv2::Value>,
            ) -> Result<eldritchv2::Value, String>;
        }

        impl<T> #adapter_name for T
        where T: #trait_name #ty_generics #where_clause
        {
             fn _eldritch_type_name(&self) -> &str {
                #lib_name_str
            }

            fn _eldritch_method_names(&self) -> alloc::vec::Vec<alloc::string::String> {
                let mut names = alloc::vec::Vec::new();
                #(names.push(alloc::string::String::from(#method_names));)*
                names
            }

            fn _eldritch_call_method(
                &self,
                name: &str,
                _eldritch_args: &[eldritchv2::Value],
                _eldritch_kwargs: &alloc::collections::BTreeMap<String, eldritchv2::Value>,
            ) -> Result<eldritchv2::Value, String> {
                 match name {
                    #(#method_dispatches)*
                    _ => Err(format!("Method '{}' not found or not exposed", name)),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn eldritch_library_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let struct_def = parse_macro_input!(item as ItemStruct);
    let struct_name = &struct_def.ident;
    let (impl_generics, ty_generics, where_clause) = struct_def.generics.split_for_impl();

    let trait_ident = parse_macro_input!(attr as syn::Ident);
    let adapter_name = syn::Ident::new(&format!("{}EldritchAdapter", trait_ident), trait_ident.span());

    let expanded = quote! {
        #struct_def

        impl #impl_generics eldritchv2::ForeignValue for #struct_name #ty_generics #where_clause {
            fn type_name(&self) -> &str {
                <Self as #adapter_name>::_eldritch_type_name(self)
            }

            fn method_names(&self) -> alloc::vec::Vec<alloc::string::String> {
                <Self as #adapter_name>::_eldritch_method_names(self)
            }

            fn call_method(
                &self,
                name: &str,
                args: &[eldritchv2::Value],
                kwargs: &alloc::collections::BTreeMap<String, eldritchv2::Value>,
            ) -> Result<eldritchv2::Value, String> {
                <Self as #adapter_name>::_eldritch_call_method(self, name, args, kwargs)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn eldritch_method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

fn generate_args_parsing(sig: &Signature) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut parsing = Vec::new();
    let mut call_args = Vec::new();
    let mut arg_idx = 0usize;

    for input in &sig.inputs {
        match input {
            FnArg::Receiver(_) => continue,
            FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                let ty = &pat_type.ty;
                let arg_name_str = quote!(#pat).to_string();

                // Detect &str
                let is_str_ref = if let Type::Reference(TypeReference { elem, .. }) = &**ty {
                    if let Type::Path(p) = &**elem {
                        p.path.is_ident("str")
                    } else { false }
                } else { false };

                if is_str_ref {
                    parsing.push(quote! {
                        let #pat: String = if #arg_idx < _eldritch_args.len() {
                            eldritchv2::conversion::FromValue::from_value(&_eldritch_args[#arg_idx])?
                        } else if let Some(val) = _eldritch_kwargs.get(#arg_name_str) {
                            eldritchv2::conversion::FromValue::from_value(val)?
                        } else {
                            return Err(format!("Missing argument: {}", #arg_name_str));
                        };
                    });
                    call_args.push(quote!(&#pat));
                } else {
                    parsing.push(quote! {
                        let #pat: #ty = if #arg_idx < _eldritch_args.len() {
                            eldritchv2::conversion::FromValue::from_value(&_eldritch_args[#arg_idx])?
                        } else if let Some(val) = _eldritch_kwargs.get(#arg_name_str) {
                            eldritchv2::conversion::FromValue::from_value(val)?
                        } else {
                            return Err(format!("Missing argument: {}", #arg_name_str));
                        };
                    });
                    call_args.push(quote!(#pat));
                }
                arg_idx += 1;
            }
        }
    }

    (quote! { #(#parsing)* }, quote! { #(#call_args),* })
}
