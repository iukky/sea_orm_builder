//! AST parsing utilities for sea_orm_builder_derive
//!
//! - Parses the input DeriveInput (SeaORM Model) to a simplified shape
//! - Extracts per-field permissions from `#[sea_builder(...)]`
//! - Derives an entity prefix from `#[sea_orm(table_name = "...")]`

use heck::ToUpperCamelCase;
use syn::{Attribute, Data, DeriveInput, Fields, LitStr};

/// Per-field permissions configured via `#[sea_builder(...)]`.
#[derive(Default, Debug, Clone)]
pub struct FieldPerms {
    pub select_where: Vec<String>,
    pub update_where: Vec<String>,
    pub update_set: bool,
    pub delete_where: Vec<String>,
}

/// Simplified model field info used by codegen.
#[derive(Debug)]
pub struct ModelInfoField {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub perms: FieldPerms,
}

/// Collect entity prefix and fields' permissions from a SeaORM model struct.
pub fn collect(di: &DeriveInput) -> syn::Result<(String, Vec<ModelInfoField>)> {
    // entity prefix from #[sea_orm(table_name = "...")]
    let mut entity_prefix: Option<String> = None;
    for attr in &di.attrs {
        if attr.path().is_ident("sea_orm") {
            // #[sea_orm(table_name = "...")]
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table_name") {
                    let lit: LitStr = meta.value()?.parse()?;
                    entity_prefix = Some(to_camel(&lit.value()));
                }
                Ok(())
            })?;
        }
    }
    let entity_prefix = entity_prefix.unwrap_or_else(|| "Entity".to_string());

    let mut fields_out: Vec<ModelInfoField> = Vec::new();
    let fields = match &di.data {
        Data::Struct(s) => &s.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                di,
                "Select/Update/Delete Builder can only be derived for structs",
            ))
        }
    };
    let named = match fields {
        Fields::Named(n) => &n.named,
        _ => return Err(syn::Error::new_spanned(fields, "Expected named fields")),
    };

    for f in named {
        let ident = f.ident.clone().expect("named");
        let ty = f.ty.clone();
        let perms = parse_sea_builder_attrs(&f.attrs)?;
        fields_out.push(ModelInfoField { ident, ty, perms });
    }
    Ok((entity_prefix, fields_out))
}

fn parse_sea_builder_attrs(attrs: &Vec<Attribute>) -> syn::Result<FieldPerms> {
    let mut perms = FieldPerms::default();
    for attr in attrs {
        if !attr.path().is_ident("sea_builder") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("select") {
                meta.parse_nested_meta(|m2| {
                    if m2.path.is_ident("where") {
                        parse_ops_nested(&m2, &mut perms.select_where)
                    } else {
                        Ok(())
                    }
                })?;
            } else if meta.path.is_ident("update") {
                meta.parse_nested_meta(|m2| {
                    if m2.path.is_ident("where") {
                        parse_ops_nested(&m2, &mut perms.update_where)
                    } else if m2.path.is_ident("set") {
                        perms.update_set = true;
                        Ok(())
                    } else {
                        Ok(())
                    }
                })?;
            } else if meta.path.is_ident("delete") {
                meta.parse_nested_meta(|m2| {
                    if m2.path.is_ident("where") {
                        parse_ops_nested(&m2, &mut perms.delete_where)
                    } else {
                        Ok(())
                    }
                })?;
            }
            Ok(())
        })?;
    }
    Ok(perms)
}

fn parse_ops_nested(meta: &syn::meta::ParseNestedMeta, target: &mut Vec<String>) -> syn::Result<()> {
    meta.parse_nested_meta(|inner| {
        if let Some(ident) = inner.path.get_ident() {
            target.push(ident.to_string());
        }
        Ok(())
    })
}

pub fn to_camel(s: &str) -> String {
    s.to_upper_camel_case()
}
