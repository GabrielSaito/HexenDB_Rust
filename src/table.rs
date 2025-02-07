use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Column {
    pub name: String,
    pub is_primary_key: bool,
    pub foreign_key: Option<(String, String)>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Table {
    pub columns: Vec<Column>,
    pub data: Vec<Vec<String>>,
}
