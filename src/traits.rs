
pub struct QTCoord(pub (u32, u32));

pub trait AsQTCoord {
    fn as_quadtree_coord(self) -> (u32, u32);
}

impl AsQTCoord for (u32, u32) {
    fn as_quadtree_coord(self) -> (u32, u32) {
        self
    }
}