use sled::Db;
use std::path::Path;
use crate::Error;

pub struct Database(sled::Db);

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Database(Db::open(path)?))
    }
}
