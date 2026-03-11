// use std::future::Future;

use once_cell::sync::OnceCell;
use redis::{Client, Commands, Connection};

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

// pub fn lock(lock_key: impl Into<String>) -> AppResult<impl FnOnce() -> AppResult<()>> {
//     let mut rc = connect()?;
//     let lock_key = lock_key.into();
//     if rc.set_nx::<_, _, ()>(&lock_key, "1").is_err() {
//         return Err(crate::Error::Internal("lock key exists already".into()));
//     }
//     drop(rc);

//     Ok(move || {
//         let mut rc = crate::redis::connect()?;
//         if rc.del::<_, ()>(lock_key).is_err() {
//             return Err(crate::Error::Internal("remove lock key failed".into()));
//         }
//         Ok(())
//     })
// }

pub struct Locker {
    lock_key: String,
    locking: bool,
}

impl Locker {
    pub fn lock(lock_key: impl Into<String>) -> AppResult<Self> {
        let mut rc = connect()?;
        let lock_key = lock_key.into();
        if rc.set_nx::<_, _, ()>(&lock_key, "1").is_err() {
            return Err(crate::Error::Internal("lock key exists already".into()));
        }
        drop(rc);
        Ok(Self {
            lock_key,
            locking: true,
        })
    }
    pub fn unlock(&mut self) -> AppResult<()> {
        if self.locking {
            let mut rc = crate::redis::connect()?;
            if rc.del::<_, ()>(&self.lock_key).is_err() {
                return Err(crate::Error::Internal("remove lock key failed".into()));
            }
            self.locking = false;
        }
        Ok(())
    }
}

impl Drop for Locker {
    fn drop(&mut self) {
        self.unlock().ok();
    }
}
