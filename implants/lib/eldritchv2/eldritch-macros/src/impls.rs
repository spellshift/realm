use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    FnArg, ItemStruct, ItemTrait, Lit, Meta, NestedMeta, Signature, TraitItem, Type, TypeReference,
    parse_quote,
};

/// Expands the `#[eldritch_library]` attribute.
///
/// This macro:
/// 1. Injects `Debug + Send + Sync` supertraits.
/// 2. Scans for methods annotated with `#[eldritch_method]`.
/// 3. Injects 3 helper methods directly into the trait with default implementations:
///    - `_eldritch_type_name`: Returns the library name.
///    - `_eldritch_method_names`: Returns a list of exposed method names.
///    - `_eldritch_call_method`: Dispatches calls to the actual methods.
#[allow(clippy::collapsible_if)]
pub fn expand_eldritch_library(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let mut trait_def: ItemTrait = syn::parse2(item)?;
    let trait_name = &trait_def.ident;

    // Parse library name from attribute
    let lib_name_str = if let Ok(Lit::Str(lit)) = syn::parse2::<Lit>(attr) {
        lit.value()
    } else {
        return Err(syn::Error::new(
            trait_name.span(),
            "Expected string literal for library name",
        ));
    };

    // Inject supertraits
    trait_def.supertraits.push(parse_quote!(core::fmt::Debug));
    trait_def.supertraits.push(parse_quote!(core::marker::Send));
    trait_def.supertraits.push(parse_quote!(core::marker::Sync));

    let mut method_dispatches = Vec::new();
    let mut method_registrations = Vec::new();
    let mut method_signatures = Vec::new();

    for item in &mut trait_def.items {
        if let TraitItem::Method(method) = item {
            // Check for eldritch_method attribute
            let mut is_eldritch = false;
            let mut rename = None;
            let mut cfg_attrs = Vec::new();

            for attr in &method.attrs {
                if attr.path.is_ident("eldritch_method") {
                    is_eldritch = true;
                    if let Ok(Meta::List(meta)) = attr.parse_meta() {
                        if let Some(NestedMeta::Lit(Lit::Str(lit))) = meta.nested.first() {
                            rename = Some(lit.value());
                        }
                    }
                } else if attr.path.is_ident("cfg") {
                    cfg_attrs.push(attr.clone());
                }
            }

            if is_eldritch {
                let method_name = &method.sig.ident;
                let bind_name = rename.unwrap_or_else(|| method_name.to_string());
                let (args_parsing, arg_names) = generate_args_parsing(&method.sig)?;
                let signature_gen = generate_signature(&method.sig, &bind_name)?;

                method_dispatches.push(quote! {
                    #(#cfg_attrs)*
                    #bind_name => {
                        #args_parsing
                        let result = self.#method_name(#arg_names);
                        eldritch_core::conversion::IntoEldritchResult::into_eldritch_result(result)
                    }
                });

                method_registrations.push(quote! {
                    #(#cfg_attrs)*
                    names.push(alloc::string::String::from(#bind_name));
                });

                method_signatures.push(quote! {
                    #(#cfg_attrs)*
                    #bind_name => Some(#signature_gen),
                });
            }
        }
    }

    // Inject helper methods directly into the trait
    trait_def.items.push(parse_quote! {
        fn _eldritch_type_name(&self) -> &str {
            #lib_name_str
        }
    });

    trait_def.items.push(parse_quote! {
        fn _eldritch_method_names(&self) -> alloc::vec::Vec<alloc::string::String> {
            let mut names = alloc::vec::Vec::new();
            #(#method_registrations)*
            names
        }
    });

    trait_def.items.push(parse_quote! {
        fn _eldritch_get_method_signature(&self, name: &str) -> Option<eldritch_core::MethodSignature> {
            match name {
                #(#method_signatures)*
                _ => None,
            }
        }
    });

    trait_def.items.push(parse_quote! {
        fn _eldritch_call_method(
            &self,
            interp: &mut eldritch_core::Interpreter,
            name: &str,
            _eldritch_args: &[eldritch_core::Value],
            _eldritch_kwargs: &alloc::collections::BTreeMap<alloc::string::String, eldritch_core::Value>,
        ) -> Result<eldritch_core::Value, alloc::string::String> {
            match name {
                #(#method_dispatches)*
                _ => {
                    let mut msg = alloc::format!("Method '{}' not found or not exposed", name);
                    let candidates = self._eldritch_method_names();
                    if let Some(suggestion) = eldritch_core::introspection::find_best_match(name, &candidates) {
                        use core::fmt::Write;
                        let _ = write!(msg, "\nDid you mean '{}'?", suggestion);
                    }
                    Err(msg)
                }
            }
        }
    });

    Ok(quote!(#trait_def))
}

/// Expands the `#[eldritch_library_impl]` attribute.
///
/// This macro implements `eldritch_core::ForeignValue` for the struct,
/// delegating to the `_eldritch_*` methods injected into the trait.
pub fn expand_eldritch_library_impl(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let struct_def: ItemStruct = syn::parse2(item)?;
    let struct_name = &struct_def.ident;
    let (impl_generics, ty_generics, where_clause) = struct_def.generics.split_for_impl();

    let trait_ident: syn::Ident = syn::parse2(attr)?;

    Ok(quote! {
        #struct_def

        impl #impl_generics eldritch_core::ForeignValue for #struct_name #ty_generics #where_clause {
            fn type_name(&self) -> &str {
                <Self as #trait_ident>::_eldritch_type_name(self)
            }

            fn method_names(&self) -> alloc::vec::Vec<alloc::string::String> {
                <Self as #trait_ident>::_eldritch_method_names(self)
            }

            fn get_method_signature(&self, name: &str) -> Option<eldritch_core::MethodSignature> {
                <Self as #trait_ident>::_eldritch_get_method_signature(self, name)
            }

            fn call_method(
                &self,
                interp: &mut eldritch_core::Interpreter,
                name: &str,
                args: &[eldritch_core::Value],
                kwargs: &alloc::collections::BTreeMap<alloc::string::String, eldritch_core::Value>,
            ) -> Result<eldritch_core::Value, alloc::string::String> {
                <Self as #trait_ident>::_eldritch_call_method(self, interp, name, args, kwargs)
            }
        }
    })
}

#[allow(clippy::collapsible_if)]
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn is_interpreter_type(ty: &Type) -> bool {
    // Check if type is `Interpreter`, `&Interpreter`, or `&mut Interpreter`
    // Or fully qualified `eldritch_core::Interpreter`
    if let Type::Reference(type_ref) = ty
        && let Type::Path(type_path) = &*type_ref.elem
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Interpreter";
    }
    false
}

fn generate_args_parsing(sig: &Signature) -> Result<(TokenStream, TokenStream), syn::Error> {
    let mut parsing = Vec::new();
    let mut call_args = Vec::new();
    let mut param_names_str = Vec::new();
    let mut arg_idx = 0usize;

    for input in &sig.inputs {
        match input {
            FnArg::Receiver(_) => continue,
            FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                let ty = &pat_type.ty;

                // Check if this argument is the Interpreter
                if is_interpreter_type(ty) {
                    // Pass the interpreter instance
                    call_args.push(quote!(interp));
                    continue;
                }

                let arg_name_str = quote!(#pat).to_string();
                param_names_str.push(arg_name_str.clone());

                // Detect &str
                let is_str_ref = if let Type::Reference(TypeReference { elem, .. }) = &**ty {
                    if let Type::Path(p) = &**elem {
                        p.path.is_ident("str")
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Add validation for multiple values (positional AND keyword)
                parsing.push(quote! {
                    if #arg_idx < _eldritch_args.len() && _eldritch_kwargs.contains_key(#arg_name_str) {
                         return Err(alloc::format!("TypeError: Function got multiple values for argument '{}'", #arg_name_str));
                    }
                });

                if is_str_ref {
                    parsing.push(quote! {
                        let #pat: alloc::string::String = if #arg_idx < _eldritch_args.len() {
                            eldritch_core::conversion::FromValue::from_value(&_eldritch_args[#arg_idx])?
                        } else if let Some(val) = _eldritch_kwargs.get(#arg_name_str) {
                            eldritch_core::conversion::FromValue::from_value(val)?
                        } else {
                            return Err(alloc::format!("Missing argument: {}", #arg_name_str));
                        };
                    });
                    call_args.push(quote!(&#pat));
                } else {
                    let missing_handler = if is_option_type(ty) {
                        quote! { Default::default() }
                    } else {
                        quote! { return Err(alloc::format!("Missing argument: {}", #arg_name_str)); }
                    };

                    parsing.push(quote! {
                        let #pat: #ty = if #arg_idx < _eldritch_args.len() {
                            eldritch_core::conversion::FromValue::from_value(&_eldritch_args[#arg_idx])?
                        } else if let Some(val) = _eldritch_kwargs.get(#arg_name_str) {
                            eldritch_core::conversion::FromValue::from_value(val)?
                        } else {
                            #missing_handler
                        };
                    });
                    call_args.push(quote!(#pat));
                }
                arg_idx += 1;
            }
        }
    }

    // Validate argument counts and unexpected keywords
    let max_pos = arg_idx;
    let mut validation = Vec::new();

    validation.push(quote! {
        if _eldritch_args.len() > #max_pos {
             return Err(alloc::format!("TypeError: Function got too many arguments. Expected {}, got {}", #max_pos, _eldritch_args.len()));
        }
    });

    // Validate keywords - handle empty case
    if param_names_str.is_empty() {
        validation.push(quote! {
             if !_eldritch_kwargs.is_empty() {
                 // Get first key to report
                 let key = _eldritch_kwargs.keys().next().unwrap();
                 return Err(alloc::format!("TypeError: Function got an unexpected keyword argument '{}'", key));
             }
        });
    } else {
        validation.push(quote! {
             for key in _eldritch_kwargs.keys() {
                 let is_valid = match key.as_str() {
                     #(#param_names_str)|* => true,
                     _ => false
                 };
                 if !is_valid {
                      return Err(alloc::format!("TypeError: Function got an unexpected keyword argument '{}'", key));
                 }
             }
        });
    }

    // Prepend validation to parsing logic
    let mut final_parsing = Vec::new();
    final_parsing.extend(validation);
    final_parsing.push(quote! { #(#parsing)* });

    Ok((quote! { #(#final_parsing)* }, quote! { #(#call_args),* }))
}

fn generate_signature(sig: &Signature, bind_name: &str) -> Result<TokenStream, syn::Error> {
    let mut params = Vec::new();

    for input in &sig.inputs {
        match input {
            FnArg::Receiver(_) => continue,
            FnArg::Typed(pat_type) => {
                let ty = &pat_type.ty;
                if is_interpreter_type(ty) {
                    continue;
                }

                let pat = &pat_type.pat;
                let arg_name_str = quote!(#pat).to_string();
                let is_optional = is_option_type(ty);
                let type_name_str = quote!(#ty).to_string(); // Simple string representation for now

                params.push(quote! {
                    eldritch_core::ParameterSignature {
                        name: alloc::string::String::from(#arg_name_str),
                        type_name: Some(alloc::string::String::from(#type_name_str)),
                        is_optional: #is_optional,
                        is_variadic: false,
                        is_kwargs: false,
                    }
                });
            }
        }
    }

    Ok(quote! {
        eldritch_core::MethodSignature {
            name: alloc::string::String::from(#bind_name),
            params: alloc::vec![#(#params),*],
            return_type: None, // TODO: Inspect return type if needed
        }
    })
}
