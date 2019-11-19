use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeId(String);

#[derive(Debug, Serialize, Deserialize)]
pub struct Date(String);

#[derive(Debug, Serialize, Deserialize)]
pub struct Oid(String);
