use postgres::{Client, NoTls};

use crate::util::variable::{PG_DBNAME, PG_HOST, PG_PASSWORD, PG_USER};

pub mod user;

fn generate_client() -> Client {
    let url = format!(
        "host={} port={} user={} password={} dbname={}",
        *PG_HOST, 5432, *PG_USER, *PG_PASSWORD, *PG_DBNAME,
    );

    match Client::connect(&url, NoTls) {
        Ok(client) => client,
        Err(err) => panic!("{}", err.to_string()),
    }
}
