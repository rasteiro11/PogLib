use diesel::prelude::*;

pub fn get_connection(url: &String) -> MysqlConnection {
    MysqlConnection::establish(&url).unwrap_or_else(|_| panic!("Error connecting to {}", url))
}
