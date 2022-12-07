use crate::schema::books;
use diesel::prelude::*;

#[derive(Queryable, Debug)]
pub struct Book {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = books)]
pub struct NewBook<'a> {
    pub name: &'a str,
}
