use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Field, Fields, FnArg, ImplItem, ItemImpl, ItemStruct, LitStr, Meta};

const HTTP_METHODS: &[&str] = &["get", "post", "put", "delete", "patch", "head", "options"];

// --- MACRO: Injectable ---

fn is_field_default(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        if attr.path().is_ident("inject") {
            if let Meta::List(list) = &attr.meta {
                if let Ok(ident) = syn::parse2::<syn::Ident>(list.tokens.clone()) {
                    ident == "default"
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    })
}

fn extract_inject_type(ty: &syn::Type) -> Option<syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Inject" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty.clone());
                    }
                }
            }
        }
    }
    None
}

#[proc_macro_derive(Injectable, attributes(inject))]
pub fn derive_injectable(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    let mut bounds = Vec::new();
    
    let create_body = match &input.data {
        syn::Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(named_fields) => {
                    let field_resolutions = named_fields.named.iter().map(|f| {
                        let name = &f.ident;
                        if is_field_default(f) {
                            quote! { #name: ::std::default::Default::default() }
                        } else if let Some(inner_ty) = extract_inject_type(&f.ty) {
                            bounds.push(quote! { C: ::nidum::__private::nidum_core::Provides<#inner_ty> });
                            quote! { #name: ::nidum::__private::nidum_core::Inject::new(container.provide()) }
                        } else {
                            let ty = &f.ty;
                            bounds.push(quote! { C: ::nidum::__private::nidum_core::Provides<#ty> });
                            quote! { #name: container.provide() }
                        }
                    });
                    quote! { ::std::sync::Arc::new(Self { #(#field_resolutions),* }) }
                }
                Fields::Unnamed(unnamed_fields) => {
                    let field_resolutions = unnamed_fields.unnamed.iter().map(|f| {
                        if is_field_default(f) {
                            quote! { ::std::default::Default::default() }
                        } else if let Some(inner_ty) = extract_inject_type(&f.ty) {
                            bounds.push(quote! { C: ::nidum::__private::nidum_core::Provides<#inner_ty> });
                            quote! { ::nidum::__private::nidum_core::Inject::new(container.provide()) }
                        } else {
                            let ty = &f.ty;
                            bounds.push(quote! { C: ::nidum::__private::nidum_core::Provides<#ty> });
                            quote! { container.provide() }
                        }
                    });
                    quote! { ::std::sync::Arc::new(Self(#(#field_resolutions),*)) }
                }
                Fields::Unit => {
                    quote! { ::std::sync::Arc::new(Self) }
                }
            }
        }
        _ => panic!("Injectable can only be derived on structs"),
    };

    let expanded = quote! {
        impl #impl_generics <C> ::nidum::__private::nidum_core::Injectable<C> for #ident #ty_generics 
        where 
            #(#bounds,)*
        {
            fn create(container: &C) -> ::std::sync::Arc<Self> {
                #create_body
            }
        }
    };

    TokenStream::from(expanded)
}

// --- MACRO: Controller ---

struct RouteDefinition {
    http_method: proc_macro2::Ident,
    path: String,
    fn_ident: proc_macro2::Ident,
    is_async: bool,
    arg_names: Vec<proc_macro2::Ident>,
    arg_types: Vec<syn::Type>,
    guards: Vec<syn::Path>,
    interceptors: Vec<syn::Path>,
}

fn extract_paths_from_attrs(attrs: &mut Vec<syn::Attribute>, attr_name: &str) -> Vec<syn::Path> {
    let mut guards = Vec::new();
    let mut indices_to_remove = Vec::new();

    for (i, attr) in attrs.iter().enumerate() {
        if attr.path().is_ident(attr_name) {
            indices_to_remove.push(i);
            if let Meta::List(list) = &attr.meta {
                if let Ok(parsed_paths) = list.parse_args_with(syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated) {
                    for path in parsed_paths {
                        guards.push(path);
                    }
                }
            }
        }
    }

    for i in indices_to_remove.into_iter().rev() {
        attrs.remove(i);
    }

    guards
}

#[proc_macro_attribute]
pub fn controller(args: TokenStream, input: TokenStream) -> TokenStream {
    let base_path = if args.is_empty() {
        "".to_string()
    } else {
        match syn::parse::<LitStr>(args) {
            Ok(lit) => lit.value(),
            Err(_) => "".to_string(),
        }
    };

    let mut item_impl = syn::parse_macro_input!(input as ItemImpl);
    let self_ty = item_impl.self_ty.clone();

    let global_guards = extract_paths_from_attrs(&mut item_impl.attrs, "use_guards");
    let global_interceptors = extract_paths_from_attrs(&mut item_impl.attrs, "use_interceptors");
    let mut routes: Vec<RouteDefinition> = Vec::new();
    let mut all_guard_bounds = Vec::new();
    let mut all_interceptor_bounds = Vec::new();

    for item in &mut item_impl.items {
        if let ImplItem::Fn(ref mut method) = item {
            let method_guards = extract_paths_from_attrs(&mut method.attrs, "use_guards");
            let method_interceptors = extract_paths_from_attrs(&mut method.attrs, "use_interceptors");
            
            let mut route_attr_index = None;
            let mut matched_http_method = None;
            let mut route_path = "/".to_string();

            for (idx, attr) in method.attrs.iter().enumerate() {
                for &http_m in HTTP_METHODS {
                    if attr.path().is_ident(http_m) {
                        route_attr_index = Some(idx);
                        matched_http_method = Some(http_m);

                        route_path = match &attr.meta {
                            Meta::Path(_) => "/".to_string(),
                            Meta::List(list) => {
                                if let Ok(lit) = syn::parse2::<LitStr>(list.tokens.clone()) { lit.value() } else { "/".to_string() }
                            }
                            Meta::NameValue(nv) => {
                                if let syn::Expr::Lit(expr_lit) = &nv.value {
                                    if let syn::Lit::Str(lit) = &expr_lit.lit { lit.value() } else { "/".to_string() }
                                } else { "/".to_string() }
                            }
                        };
                        break;
                    }
                }
                if route_attr_index.is_some() { break; }
            }

            if let (Some(idx), Some(http_m)) = (route_attr_index, matched_http_method) {
                method.attrs.remove(idx);

                let fn_ident = method.sig.ident.clone();
                let is_async = method.sig.asyncness.is_some();

                let mut arg_names = Vec::new();
                let mut arg_types = Vec::new();
                let mut arg_counter: usize = 0;

                for input_arg in &method.sig.inputs {
                    match input_arg {
                        FnArg::Receiver(_) => {}
                        FnArg::Typed(pat_type) => {
                            let arg_ident = format_ident!("__nidum_arg_{}", arg_counter);
                            arg_counter += 1;
                            arg_names.push(arg_ident);
                            arg_types.push((*pat_type.ty).clone());
                        }
                    }
                }

                let mut all_guards = global_guards.clone();
                all_guards.extend(method_guards);

                let mut all_interceptors = global_interceptors.clone();
                all_interceptors.extend(method_interceptors);

                for g in &all_guards {
                    all_guard_bounds.push(quote! { C: ::nidum::__private::nidum_core::Provides<#g> });
                }

                for i in &all_interceptors {
                    all_interceptor_bounds.push(quote! { C: ::nidum::__private::nidum_core::Provides<#i> });
                }

                routes.push(RouteDefinition {
                    http_method: format_ident!("{}", http_m),
                    path: route_path,
                    fn_ident,
                    is_async,
                    arg_names,
                    arg_types,
                    guards: all_guards,
                    interceptors: all_interceptors,
                });
            }
        }
    }

    let route_registrations = routes.iter().map(|route| {
        let http_method = &route.http_method;
        let subpath = &route.path;
        let fn_ident = &route.fn_ident;
        let arg_names = &route.arg_names;
        let arg_types = &route.arg_types;
        let guards = &route.guards;
        let interceptors = &route.interceptors;

        let call_expr = if route.is_async {
            quote! { controller.#fn_ident(#(#arg_names),*).await }
        } else {
            quote! { controller.#fn_ident(#(#arg_names),*) }
        };

        let guard_layers = guards.iter().map(|guard| {
            quote! {
                .layer(::nidum::__private::axum::middleware::from_fn_with_state(
                    ::nidum::__private::nidum_core::Provides::<#guard>::provide(container),
                    ::nidum::__private::nidum_core::guard_middleware::<#guard>
                ))
            }
        });

        let interceptor_layers = interceptors.iter().map(|interceptor| {
            quote! {
                .layer(::nidum::__private::axum::middleware::from_fn_with_state(
                    ::nidum::__private::nidum_core::Provides::<#interceptor>::provide(container),
                    ::nidum::__private::nidum_core::interceptor_middleware::<#interceptor>
                ))
            }
        });

        quote! {
            {
                let controller = controller.clone();
                let full_path = ::nidum::__private::nidum_core::combine_paths(#base_path, #subpath);
                
                let method_router = ::nidum::__private::axum::routing::#http_method(move |#(#arg_names: #arg_types),*| {
                    let controller = controller.clone();
                    async move {
                        #call_expr
                    }
                })
                #(#interceptor_layers)*
                #(#guard_layers)*;

                router = router.route(&full_path, method_router);
            }
        }
    });

    let (impl_generics, ty_generics, _) = item_impl.generics.split_for_impl();

    let expanded = quote! {
        #item_impl

        impl #impl_generics <C> ::nidum::__private::nidum_core::Controller<C> for #self_ty #ty_generics 
        where 
            #(#all_guard_bounds,)*
            #(#all_interceptor_bounds,)*
        {
            fn base_path() -> &'static str {
                #base_path
            }

            fn build_router(self: ::std::sync::Arc<Self>, container: &C) -> ::nidum::__private::axum::Router {
                let mut router = ::nidum::__private::axum::Router::new();
                let controller = self;

                #(#route_registrations)*

                router
            }
        }
    };

    TokenStream::from(expanded)
}

// --- MACRO: Module ---

#[derive(Default)]
struct ModuleArgs {
    imports: Vec<syn::Path>,
    controllers: Vec<syn::Path>,
    providers: Vec<syn::Path>,
}

impl syn::parse::Parse for ModuleArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = ModuleArgs::default();
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;

            let content;
            syn::bracketed!(content in input);
            let paths = content.parse_terminated(syn::Path::parse, syn::Token![,])?;
            let path_vec: Vec<syn::Path> = paths.into_iter().collect();

            match key.to_string().as_str() {
                "imports" => args.imports = path_vec,
                "controllers" => args.controllers = path_vec,
                "providers" => args.providers = path_vec,
                "exports" => {}, // Ignored in static DI (everything in the container is public)
                _other => {
                    return Err(syn::Error::new(key.span(), "Unknown module property"));
                }
            }

            if input.peek(syn::Token![,]) {
                let _comma: syn::Token![,] = input.parse()?;
            }
        }
        Ok(args)
    }
}

#[proc_macro_attribute]
pub fn module(args: TokenStream, input: TokenStream) -> TokenStream {
    let module_args = syn::parse_macro_input!(args as ModuleArgs);
    let item_struct = syn::parse_macro_input!(input as ItemStruct);
    let ident = &item_struct.ident;

    let container_ident = format_ident!("{}Container", ident);
    
    let providers = &module_args.providers;
    let controllers = &module_args.controllers;
    let imports = &module_args.imports;

    let provider_fields = providers.iter().map(|p| {
        let field_name = format_ident!("{}", p.segments.last().unwrap().ident.to_string().to_lowercase());
        quote! { #field_name: ::std::sync::OnceLock<::std::sync::Arc<#p>> }
    });

    let import_fields = imports.iter().map(|i| {
        let field_name = format_ident!("{}", i.segments.last().unwrap().ident.to_string().to_lowercase());
        quote! { #field_name: ::std::sync::Arc<<#i as ::nidum::__private::nidum_core::Module>::Container> }
    });

    let provider_inits = providers.iter().map(|p| {
        let field_name = format_ident!("{}", p.segments.last().unwrap().ident.to_string().to_lowercase());
        quote! { #field_name: ::std::sync::OnceLock::new() }
    });

    let import_inits = imports.iter().map(|i| {
        let field_name = format_ident!("{}", i.segments.last().unwrap().ident.to_string().to_lowercase());
        quote! { #field_name: ::std::sync::Arc::new(<<#i as ::nidum::__private::nidum_core::Module>::Container>::new()) }
    });

    let provider_impls = providers.iter().map(|p| {
        let field_name = format_ident!("{}", p.segments.last().unwrap().ident.to_string().to_lowercase());
        quote! {
            impl ::nidum::__private::nidum_core::Provides<#p> for #container_ident {
                fn provide(&self) -> ::std::sync::Arc<#p> {
                    self.#field_name.get_or_init(|| {
                        <#p as ::nidum::__private::nidum_core::Injectable<Self>>::create(self)
                    }).clone()
                }
            }
        }
    });

    let with_methods = providers.iter().map(|p| {
        let field_name = format_ident!("{}", p.segments.last().unwrap().ident.to_string().to_lowercase());
        let method_name = format_ident!("with_{}", field_name);
        quote! {
            pub fn #method_name(self, instance: ::std::sync::Arc<#p>) -> Self {
                let _ = self.#field_name.set(instance);
                self
            }
        }
    });

    let controller_mounts = controllers.iter().map(|c| {
        quote! {
            let ctrl = <#c as ::nidum::__private::nidum_core::Injectable<Self::Container>>::create(&*container);
            router = router.merge(
                <#c as ::nidum::__private::nidum_core::Controller<Self::Container>>::build_router(ctrl, &*container)
            );
        }
    });

    let import_mounts = imports.iter().map(|i| {
        let field_name = format_ident!("{}", i.segments.last().unwrap().ident.to_string().to_lowercase());
        quote! {
            router = router.merge(<#i as ::nidum::__private::nidum_core::Module>::build_router(container.#field_name.clone()));
        }
    });

    let expanded = quote! {
        #item_struct

        pub struct #container_ident {
            #(#provider_fields,)*
            #(#import_fields,)*
        }

        impl #container_ident {
            pub fn new() -> Self {
                Self {
                    #(#provider_inits,)*
                    #(#import_inits,)*
                }
            }

            #(#with_methods)*
        }

        #(#provider_impls)*

        impl ::nidum::__private::nidum_core::Module for #ident {
            type Container = #container_ident;

            fn build_router(container: ::std::sync::Arc<Self::Container>) -> ::nidum::__private::axum::Router {
                let mut router = ::nidum::__private::axum::Router::new();
                
                #(#controller_mounts)*
                #(#import_mounts)*

                router
            }
        }
    };

    TokenStream::from(expanded)
}

// HTTP Verbs
#[proc_macro_attribute] pub fn get(_args: TokenStream, input: TokenStream) -> TokenStream { input }
#[proc_macro_attribute] pub fn post(_args: TokenStream, input: TokenStream) -> TokenStream { input }
#[proc_macro_attribute] pub fn put(_args: TokenStream, input: TokenStream) -> TokenStream { input }
#[proc_macro_attribute] pub fn delete(_args: TokenStream, input: TokenStream) -> TokenStream { input }
#[proc_macro_attribute] pub fn patch(_args: TokenStream, input: TokenStream) -> TokenStream { input }
#[proc_macro_attribute] pub fn head(_args: TokenStream, input: TokenStream) -> TokenStream { input }
#[proc_macro_attribute] pub fn options(_args: TokenStream, input: TokenStream) -> TokenStream { input }
#[proc_macro_attribute] pub fn use_guards(_args: TokenStream, input: TokenStream) -> TokenStream { input }
#[proc_macro_attribute] pub fn use_interceptors(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream { input }
