# sea_orm_builder

Ergonomic, safe SeaORM query builders generated via derive macros. Users depend only on `sea_orm_builder`; it re‑exports the procedural macros from `sea_orm_builder_derive` and provides small runtime helpers.

- Builders are backend‑agnostic
- Enforce safety (must have WHERE; updates require SET + WHERE)
- Accept `&str` or `String` for `String` fields
- Field‑level attributes on SeaORM `Model` drive which WHERE ops and SETs are allowed
- Convenient typed accessors for WHERE inputs, even after build

## Quick Usage

```rust
use sea_orm_builder::*;
use sea_orm::entity::prelude::*;

#[derive(DeriveEntityModel, SelectBuilder, UpdateBuilder, DeleteBuilder)]
#[sea_orm(table_name = "demo_item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[sea_builder(select(where(eq, in)), update(where(eq, in)), delete(where(eq, in)))]
    pub id: u64,

    #[sea_builder(select(where(eq, like)), update(where(eq), set))]
    pub name: String,
}

use demo_item::{Entity, Column};

let (_stmt, params) = DemoItemSelect::new()
    .name_like("foo")
    .id_eq(1u64)
    .build_with_params();
assert!(params.is_id_eq());
assert_eq!(params.get_name_like(), Some(&"foo".to_string()));
```

## Query Condition Actions

The derive macros recognize the following `where(...)` operations when parsing `#[sea_builder(...)]` attributes. Each action generates typed builder methods (for example, `id_eq`, `name_like`).

- `eq` / `ne` – equality and inequality (`Column::eq`, `Column::ne`)
- `lt` / `lte` – less-than and less-than-or-equal (`Column::lt`, `Column::lte`)
- `gt` / `gte` – greater-than and greater-than-or-equal (`Column::gt`, `Column::gte`)
- `like` / `ilike` – pattern matches; `ilike` is case-insensitive where supported (`Column::like`, `Column::ilike`)
- `in` / `isin` – membership check; both keywords map to the same generated `<field>_in` method (`Column::is_in`)
- `between` – inclusive range check that accepts two arguments and maps to `Column::between`

## Regeneration Prompt

Paste the following prompt into Codex CLI next time you want to (re)generate both crates. It restates the requirements and expected deliverables.

---
You are building two Rust crates:
- sea_orm_builder (runtime crate)
- sea_orm_builder_derive (proc‑macro crate)

Goal: Provide ergonomic, safe SeaORM query builders generated via derive macros, using field‑level attributes on SeaORM Model to specify which columns are allowed in select/update/delete WHERE clauses and which fields are set‑able in updates. Builders are backend‑agnostic, enforce safety, and accept &str for String fields automatically. Also provide a way to retrieve WHERE inputs after build.

Deliverables
1. Full code for both crates with Cargo.toml files, src/lib.rs for each, and re‑exports so users only depend on sea_orm_builder.
2. Field‑level attributes on SeaORM Model control generation:
   - #[sea_builder(select(where(eq, like)))]
   - #[sea_builder(update(where(eq, in), set))]
   - #[sea_builder(delete(where(gte, lt)))]
3. Three derives:
   - #[derive(SelectBuilder)] → generates <Entity>Select
   - #[derive(UpdateBuilder)] → generates <Entity>Update with build() requiring at least one SET and at least one WHERE
   - #[derive(DeleteBuilder)] → generates <Entity>Delete with build() requiring at least one WHERE
4. Builders assume `use my_entity::{Entity, Column};` is in scope at callsite.
5. Supported ops: eq, ne, lt, lte, gt, gte, like, ilike, in/isin, between. (Map to SeaORM’s ColumnTrait methods. between(a,b) takes two args. in and isin are synonyms; generate method <field>_in.)
6. String parameters accept &str or String via a generic adapter.
7. Re‑export derives from sea_orm_builder so users do: `use sea_orm_builder::{SelectBuilder, UpdateBuilder, DeleteBuilder};`
8. Add a minimal example and tests.
9. After calling build_with_params(), return a Params snapshot that exposes typed WHERE accessors:
   - is_<field>_<op>() -> bool
   - get_<field>_<op>() -> Option<&T>
   - for IN: get_<field>_in() -> Option<&[T]>
   - for BETWEEN: get_<field>_between() -> Option<(&T, &T)>
   Also return a Vec<WhereParam> for logging with enum WhereValue = Single(String) | List(Vec<String)) | Range { start, end }.
10. Keep builders backend‑agnostic. If an op (e.g., ilike) needs a backend trait, allow generation but tests/examples should avoid backend‑specific ops unless enabled.

Implementation notes
- Crate sea_orm_builder exposes:
  - trait IntoField<T> so &str -> String conversion works
  - error enum SeaOrmBuilderError { NoWhere, NoSet }
  - re‑exports module `gen` with SeaORM traits used in generated code
  - public types WhereParam and WhereValue
- Proc‑macro parses #[sea_orm(table_name = "...")] to name builders <TableNameCamelCase>Select/Update/Delete.
- Each builder tracks has_where and (for Update) set_count; enforce in build().
- WHERE methods set flags, push into where_params Vec<WhereParam>, and store typed copies in per‑field storage used by the Params snapshot.
- Provide build_with_params() that returns (statement, <Builder>Params) with typed accessors and where_params().
- Provide example and tests that compile in a generic workspace.

Please scan the workspace Cargo.toml to add both crates to [workspace]. Keep changes minimal and follow existing code style.
---

## License
Internal project – no license header added by default.
