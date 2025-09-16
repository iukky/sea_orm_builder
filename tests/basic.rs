mod my_entity {
    use sea_orm::entity::prelude::*;
    use sea_orm_builder::*;

    #[derive(
        Clone, Debug, PartialEq, DeriveEntityModel, SelectBuilder, UpdateBuilder, DeleteBuilder,
    )]
    #[sea_orm(table_name = "foo_bar")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        #[sea_builder(
            select(where(eq, in, not_in)),
            update(where(eq, in, not_in)),
            delete(where(eq, in, not_in))
        )]
        pub id: u64,

        #[sea_builder(select(where(eq, like)), update(where(eq), set))]
        pub name: String,

        #[sea_builder(delete(where(gte, lt)), update(where(between), set))]
        pub age: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

use my_entity::Column;

#[test]
fn builders_compile_and_enforce() {
    use sea_orm_builder::*;
    // select
    let (_sel_stmt, params) = my_entity::FooBarSelect::new()
        .name_like("abc")
        .id_not_in([2u64, 3u64])
        .id_eq(1u64)
        .order_by_desc(Column::Name)
        .limit(10)
        .offset(5)
        .build_with_params();
    assert!(params.is_id_eq());
    assert_eq!(params.get_id_eq(), Some(&1u64));
    assert!(params.is_id_not_in());
    assert_eq!(params.get_id_not_in().unwrap(), &[2u64, 3u64]);
    assert!(params.is_name_like());
    assert_eq!(params.get_name_like().unwrap(), "abc");
    assert_eq!(params.where_params().len(), 3);

    // update ok
    let up_ok = my_entity::FooBarUpdate::new()
        .set_name("new")
        .age_between(1, 99)
        .id_not_in([1u64, 2u64])
        .id_eq(42)
        .build();
    assert!(up_ok.is_ok());

    // update with params snapshot
    let up_ok_with_params = my_entity::FooBarUpdate::new()
        .set_name("new")
        .age_between(1, 99)
        .id_not_in([1u64, 2u64])
        .id_eq(42)
        .build_with_params();
    assert!(up_ok_with_params.is_ok());
    let (_u_stmt, u_params) = up_ok_with_params.unwrap();
    assert!(u_params.is_id_eq());
    assert_eq!(u_params.get_id_eq(), Some(&42));
    assert!(u_params.is_id_not_in());
    assert_eq!(u_params.get_id_not_in().unwrap(), &[1u64, 2u64]);
    assert!(u_params.is_age_between());
    let (a, b) = u_params.get_age_between().unwrap();
    assert_eq!((*a, *b), (1, 99));
    assert_eq!(u_params.where_params().len(), 3);

    // update missing where
    let up_no_where = my_entity::FooBarUpdate::new().set_name("x").build();
    assert!(matches!(up_no_where, Err(SeaOrmBuilderError::NoWhere)));

    // delete ok
    let del_ok = my_entity::FooBarDelete::new()
        .id_not_in([1u64, 2u64])
        .id_eq(1u64)
        .build_with_params();
    assert!(del_ok.is_ok());
    let (_, del_params) = del_ok.unwrap();
    assert!(del_params.is_id_eq());
    assert!(del_params.is_id_not_in());
    assert_eq!(del_params.get_id_not_in().unwrap(), &[1u64, 2u64]);
    assert_eq!(del_params.where_params().len(), 2);

    // delete missing where
    let del_err = my_entity::FooBarDelete::new().build();
    assert!(matches!(del_err, Err(SeaOrmBuilderError::NoWhere)));
}
