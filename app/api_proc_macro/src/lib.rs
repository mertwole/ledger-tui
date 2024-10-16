#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate syn;

use proc_macro2::TokenStream;
use syn::{
    parse_macro_input, FnArg, Ident, ItemTrait, Pat, PatType, ReturnType, TraitItem, Type,
    Visibility,
};

#[proc_macro]
pub fn implement_cache(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut proc_macro_2_input = TokenStream::from(input.clone());

    let trait_info = parse_macro_input!(input as ItemTrait);
    let trait_info = TraitInfo::from_item_trait(trait_info);

    let cache_impl = trait_info.generate_cache_impl();
    proc_macro_2_input.extend(cache_impl);

    proc_macro::TokenStream::from(proc_macro_2_input)
}

struct TraitInfo {
    visibility: Visibility,
    name: Ident,

    methods: Vec<TraitMethodInfo>,
}

struct TraitMethodInfo {
    name: Ident,
    arguments: Vec<ArgumentInfo>,
    return_type: Type,
}

struct ArgumentInfo {
    name: Ident,
    ty: Type,
}

impl TraitInfo {
    fn from_item_trait(item_trait: ItemTrait) -> Self {
        Self {
            visibility: item_trait.vis,
            name: item_trait.ident,

            methods: item_trait
                .items
                .into_iter()
                .map(TraitMethodInfo::from_trait_item)
                .collect(),
        }
    }

    fn generate_cache_impl(&self) -> TokenStream {
        let trait_name = &self.name;
        let vis = &self.visibility;

        let (cache_fields, cache_field_default_assigns, mode_setters, api_method_wrappers): (
            TokenStream,
            TokenStream,
            TokenStream,
            TokenStream,
        ) = itertools::multiunzip(self.methods.iter().map(|method| {
            (
                method.generate_cache_fields(),
                method.generate_cache_field_default_assign(),
                method.generate_mode_setter(),
                method.generate_api_method_wrapper(),
            )
        }));

        quote! {
            #vis mod cache {
                use crate::api::cache_utils::{Mode, ModePlan};
                use super::*;

                pub struct Cache<A: super::#trait_name> {
                    api: A,

                    #cache_fields
                }

                impl<A: super::#trait_name> Cache<A> {
                    pub async fn new(api: A) -> Self {
                        Self {
                            api,

                            #cache_field_default_assigns
                        }
                    }

                    pub fn set_all_modes(&mut self, mode_plan: ModePlan) {
                        #mode_setters
                    }
                }

                impl<A: super::#trait_name> super::#trait_name for Cache<A> {
                    #api_method_wrappers
                }
            }
        }
    }
}

impl TraitMethodInfo {
    fn from_trait_item(trait_item: TraitItem) -> Self {
        match trait_item {
            TraitItem::Fn(fun) => {
                let sig = fun.sig;

                let mut arguments = vec![];
                for arg in sig.inputs {
                    match arg {
                        FnArg::Receiver(_) => {}
                        FnArg::Typed(ty) => {
                            arguments.push(ArgumentInfo::from_pat_type(ty));
                        }
                    }
                }

                let return_type = match sig.output {
                    ReturnType::Type(_, ty) => *ty,
                    _ => unimplemented!(),
                };

                Self {
                    name: sig.ident,
                    arguments,
                    return_type,
                }
            }
            _ => unimplemented!(),
        }
    }

    fn generate_cache_fields(&self) -> TokenStream {
        let name = &self.name;
        let mode_field_name = make_mode_field_name(&self.name);

        let return_type = &self.return_type;

        let arg_types = self
            .arguments
            .iter()
            .map(|arg| remove_type_reference(arg.ty.clone()));
        let args_tuple = quote! { ( #(#arg_types),* ) };

        quote! {
            #[allow(unused_parens)]
            #name: ::std::sync::Mutex<::std::collections::HashMap<#args_tuple, #return_type>>,
            #[allow(unused_parens)]
            #mode_field_name : ::std::sync::Mutex<Mode<#args_tuple>>,
        }
    }

    fn generate_cache_field_default_assign(&self) -> TokenStream {
        let name = &self.name;
        let mode_field_name = make_mode_field_name(&self.name);

        quote! {
            #name: ::std::default::Default::default(),
            #mode_field_name : ::std::default::Default::default(),
        }
    }

    fn generate_mode_setter(&self) -> TokenStream {
        let mode_field_name = make_mode_field_name(&self.name);

        quote! {
            (*self. #mode_field_name .lock().unwrap()) = mode_plan.into_mode();
        }
    }

    fn generate_api_method_wrapper(&self) -> TokenStream {
        let name = &self.name;
        let mode_field_name = make_mode_field_name(&self.name);
        let ret = &self.return_type;
        let arg_tuple = self.generate_arg_tuple();
        let api_call_args = self.generate_api_call_args();

        let args: Vec<_> = self
            .arguments
            .iter()
            .map(ArgumentInfo::generate_argument)
            .collect();

        quote! {
            #[allow(clippy::await_holding_refcell_ref)]
            async fn #name(&self, #(#args),*) -> #ret {
                let api_result = self.api.#name(#api_call_args);
                let api_result = ::std::boxed::Box::pin(api_result);

                let mut cache = self.#name.lock().unwrap();
                let cache = cache.entry(#arg_tuple);

                let mut mode = self.#mode_field_name.lock().unwrap();

                crate::api::cache_utils::use_cache(
                    #arg_tuple,
                    cache,
                    api_result,
                    &mut *mode
                ).await
            }
        }
    }

    fn generate_arg_tuple(&self) -> TokenStream {
        let args = self.arguments.iter().map(ArgumentInfo::generate_name);
        quote! {
            ( #(#args.clone()),* )
        }
    }

    fn generate_api_call_args(&self) -> TokenStream {
        self.arguments
            .iter()
            .map(|info| {
                let arg = info.generate_name();
                let add_clone = !matches!(info.ty, Type::Reference(_));

                if add_clone {
                    quote! { #arg.clone(), }
                } else {
                    quote! { #arg, }
                }
            })
            .collect()
    }
}

impl ArgumentInfo {
    fn from_pat_type(pat_type: PatType) -> Self {
        let name = match *pat_type.pat {
            Pat::Ident(iden) => iden.ident,
            _ => unimplemented!(),
        };

        Self {
            name,
            ty: *pat_type.ty,
        }
    }

    fn generate_argument(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        quote! { #name : #ty }
    }

    fn generate_name(&self) -> TokenStream {
        let name = &self.name;
        quote! { #name }
    }
}

fn remove_type_reference(ty: Type) -> Type {
    match ty {
        Type::Reference(reference) => *reference.elem,
        _ => ty,
    }
}

fn make_mode_field_name(ident: &Ident) -> Ident {
    format_ident!("__{}_mode", ident)
}
