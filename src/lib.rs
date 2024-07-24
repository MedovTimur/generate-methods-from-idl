use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};
use std::fs;

#[proc_macro]
pub fn generate_methods_from_idl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let idl_path = input.value();
    
    let idl_content = fs::read_to_string(idl_path).expect("Failed to read the IDL file");
    
    let re_constructor = regex::Regex::new(r"constructor\s*\{\s*New\s*:\s*\(([^)]*)\);\s*}")
        .expect("Incorrect regular expression");
    
    let captures = re_constructor
        .captures(&idl_content)
        .expect("Couldn't find the constructor");
    
    let params = &captures[1];
    
    let re_params = regex::Regex::new(r"(\w+)\s*:\s*(\w+)")
        .expect("Incorrect regular expression");
    
    let mut param_names = Vec::new();
    let mut param_types = Vec::new();
    
    for cap in re_params.captures_iter(params) {
        param_names.push(cap[1].to_string());
        param_types.push(cap[2].to_string());
    }
    
    let param_names_ident: Vec<_> = param_names.iter().map(|name| syn::Ident::new(name, proc_macro2::Span::call_site())).collect();
    let param_types_rust: Vec<_> = param_types.iter().map(|typ| map_type(typ)).collect();

    let re_service = regex::Regex::new(r"(?s)service\s+(\w+)\s*\{([^}]*)\}")
        .expect("Incorrect regular expression");

    let service_caps = re_service
        .captures(&idl_content)
        .expect("Couldn't find the service");

    let service_name = service_caps[1].to_string();
    let service_methods = &service_caps[2];

    // let re_service_methods = regex::Regex::new(r"(\w+)\s*:\s*\(([^)]*)\)\s*->\s*(\w+);").expect("Неправильное регулярное выражение");
    let re_service_methods = regex::Regex::new(r"(\w+)\s*:\s*\(([^)]*)\)\s*")
        .expect("Incorrect regular expression");

    let mut method_impls = Vec::new();

    for cap in re_service_methods.captures_iter(service_methods) {
        let method_name = cap[1].to_string();
        let method_params = cap[2].to_string();
        // let method_return_type = map_type(&cap[3]);

        if method_name.starts_with("query") {
            continue;
        }

        let method_name_lower = method_name.to_lowercase();
        let method_name_ident = syn::Ident::new(&method_name_lower, proc_macro2::Span::call_site());
        let method_param_names = method_params.split(",").map(|s| s.trim().split(":").next().unwrap().to_string()).collect::<Vec<_>>();
        let method_param_types = method_params.split(",").map(|s| s.trim().split(":").nth(1).unwrap().to_string()).collect::<Vec<_>>();

        let method_param_names_ident: Vec<_> = method_param_names.iter().map(|name| syn::Ident::new(name, proc_macro2::Span::call_site())).collect();
        let method_param_types_rust: Vec<_> = method_param_types.iter().map(|typ| map_type(typ)).collect();

        let method_impl = quote! {
            pub fn #method_name_ident(&self, from_id: u64, #( #method_param_names_ident: #method_param_types_rust ),*) {
                let request = [
                    #service_name.encode(),
                    #method_name.encode(),
                    (#(#method_param_names_ident),*).encode(),
                ]
                .concat();
                self.send_bytes(from_id, request);
            }
        };
        method_impls.push(method_impl);
    }
    
    let expanded = quote! {
        use gstd::Encode;
        impl Program {
            pub fn init(&self, from_id: u64, #( #param_names_ident: #param_types_rust ),*) {
                let init_request = [
                    "New".encode(),
                    (#(#param_names_ident),*).encode(),
                ]
                .concat();
                self.send_bytes(from_id, init_request);
            }

            #( #method_impls )*
        }
    };
    
    TokenStream::from(expanded)
}

fn map_type(idl_type: &str) -> proc_macro2::TokenStream {
    match idl_type.trim() {
        "str" => quote! { String },
        "u8" => quote! { u8 },
        "actor_id" => quote! { u64 }, //Replace with the appropriate type for actor_id
        _ => panic!("Unknown type: {}", idl_type),
    }
}
