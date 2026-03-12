use once_cell::sync::OnceCell;
use redis::{Client, Connection};

use crate::AppResult;

pub static CLIENT: OnceCell<Client> = OnceCell::new();

pub fn try_init() -> AppResult<()> {
    let client = redis::Client::open(crate::redis_url())?;
    let _ = client.get_connection()?;
    CLIENT.set(client).unwrap();
    Ok(())
}
pub fn connect() -> AppResult<Connection> {
    Ok(CLIENT.get().unwrap().get_connection()?)
}
