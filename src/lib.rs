mod cell;
mod traits;

use cell::*;
pub use traits::*;

use serde::{Deserialize, Serialize};

/// a specialized packed quadtree, for use in another project.
/// D is the max recursion depth, allowing up to 2^D coordinates.
/// this data format makes insertion / deletion slower, but query is D at worst.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuadTree<T, const D: u32> {
    /// the root cell is at 0, with node cells referencing indices to the next cells.
    /// indices are absolute.
    cells: Vec<Cell<T>>,
}

impl<T, const D: u32> Default for QuadTree<T, D> {
    fn default() -> Self {
        Self {
            cells: vec![Cell::new_empty(usize::MAX)],
        }
    }
}

impl<T, const D: u32> QuadTree<T, D> {
    pub fn new() -> Self {
        Default::default()
    }

    /// adds a point into the tree with associated data.
    /// returns if the insertion succeeded, i.e. no clashes occured.
    pub fn insert<C: AsQTCoord>(&mut self, xy: C, data: T) -> bool {
        let (x, y) = xy.as_quadtree_coord();
        assert!(x.max(y) < 2u32.pow(D));
        self.insert_at(0, D, (x, y), data)
    }
    
    /// removes a point and returns the stored data.
    pub fn remove<C: AsQTCoord>(&mut self, xy: C) -> Option<T> {
        let (x, y) = xy.as_quadtree_coord();
        assert!(x.max(y) < 2u32.pow(D));
        let (d, i) = self.get_in_cell(D, 0, x, y)?;
        let res = self.cells[i].data.take().unwrap();
        
        self.prune_cell(i, d, x, y);
        
        let CellData::Leaf(res) = res else { unreachable!() };
        Some(res.data)
    }
    
    pub fn get<C: AsQTCoord>(&self, xy: C) -> Option<&T> {
        let (x, y) = xy.as_quadtree_coord();
        assert!(x.max(y) < 2u32.pow(D));
        self.get_in_cell(D, 0, x, y).map(move |(_, i)| {
            let c = &self.cells[i];
            let Some(CellData::Leaf(pl)) = &c.data else { unreachable!() };
            &pl.data
        })
    }
    
    pub fn get_mut<C: AsQTCoord>(&mut self, xy: C) -> Option<&mut T> {
        let (x, y) = xy.as_quadtree_coord();
        assert!(x.max(y) < 2u32.pow(D));
        self.get_in_cell(D, 0, x, y).map(move |(_, i)| {
            let c = &mut self.cells[i];
            let Some(CellData::Leaf(pl)) = &mut c.data else { unreachable!() };
            &mut pl.data
        })
    }
    
    /// w and h must be positive non-zero!
    /// returns all items within the area `(x, y) ~ (x+w-1, y+h-1)` inclusive.
    pub fn get_in_region<C: AsQTCoord>(&self, xy: C, w: u32, h: u32) -> Vec<&T> {
        let (x0, y0) = xy.as_quadtree_coord();
        let x1 = x0 + w - 1;
        let y1 = y0 + h - 1;
        assert!(x0.max(y0).max(x1).max(y1) < 2u32.pow(D));
        let mut res = vec![];
        self.push_if_in_region(D, 0, x0, y0, x1, y1, &mut res);
        res
    }
    
    /// iterate over all stored items
    pub fn iter(&self) -> impl Iterator<Item=((u32, u32), &T)> {
        self.cells.iter().filter_map(|c| {
            if let Some(CellData::Leaf(pl)) = &c.data {
                Some(((pl.x, pl.y), &pl.data))
            }
            else {
                None
            }
        })
    }
    /// iterate over all stored items
    pub fn iter_mut(&mut self) -> impl Iterator<Item=((u32, u32), &mut T)> {
        self.cells.iter_mut().filter_map(|c| {
            if let Some(CellData::Leaf(pl)) = &mut c.data {
                Some(((pl.x, pl.y), &mut pl.data))
            }
            else {
                None
            }
        })
    }
    
    // ---------------- internal functions ----------------------
    
