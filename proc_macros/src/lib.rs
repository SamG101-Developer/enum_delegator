use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::sync::Mutex;
use syn::{parse_macro_input, ItemEnum, ItemTrait, Meta};

// Create a lookup table for the delegator that maps a trait name to a list of stringified methods
// that will be re-parsed and overridden.
static LOOKUP_TABLE: Lazy<Mutex<HashMap<String, Vec<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// The define_delegator macro is placed on a trait. The name of the trait is saved into a register,
/// then the macro DefineDelgation can be applied over an enum, where the methods of the delegator
/// are applied to the enum, with a match-and-dispatch application of each method to each member of
/// the enum.
#[proc_macro_attribute]
pub fn define_delegator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let clone_item = item.clone();
    let input = parse_macro_input!(clone_item as ItemTrait);
    let trait_name = &input.ident;
    let methods = input
        .items
        .iter()
        .filter_map(|item| if let syn::TraitItem::Fn(method) = item { Some(method.sig.clone()) } else { None })
        .collect::<Vec<_>>();

    // Convert the trait name to a string and save the methods in the lookup table.
    let mut lookup_table = LOOKUP_TABLE.lock().unwrap();
    let method_names = methods
        .iter()
        .map(|method| quote! { #method }.to_string())
        .collect::<Vec<_>>();
    lookup_table.insert(trait_name.to_string(), method_names);
    
    item
}


/// The define_delegation macro applies a delegator to an enum. The macro will generate methods for
/// each method on the trait, and will match on the enum to call the method for every variant of the
/// enum.
#[proc_macro_attribute]
pub fn define_delegation(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let enum_name = &input.ident;
    let variants = &input.variants;

    let trait_paths = attr
        .into_iter()
        .filter_map(|token| syn::parse(token.into()).ok())
        .filter_map(|meta| if let Meta::Path(path) = meta { Some(path) } else { None })
        .collect::<Vec<_>>();
    
    // Iterate the traits, then the methods within the traits, then the variants within each method.
    let mut parsed_impls = vec![];
    for path in trait_paths {
        
        let method_list = {
            let mut lookup_table = LOOKUP_TABLE.lock().unwrap();
            if let Some(methods) = lookup_table.get_mut(&path.get_ident().unwrap().to_string()) {
                methods.clone()
            } else {
                panic!("Trait {} not found in lookup table", path.get_ident().unwrap());
            }
        };
        
        let mut parsed_methods = vec![];
        
        for method in method_list {
            // Parse the string into tokens and then into the function signature.
            let method_tokens = method.as_str().parse::<TokenStream>().unwrap();
            let method_sig: syn::Signature = syn::parse(method_tokens).unwrap();
            let method_ident = &method_sig.ident;
            
            let match_arms = variants.iter().map(|variant| {
                let variant_ident = &variant.ident;
                
                // Create a potential argument list based on non-self parameters in the function signature.
                let args = method_sig.inputs.iter().filter_map(|arg| {
                    if let syn::FnArg::Typed(pat_type) = arg { Some(pat_type.pat.clone()) } else { None }
                });
                
                quote! {
                    #enum_name::#variant_ident(inner) => inner.#method_ident(#(#args),*),
                }
            });
            
            // Set the function body to the match arms.
            let method_body = quote! {
                #method_sig {
                    match self {
                        #(#match_arms)*
                    }
                }
            };
            
            parsed_methods.push(method_body);
        }
        
        // Set the impl body to the parsed methods.
        let impl_body = quote! {
            impl #path for #enum_name {
                #(#parsed_methods)*
            }
        };
        parsed_impls.push(impl_body);
    }

    let expanded = quote! {
        #input

        #(#parsed_impls)*
    };

    TokenStream::from(expanded)
}
