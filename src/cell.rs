

#[derive(Debug)]
pub struct Cell<T> {
    //pub mask_x: u32,
    //pub mask_y: u32,
    pub parent: usize,
    pub data: Option<CellData<T>>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Payload<T> {
    pub x: u32,
    pub y: u32,
    pub data: T
}