use crate::schema::books;
use diesel::prelude::*;

#[derive(
    PartialEq,
    Eq,
    Debug,
    Clone,
    Queryable,
    Identifiable,
    Insertable,
    AsChangeset,
    QueryableByName,
    Selectable,
)]
#[diesel(table_name = books)]
pub struct Book {
    pub id: i32,
    pub name: String,
}
