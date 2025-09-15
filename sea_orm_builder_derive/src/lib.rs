//! sea_orm_builder_derive
//!
//! Thin, well-documented proc-macro entry points delegating to
//! - `ast`: parsing SeaORM Model + `#[sea_builder(..)]` attributes
//! - `gen`: code generation for Select/Update/Delete builders
//!
//! Keeping `lib.rs` small makes the crate easier to read and maintain.

mod ast;
mod gen;

use proc_macro::TokenStream;

/// Derive a `<Entity>Select` builder.
#[proc_macro_derive(SelectBuilder, attributes(sea_builder, sea_orm))]
pub fn derive_select_builder(input: TokenStream) -> TokenStream {
    gen::expand(input, gen::Mode::Select)
}

/// Derive a `<Entity>Update` builder, enforcing at least one SET and WHERE.
#[proc_macro_derive(UpdateBuilder, attributes(sea_builder, sea_orm))]
pub fn derive_update_builder(input: TokenStream) -> TokenStream {
    gen::expand(input, gen::Mode::Update)
}

/// Derive a `<Entity>Delete` builder, enforcing at least one WHERE.
#[proc_macro_derive(DeleteBuilder, attributes(sea_builder, sea_orm))]
pub fn derive_delete_builder(input: TokenStream) -> TokenStream {
    gen::expand(input, gen::Mode::Delete)
}
