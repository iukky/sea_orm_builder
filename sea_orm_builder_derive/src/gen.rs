//! Code generation for Select/Update/Delete builders.
//!
//! This module consumes the parsed model info from `ast` and produces the
//! builder structs, methods and the Params snapshot types.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::ast::{collect, to_camel, ModelInfoField};

/// Which builder kind to generate.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    Select,
    Update,
    Delete,
}

/// Entry point used by the proc-macro functions in lib.rs
pub fn expand(input: TokenStream, mode: Mode) -> TokenStream {
    let di = syn::parse_macro_input!(input as DeriveInput);
    let (entity_prefix, fields) = match collect(&di) {
        Ok(v) => v,
        Err(err) => return err.to_compile_error().into(),
    };

    let name_prefix = format_ident!("{}", entity_prefix);
    let (builder_struct, builder_impl) = match mode {
        Mode::Select => {
            let name = format_ident!("{}Select", name_prefix);
            build_select(&name, &fields)
        }
        Mode::Update => {
            let name = format_ident!("{}Update", name_prefix);
            build_update(&name, &fields)
        }
        Mode::Delete => {
            let name = format_ident!("{}Delete", name_prefix);
            build_delete(&name, &fields)
        }
    };

    let out = quote! {
        #builder_struct
        #builder_impl
    };
    out.into()
}

