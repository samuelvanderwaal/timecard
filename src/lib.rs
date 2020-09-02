use fake::{Dummy, Fake};
use serde::{Deserialize, Serialize};

pub mod api;
pub mod db;

#[derive(Debug, Dummy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub id: Option<i32>,
    pub start: String,
    pub stop: String,
    pub week_day: String,
    pub code: String,
    pub memo: String,
}

#[derive(Debug, Dummy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<i32>,
    pub name: String,
    pub code: String,
}
