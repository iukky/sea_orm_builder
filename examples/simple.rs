use sea_orm::entity::prelude::*;
use sea_orm_builder::*;

mod my_entity {
    use super::*;

    #[derive(
        Clone, Debug, PartialEq, DeriveEntityModel, SelectBuilder, UpdateBuilder, DeleteBuilder,
    )]
    #[sea_orm(table_name = "demo_item")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        #[sea_builder(select(where(eq, in)), update(where(eq, in)), delete(where(eq, in)))]
        pub id: u64,

        #[sea_builder(select(where(eq, like)), update(where(eq), set))]
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

use my_entity::{Column, Entity};

fn main() {
    // Build a safe select and inspect params
    let (_q, params) = my_entity::DemoItemSelect::new()
        .name_like("foo")
        .id_eq(1u64)
        .build_with_params();
    println!(
        "id set? {} val: {:?}",
        params.is_id_eq(),
        params.get_id_eq()
    );
    println!(
        "name like? {} val: {:?}",
        params.is_name_like(),
        params.get_name_like()
    );

    // Safe update
    let _u = my_entity::DemoItemUpdate::new()
        .set_name("bar")
        .id_eq(1u64)
        .build();
    // Safe delete with params
    let _d = my_entity::DemoItemDelete::new()
        .id_eq(1u64)
        .build_with_params();
}
