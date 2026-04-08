mod cell;
use cell::*;

/// a specialized packed quadtree, for use in another project.
/// D is the max recursion depth, allowing up to 2^D coordinates.
#[derive(Debug)]
pub struct QuadTree<T, const D: u32>
where
    T: std::fmt::Debug
{
    /// the root cell is at 0, with node cells referencing indices to the next cells.
    /// indices are absolute.
    cells: Vec<Cell<T>>,
}

impl<T: std::fmt::Debug, const D: u32> Default for QuadTree<T, D> {
    fn default() -> Self {
        Self {
            cells: vec![Cell::new_empty(usize::MAX, 0)],
        }
    }
}

impl<T: std::fmt::Debug, const D: u32> QuadTree<T, D> {
    pub fn new() -> Self {
        Default::default()
    }

    /// adds a point into the tree with associated data.
    /// returns if the insertion succeeded, i.e. no clashes occured.
    pub fn insert(&mut self, (x, y): (u32, u32), data: T) -> bool {
        assert!(x.max(y) < 2u32.pow(D));
        self.insert_at(0, D, (x, y), data)
    }
    
    /// removes a point and returns the stored data.
    pub fn remove(&mut self, (x, y): (u32, u32)) -> Option<T> {
        assert!(x.max(y) < 2u32.pow(D));
        todo!()
    }
    
    fn insert_at(&mut self, at: usize, next_depth: u32, (x, y): (u32, u32), t: T) -> bool {
        println!("try {at} @ depth {next_depth}");
        let cell = &mut self.cells[at];
        let data = &mut cell.data;
        match data {
            None => {//if the cell is empty, put directly
                *data = Some(CellData::Leaf(Payload {
                    x, y, data: t
                }));
                return true;
            }
            Some(d) => {
                if next_depth == 0 {
                    //same cell at lowest depth, so its the same coord.
                    //fail if occupied.
                    println!("reached depth 0!");
                    return false;
                }
                let cells = match d {
                    CellData::Leaf(t) => { //alloc a new node, place the old in
                        let plc = std::mem::replace(data, None).unwrap();
                        let CellData::Leaf(pl) = &plc else { unreachable!() };
                        let new_cells = self.extend_node(at);
                        self.cells[at].data = Some(CellData::Node(new_cells));
                        let cell_i = Self::coord2cell_i(pl.x, pl.y, next_depth);
                        let nd = &mut self.cells[new_cells[cell_i]];
                        nd.data = Some(plc);
                        println!("  move {cell_i}");
                        self.pretty_print();
                        new_cells
                    }
                    CellData::Node(cells) => *cells
                };
                //find the cell for the new data, and recurse
                let cell_i = Self::coord2cell_i(x, y, next_depth);
                println!("  cell {cell_i}");
                self.insert_at(cells[cell_i], next_depth - 1, (x, y), t)
            }
        }
    }
    
    pub fn pretty_print(&self) {
        let x0 = 0;
        let y0 = 0;
        self.pretty_print_one(x0, y0, D, 0);
    }
    
    fn pretty_print_one(&self, x0: u32, y0: u32, depth: u32, i: usize) {
        let w = 2u32.pow(depth);
        if w > 1 {
            let w2 = w / 2;
            let w = w - 1;
            let x1 = x0 + w2;
            let y1 = y0 + w2;
            let x1b = x1 - 1;
            let y1b = y1 - 1;
            let cell = &self.cells[i];
            if let Some(d) = &cell.data {
                let (s, inds) = match d {
                    CellData::Leaf(pl) => (format!(" {pl:?}"), None),
                    CellData::Node(inds) => ("".to_owned(), Some(inds))
                };
                println!("{:indent$}({x0:0>5}~{:0>5}, {y0:0>5}~{:0>5}) => {s}", "", x0+w, y0+w, indent=((D - depth) * 2) as usize);
                if let Some(inds) = inds {
                    let dd = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];
                    for (ind, (x, y)) in inds.iter().zip(dd) {
                        self.pretty_print_one(x, y, depth - 1, *ind);
                    }
                }
            }
        }
        else {
            let cell = &self.cells[i];
            if let Some(d) = &cell.data {
                let CellData::Leaf(pl) = d else { unreachable!() };
                println!("{:indent$}({x0:0>5}~{x0:0>5}, {y0:0>5}~{y0:0>5}) => {pl:?}", "", indent=(D * 2) as usize)
            }
        }
    }
    
    fn extend_node(&mut self, par_index: usize) -> [usize; 4] {
        let n = self.cells.len();
        self.cells.extend((0..4).map(|i| Cell::new_empty(par_index, i)));
        [n, n+1, n+2, n+3]
    }
    
    fn coord2cell_i(x: u32, y: u32, depth: u32) -> usize {
        let d = depth - 1;
        let dx = (x >> d) & 1;
        let dy = (y >> d) & 1;
        let i = (dx + dy * 2) as usize;
        i
    }
}