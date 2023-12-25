use anyhow::Result;
use rusqlite::{params, Connection, Params, Statement};
use serde::Deserialize;
use std::marker::PhantomData;

pub trait AsRepo<'a>
where
    for<'de> Self: Deserialize<'de> + Clone,
{
    fn repo(conn: &'a Connection) -> Repo<'a, Self>;
}

#[derive(Debug)]
pub struct Repo<'a, T: for<'de> Deserialize<'de> + Clone> {
    pub table: &'static str,
    pub conn: &'a Connection,
    _type: PhantomData<T>,
}

impl<'a, T: for<'de> Deserialize<'de> + Clone> Repo<'a, T> {
    pub fn new(conn: &'a Connection, table: &'static str) -> Self {
        Self {
            table,
            conn,
            _type: PhantomData,
        }
    }

    pub fn query<P: Params>(&self, stmt: &mut Statement<'_>, params: P) -> Result<Vec<T>> {
        let items = stmt
            .query_and_then(params, |row| serde_rusqlite::from_row::<T>(row))?
            .map(|r| r.map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn select_all(&self) -> Result<Vec<T>> {
        let mut stmt = self
            .conn
            .prepare(format!("SELECT * FROM {}", self.table).as_str())?;
        self.query(&mut stmt, [])
    }

    pub fn select(&self, id: i32) -> Result<Option<T>> {
        let mut stmt = self
            .conn
            .prepare(format!("SELECT * FROM {} WHERE id=?1", self.table).as_str())?;
        let items = self.query(&mut stmt, params![id])?;
        Ok(items.get(0).cloned())
    }
}
