/// Generic converter so String fields accept both &str and String; identity for others.
pub trait IntoField<T> {
    fn into_field(self) -> T;
}
impl<'a> IntoField<String> for &'a str {
    #[inline]
    fn into_field(self) -> String {
        self.to_owned()
    }
}
impl<T> IntoField<T> for T {
    #[inline]
    fn into_field(self) -> T {
        self
    }
}

/// Re-exports used by generated code
pub mod gen {
    pub use crate::IntoField;
    pub use sea_orm::{
        sea_query::{Expr, ValueType},
        ColumnTrait, DeleteMany, EntityTrait, Order, QueryFilter, QueryOrder, QuerySelect, Select,
        UpdateMany,
    };
}

// Simple error type used by generated builders
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum SeaOrmBuilderError {
    #[error("no WHERE added")]
    NoWhere,
    #[error("no SET added")]
    NoSet,
}

// Re-export the derive macros so users only depend on sea_orm_builder
pub use sea_orm_builder_derive::{DeleteBuilder, SelectBuilder, UpdateBuilder};

// Metadata captured for where clauses
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereParam {
    pub field: &'static str,
    pub op: &'static str,
    pub value: WhereValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhereValue {
    Single(String),
    List(Vec<String>),
    Range { start: String, end: String },
}
