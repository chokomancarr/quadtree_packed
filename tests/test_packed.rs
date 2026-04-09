use quadtree::*;
use std::collections::HashSet;

#[test]
fn test_packed() {
    let mut tree: QuadTree<u8, 5> = QuadTree::new();
    
    assert!(tree.insert((3, 5), 1));
    assert!(tree.insert((31, 24), 2));
    assert!(tree.insert((4, 19), 3));
    assert!(tree.insert((28, 20), 4));
    assert!(tree.insert((2, 5), 5));
    
    tree.pretty_print();
    
    assert_eq!(tree.get((2, 2)), None);
    *tree.get_mut((4, 19)).unwrap() = 9;
    
    assert_eq!(tree.remove((3, 5)), Some(1));
    assert_eq!(tree.remove((2, 5)), Some(5));
    
    assert!(tree.insert((10, 2), 8));
    assert!(tree.insert((1, 1), 0));
    assert!(tree.insert((0, 1), 1));
    
    tree.pretty_print();
    
    let items = tree.iter().map(|(_, i)| *i).collect::<HashSet<_>>();
    assert_eq!(items, [0, 1, 2,4,8,9].into());
    
    let items = tree.get_in_region((8, 0), 21, 30);
    assert_eq!(Into::<HashSet<_>>::into([4, 8]), items.into_iter().copied().collect());
}