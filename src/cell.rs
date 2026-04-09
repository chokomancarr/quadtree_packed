
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell<T> {
    pub parent: usize,
    pub data: Option<CellData<T>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CellData<T> {
    Leaf(Payload<T>),
    Node([usize; 4])
}

impl<T> Cell<T> {
    pub fn new_empty(parent: usize) -> Self {
        Self {
            parent,
            data: None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Payload<T> {
    pub x: u32,
    pub y: u32,
    pub data: T
}