    fn insert_at(&mut self, at: usize, depth: u32, (x, y): (u32, u32), t: T) -> bool {
        //println!("try {at} @ depth {depth}");
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
                if depth == 0 {
                    //same cell at lowest depth, so its the same coord.
                    //fail if occupied.
                    //println!("reached depth 0!");
                    return false;
                }
                let cells = match d {
                    CellData::Leaf(_) => { //alloc a new node, place the old in
                        let plc = data.take().unwrap();
                        let CellData::Leaf(pl) = &plc else { unreachable!() };
                        let new_cells = self.extend_node(at);
                        self.cells[at].data = Some(CellData::Node(new_cells));
                        let cell_i = Self::coord2cell_i(pl.x, pl.y, depth);
                        let nd = &mut self.cells[new_cells[cell_i]];
                        nd.data = Some(plc);
                        //println!("  move {cell_i}");
                        //self.pretty_print();
                        new_cells
                    }
                    CellData::Node(cells) => *cells
                };
                //find the cell for the new data, and recurse
                let cell_i = Self::coord2cell_i(x, y, depth);
                //println!("  cell {cell_i}");
                self.insert_at(cells[cell_i], depth - 1, (x, y), t)
            }
        }
    }
    
    fn extend_node(&mut self, par_index: usize) -> [usize; 4] {
        let n = self.cells.len();
        self.cells.resize_with(n+4, || Cell::new_empty(par_index));
        [n, n+1, n+2, n+3]
    }
    
    fn coord2cell_i(x: u32, y: u32, depth: u32) -> usize {
        let d = depth - 1;
        let dx = (x >> d) & 1;
        let dy = (y >> d) & 1;
        let i = (dx + dy * 2) as usize;
        i
    }
    
    fn get_in_cell(&self, depth: u32, i: usize, x: u32, y: u32) -> Option<(u32, usize)> {
        let c = &self.cells[i];
        let c = &c.data.as_ref()?;
        match c {
            CellData::Leaf(pl) => (pl.x == x && pl.y == y).then(|| (depth, i)),
            CellData::Node(jj) => {
                let cell_i = Self::coord2cell_i(x, y, depth);
                self.get_in_cell(depth - 1, jj[cell_i], x, y)
            }
        }
    }
    
    //TODO: make this faster, now it accesses all children nodes to check if empty
    fn prune_cell(&mut self, i: usize, depth: u32, x: u32, y: u32) {
        let cell = &self.cells[i];
        //if this cell is empty, or all children are empty, delete and recurse up
        let par = cell.parent;
        if par == usize::MAX {
            //we are at root, do not remove
            return
        }
        if let Some(CellData::Node(jj)) = &cell.data {
           let mut jj = *jj;
           if jj.iter().any(|j| self.cells[*j].data.is_some()) {
                //theres a valid child, do not delete
                return
           }
           //clear all the children
           jj.sort_by(|a, b| b.cmp(a)); //make sure we dont invalidate other indices while deleting
           for j in jj {
               self.swap_rem(j, depth, x, y);
           }
        }
        //set to null and recurse
        self.cells[i].data = None;
        self.prune_cell(par, depth + 1, x, y);
    }
    
    fn swap_rem(&mut self, i: usize, depth: u32, x: u32, y: u32) {
        self.cells.swap_remove(i);
        if let Some(c) = self.cells.get(i) {
            let p = c.parent;
            let par = &mut self.cells[p];
            let Some(CellData::Node(jj)) = &mut par.data else { unreachable!() };
            let cell_i = Self::coord2cell_i(x, y, depth + 1);
            jj[cell_i] = i;
        }
    }
    
    fn push_if_in_region<'a, 'b>(&'a self, depth: u32, i: usize, x0: u32, y0: u32, x1: u32, y1: u32, res: &mut Vec<&'b T>)
    where
        'a: 'b
    {
        let cell = &self.cells[i];
        let Some(data) = &cell.data else { return };
        match data {
            CellData::Leaf(pl) => {
                if (x0..=x1).contains(&pl.x) && (y0..=y1).contains(&pl.y) {
                    res.push(&pl.data);
                }
            },
            CellData::Node(jj) => {
                let ja = Self::coord2cell_i(x0, y0, depth);
                let jb = Self::coord2cell_i(x1, y1, depth);
                let d2 = depth - 1;
                self.push_if_in_region(d2, jj[ja], x0, y0, x1, y1, res);
                if jb != ja {
                    //we know jb cannot be smaller than ja
                    //since the data is ordered so (x right, y up):
                    // 2 3
                    // 0 1
                    //there are only a few possibilities:
                    // 0~2; 0~3, 1~3
                    //so we only have to consider the case of 0~3
                    if ja == 0 && jb == 3 {
                        self.push_if_in_region(d2, jj[1], x0, y0, x1, y1, res);
                        self.push_if_in_region(d2, jj[2], x0, y0, x1, y1, res);
                    }
                    self.push_if_in_region(d2, jj[jb], x0, y0, x1, y1, res);
                }
            }
        }
    }
}

impl<T: std::fmt::Debug, const D: u32> QuadTree<T, D> {
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
}