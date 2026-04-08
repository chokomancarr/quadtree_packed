use quadtree::*;

#[test]
fn test_packed() {
    let mut tree: QuadTree<f32, 5> = QuadTree::new();
    
    println!(" ---- insert");
    assert!(tree.insert((3, 5), 1.0));
    println!(" ---- insert");
    assert!(tree.insert((31, 24), 2.0));
    println!(" ---- insert");
    assert!(tree.insert((4, 19), 3.0));
    println!(" ---- insert");
    assert!(tree.insert((28, 20), 3.0));
    println!(" ---- insert");
    assert!(tree.insert((2, 5), 3.0));
    
    tree.pretty_print();
}