pub fn build_select(
    name: &syn::Ident,
    fields: &Vec<ModelInfoField>,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut methods = vec![];
    let mut storages = vec![];
    let mut inits = vec![];
    let mut accessors = vec![];
    let mut params_accessors = vec![];
    let mut move_fields = vec![];
    for f in fields {
        for op in &f.perms.select_where {
            let op_str = op.as_str();
            let (s, i, m, a) = gen_where_pieces(&f.ident, &f.ty, op_str);
            storages.push(s);
            inits.push(i);
            methods.push(m);
            accessors.push(a);
            let field_name = f.ident.to_string();
            let storage_ident = format_ident!("{}_{}_val", field_name, op_str);
            move_fields.push(quote! { #storage_ident: self.#storage_ident });
            params_accessors.push(gen_params_accessors(&f.ident, &f.ty, op_str));
        }
    }
    let st = quote! {
        pub struct #name {
            pub statement: ::sea_orm_builder::gen::Select<Entity>,
            has_where: bool,
            where_params: ::std::vec::Vec<::sea_orm_builder::WhereParam>,
            #(#storages,)*
        }
    };
    let params_name = format_ident!("{}Params", name);
    let params_struct = quote! {
        pub struct #params_name {
            pub where_params: ::std::vec::Vec<::sea_orm_builder::WhereParam>,
            #(#storages,)*
        }
        impl #params_name {
            #(#params_accessors)*
            pub fn where_params(&self) -> &[::sea_orm_builder::WhereParam] { &self.where_params }
        }
    };
    let imp = quote! {
        impl #name {
            pub fn new() -> Self { Self { statement: Entity::find(), has_where: false, where_params: ::std::vec::Vec::new(), #(#inits,)* } }
            pub fn order_by_asc(mut self, col: Column) -> Self {
                self.statement = <::sea_orm_builder::gen::Select<Entity> as ::sea_orm_builder::gen::QueryOrder>::order_by(
                    self.statement,
                    col,
                    ::sea_orm_builder::gen::Order::Asc,
                );
                self
            }
            pub fn order_by_desc(mut self, col: Column) -> Self {
                self.statement = <::sea_orm_builder::gen::Select<Entity> as ::sea_orm_builder::gen::QueryOrder>::order_by(
                    self.statement,
                    col,
                    ::sea_orm_builder::gen::Order::Desc,
                );
                self
            }
            pub fn limit(mut self, limit: u64) -> Self {
                self.statement = <::sea_orm_builder::gen::Select<Entity> as ::sea_orm_builder::gen::QuerySelect>::limit(
                    self.statement,
                    limit,
                );
                self
            }
            pub fn offset(mut self, offset: u64) -> Self {
                self.statement = <::sea_orm_builder::gen::Select<Entity> as ::sea_orm_builder::gen::QuerySelect>::offset(
                    self.statement,
                    offset,
                );
                self
            }
            #(#methods)*
            #(#accessors)*
            pub fn build(self) -> ::sea_orm_builder::gen::Select<Entity> { self.statement }
            pub fn build_with_params(self) -> (::sea_orm_builder::gen::Select<Entity>, #params_name) {
                let p = #params_name { where_params: self.where_params, #(#move_fields,)* };
                (self.statement, p)
            }
            pub fn where_params(&self) -> &[::sea_orm_builder::WhereParam] { &self.where_params }
        }
    };
    (quote! { #st #params_struct }, imp)
}

pub fn build_update(
    name: &syn::Ident,
    fields: &Vec<ModelInfoField>,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut where_methods = vec![];
    let mut set_methods = vec![];
    let mut storages = vec![];
    let mut inits = vec![];
    let mut accessors = vec![];
    let mut params_accessors = vec![];
    let mut move_fields = vec![];
    for f in fields {
        for op in &f.perms.update_where {
            let op_str = op.as_str();
            let (s, i, m, a) = gen_where_pieces(&f.ident, &f.ty, op_str);
            storages.push(s);
            inits.push(i);
            where_methods.push(m);
            accessors.push(a);
            let field_name = f.ident.to_string();
            let storage_ident = format_ident!("{}_{}_val", field_name, op_str);
            move_fields.push(quote! { #storage_ident: self.#storage_ident });
            params_accessors.push(gen_params_accessors(&f.ident, &f.ty, op_str));
        }
        if f.perms.update_set {
            set_methods.push(gen_set_method(&f.ident, &f.ty));
        }
    }
    let st = quote! {
        pub struct #name {
            pub statement: ::sea_orm_builder::gen::UpdateMany<Entity>,
            has_where: bool,
            set_count: usize,
            where_params: ::std::vec::Vec<::sea_orm_builder::WhereParam>,
            #(#storages,)*
        }
    };
    let params_name = format_ident!("{}Params", name);
    let params_struct = quote! {
        pub struct #params_name {
            pub where_params: ::std::vec::Vec<::sea_orm_builder::WhereParam>,
            #(#storages,)*
        }
        impl #params_name {
            #(#params_accessors)*
            pub fn where_params(&self) -> &[::sea_orm_builder::WhereParam] { &self.where_params }
        }
    };
    let imp = quote! {
        impl #name {
            pub fn new() -> Self { Self { statement: Entity::update_many(), has_where: false, set_count: 0, where_params: ::std::vec::Vec::new(), #(#inits,)* } }
            #(#set_methods)*
            #(#where_methods)*
            #(#accessors)*
            pub fn build(self) -> Result<::sea_orm_builder::gen::UpdateMany<Entity>, ::sea_orm_builder::SeaOrmBuilderError> {
                if self.set_count == 0 { return Err(::sea_orm_builder::SeaOrmBuilderError::NoSet); }
                if !self.has_where { return Err(::sea_orm_builder::SeaOrmBuilderError::NoWhere); }
                Ok(self.statement)
            }
            pub fn build_with_params(self) -> Result<(::sea_orm_builder::gen::UpdateMany<Entity>, #params_name), ::sea_orm_builder::SeaOrmBuilderError> {
                if self.set_count == 0 { return Err(::sea_orm_builder::SeaOrmBuilderError::NoSet); }
                if !self.has_where { return Err(::sea_orm_builder::SeaOrmBuilderError::NoWhere); }
                let p = #params_name { where_params: self.where_params, #(#move_fields,)* };
                Ok((self.statement, p))
            }
            pub fn where_params(&self) -> &[::sea_orm_builder::WhereParam] { &self.where_params }
        }
    };
    (quote! { #st #params_struct }, imp)
}

pub fn build_delete(
    name: &syn::Ident,
    fields: &Vec<ModelInfoField>,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut where_methods = vec![];
    let mut storages = vec![];
    let mut inits = vec![];
    let mut accessors = vec![];
    let mut params_accessors = vec![];
    let mut move_fields = vec![];
    for f in fields {
        for op in &f.perms.delete_where {
            let op_str = op.as_str();
            let (s, i, m, a) = gen_where_pieces(&f.ident, &f.ty, op_str);
            storages.push(s);
            inits.push(i);
            where_methods.push(m);
            accessors.push(a);
            let field_name = f.ident.to_string();
            let storage_ident = format_ident!("{}_{}_val", field_name, op_str);
            move_fields.push(quote! { #storage_ident: self.#storage_ident });
            params_accessors.push(gen_params_accessors(&f.ident, &f.ty, op_str));
        }
    }
    let st = quote! {
        pub struct #name {
            pub statement: ::sea_orm_builder::gen::DeleteMany<Entity>,
            has_where: bool,
            where_params: ::std::vec::Vec<::sea_orm_builder::WhereParam>,
            #(#storages,)*
        }
    };
    let params_name = format_ident!("{}Params", name);
    let params_struct = quote! {
        pub struct #params_name {
            pub where_params: ::std::vec::Vec<::sea_orm_builder::WhereParam>,
            #(#storages,)*
        }
        impl #params_name {
            #(#params_accessors)*
            pub fn where_params(&self) -> &[::sea_orm_builder::WhereParam] { &self.where_params }
        }
    };
    let imp = quote! {
        impl #name {
            pub fn new() -> Self { Self { statement: Entity::delete_many(), has_where: false, where_params: ::std::vec::Vec::new(), #(#inits,)* } }
            #(#where_methods)*
            #(#accessors)*
            pub fn build(self) -> Result<::sea_orm_builder::gen::DeleteMany<Entity>, ::sea_orm_builder::SeaOrmBuilderError> {
                if !self.has_where { return Err(::sea_orm_builder::SeaOrmBuilderError::NoWhere); }
                Ok(self.statement)
            }
            pub fn build_with_params(self) -> Result<(::sea_orm_builder::gen::DeleteMany<Entity>, #params_name), ::sea_orm_builder::SeaOrmBuilderError> {
                if !self.has_where { return Err(::sea_orm_builder::SeaOrmBuilderError::NoWhere); }
                let p = #params_name { where_params: self.where_params, #(#move_fields,)* };
                Ok((self.statement, p))
            }
            pub fn where_params(&self) -> &[::sea_orm_builder::WhereParam] { &self.where_params }
        }
    };
    (quote! { #st #params_struct }, imp)
}

fn gen_where_pieces(
    field_ident: &syn::Ident,
    field_ty: &syn::Type,
    op: &str,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let field_name = field_ident.to_string();
    let method_ident = match op {
        "in" => format_ident!("{}_in", field_name),
        o => format_ident!("{}_{}", field_name, o),
    };
    let column_variant = format_ident!("{}", to_camel(&field_name));
    let storage_ident = format_ident!("{}_{}_val", field_name, op);

    match op {
        "eq" | "ne" | "lt" | "lte" | "gt" | "gte" | "like" | "ilike" => {
            let op_ident = format_ident!("{}", op);
            let storage = quote! { #storage_ident: ::std::option::Option<#field_ty> };
            let init = quote! { #storage_ident: ::std::option::Option::None };
            let is_ident = format_ident!("is_{}_{}", field_name, op);
            let get_ident = format_ident!("get_{}_{}", field_name, op);
            let method = quote! {
                pub fn #method_ident<V: ::sea_orm_builder::IntoField<#field_ty>>(mut self, v: V) -> Self where #field_ty: ::std::clone::Clone {
                    let vv: #field_ty = v.into_field();
                    self.#storage_ident = ::std::option::Option::Some(vv.clone());
                    self.statement = self.statement.filter(Column::#column_variant.#op_ident(vv));
                    self.has_where = true;
                    self.where_params.push(::sea_orm_builder::WhereParam { field: #field_name, op: #op, value: ::sea_orm_builder::WhereValue::Single(format!("{:?}", &self.#storage_ident)) });
                    self
                }
            };
            let accessor = quote! {
                pub fn #is_ident(&self) -> bool { self.#storage_ident.is_some() }
                pub fn #get_ident(&self) -> ::std::option::Option<&#field_ty> { self.#storage_ident.as_ref() }
            };
            (storage, init, method, accessor)
        }
        "in" => {
            let storage =
                quote! { #storage_ident: ::std::option::Option<::std::vec::Vec<#field_ty>> };
            let init = quote! { #storage_ident: ::std::option::Option::None };
            let is_ident = format_ident!("is_{}_in", field_name);
            let get_ident = format_ident!("get_{}_in", field_name);
            let method = quote! {
                pub fn #method_ident<V: ::sea_orm_builder::IntoField<#field_ty>, I: IntoIterator<Item = V>>(mut self, iter: I) -> Self where #field_ty: ::std::clone::Clone {
                    let vec_tmp: ::std::vec::Vec<#field_ty> = iter.into_iter().map(|x| x.into_field()).collect();
                    self.#storage_ident = ::std::option::Option::Some(vec_tmp.clone());
                    self.statement = self.statement.filter(Column::#column_variant.is_in(vec_tmp));
                    self.has_where = true;
                    self.where_params.push(::sea_orm_builder::WhereParam { field: #field_name, op: #op, value: ::sea_orm_builder::WhereValue::List(self.#storage_ident.as_ref().unwrap().iter().map(|x| format!("{:?}", x)).collect()) });
                    self
                }
            };
            let accessor = quote! {
                pub fn #is_ident(&self) -> bool { self.#storage_ident.is_some() }
                pub fn #get_ident(&self) -> ::std::option::Option<&[#field_ty]> { self.#storage_ident.as_deref().map(|v| &v[..]) }
            };
            (storage, init, method, accessor)
        }
        "not_in" => {
            let storage =
                quote! { #storage_ident: ::std::option::Option<::std::vec::Vec<#field_ty>> };
            let init = quote! { #storage_ident: ::std::option::Option::None };
            let is_ident = format_ident!("is_{}_not_in", field_name);
            let get_ident = format_ident!("get_{}_not_in", field_name);
            let method = quote! {
                pub fn #method_ident<V: ::sea_orm_builder::IntoField<#field_ty>, I: IntoIterator<Item = V>>(mut self, iter: I) -> Self where #field_ty: ::std::clone::Clone {
                    let vec_tmp: ::std::vec::Vec<#field_ty> = iter.into_iter().map(|x| x.into_field()).collect();
                    self.#storage_ident = ::std::option::Option::Some(vec_tmp.clone());
                    self.statement = self.statement.filter(Column::#column_variant.is_not_in(vec_tmp));
                    self.has_where = true;
                    self.where_params.push(::sea_orm_builder::WhereParam { field: #field_name, op: #op, value: ::sea_orm_builder::WhereValue::List(self.#storage_ident.as_ref().unwrap().iter().map(|x| format!("{:?}", x)).collect()) });
                    self
                }
            };
            let accessor = quote! {
                pub fn #is_ident(&self) -> bool { self.#storage_ident.is_some() }
                pub fn #get_ident(&self) -> ::std::option::Option<&[#field_ty]> { self.#storage_ident.as_deref().map(|v| &v[..]) }
            };
            (storage, init, method, accessor)
        }
        "between" => {
            let storage = quote! { #storage_ident: ::std::option::Option<(#field_ty, #field_ty)> };
            let init = quote! { #storage_ident: ::std::option::Option::None };
            let is_ident = format_ident!("is_{}_between", field_name);
            let get_ident = format_ident!("get_{}_between", field_name);
            let method = quote! {
                pub fn #method_ident<V1: ::sea_orm_builder::IntoField<#field_ty>, V2: ::sea_orm_builder::IntoField<#field_ty>>(mut self, a: V1, b: V2) -> Self where #field_ty: ::std::clone::Clone {
                    let a: #field_ty = a.into_field();
                    let b: #field_ty = b.into_field();
                    self.#storage_ident = ::std::option::Option::Some((a.clone(), b.clone()));
                    self.statement = self.statement.filter(Column::#column_variant.between(a, b));
                    self.has_where = true;
                    if let ::std::option::Option::Some((ref sa, ref sb)) = self.#storage_ident {
                        self.where_params.push(::sea_orm_builder::WhereParam { field: #field_name, op: #op, value: ::sea_orm_builder::WhereValue::Range { start: format!("{:?}", sa), end: format!("{:?}", sb) } });
                    }
                    self
                }
            };
            let accessor = quote! {
                pub fn #is_ident(&self) -> bool { self.#storage_ident.is_some() }
                pub fn #get_ident(&self) -> ::std::option::Option<(&#field_ty, &#field_ty)> { self.#storage_ident.as_ref().map(|(a,b)| (a,b)) }
            };
            (storage, init, method, accessor)
        }
        _ => {
            let msg = format!("unsupported op: {}", op);
            (
                quote! {},
                quote! {},
                quote! { const _: () = { compile_error!(#msg); }; },
                quote! {},
            )
        }
    }
}

fn gen_set_method(field_ident: &syn::Ident, field_ty: &syn::Type) -> proc_macro2::TokenStream {
    let method_ident = format_ident!("set_{}", field_ident);
    let column_variant = format_ident!("{}", to_camel(&field_ident.to_string()));
    quote! {
        pub fn #method_ident<V: ::sea_orm_builder::IntoField<#field_ty>>(mut self, v: V) -> Self {
            let v: #field_ty = v.into_field();
            self.statement = self.statement.col_expr(Column::#column_variant, ::sea_orm_builder::gen::Expr::value(v));
            self.set_count += 1;
            self
        }
    }
}

fn gen_params_accessors(
    field_ident: &syn::Ident,
    field_ty: &syn::Type,
    op: &str,
) -> proc_macro2::TokenStream {
    let field_name = field_ident.to_string();
    let storage_ident = format_ident!("{}_{}_val", field_name, op);
    match op {
        "eq" | "ne" | "lt" | "lte" | "gt" | "gte" | "like" | "ilike" => {
            let is_ident = format_ident!("is_{}_{}", field_name, op);
            let get_ident = format_ident!("get_{}_{}", field_name, op);
            quote! {
                pub fn #is_ident(&self) -> bool { self.#storage_ident.is_some() }
                pub fn #get_ident(&self) -> ::std::option::Option<&#field_ty> { self.#storage_ident.as_ref() }
            }
        }
        "in" => {
            let is_ident = format_ident!("is_{}_in", field_name);
            let get_ident = format_ident!("get_{}_in", field_name);
            quote! {
                pub fn #is_ident(&self) -> bool { self.#storage_ident.is_some() }
                pub fn #get_ident(&self) -> ::std::option::Option<&[#field_ty]> { self.#storage_ident.as_deref().map(|v| &v[..]) }
            }
        }
        "not_in" => {
            let is_ident = format_ident!("is_{}_not_in", field_name);
            let get_ident = format_ident!("get_{}_not_in", field_name);
            quote! {
                pub fn #is_ident(&self) -> bool { self.#storage_ident.is_some() }
                pub fn #get_ident(&self) -> ::std::option::Option<&[#field_ty]> { self.#storage_ident.as_deref().map(|v| &v[..]) }
            }
        }
        "between" => {
            let is_ident = format_ident!("is_{}_between", field_name);
            let get_ident = format_ident!("get_{}_between", field_name);
            quote! {
                pub fn #is_ident(&self) -> bool { self.#storage_ident.is_some() }
                pub fn #get_ident(&self) -> ::std::option::Option<(&#field_ty, &#field_ty)> { self.#storage_ident.as_ref().map(|(a,b)| (a,b)) }
            }
        }
        _ => quote! {},
    }
}
