use std::fmt::Debug;
// https://en.wikipedia.org/wiki/B-tree
// A B-tree with order m will have a max of m children and thus a max of m-1 keys.

// If K = m-1 is the max num of search keys in a node, we have the following:
// A root node when it is a leaf node: min 0 max K keys, min 0 max 0 children
// A root node when it is an internal node: min 1 max K keys, min 2 max K+1 children
// An internal node: min floor(K/2) max K keys, min ceiling((K+1)/2)=floor(K/2)+1 max K+1 children
// A leaf node: min ceiling(K/2) max K keys, min 0 max 0 children

// NOTE: There are two main definitions of B-Trees (Knuth and CLRS): https://stackoverflow.com/questions/28846377/what-is-the-difference-btw-order-and-degree-in-terms-of-tree-data-structure
// This uses the Knuth defintion, which allows for the special case of 2-3 trees

pub struct BTree<T: PartialOrd + Debug + Clone> {
    root: Option<Box<Node<T>>>,
    order: usize, 
}

// The number of child nodes will be 1 more than the number of keys -> ceiling(m/2) = floor(m/2) + 1
struct Node<T: PartialOrd + Debug + Clone> {
    keys: Vec<T>,
    children: Vec<Box<Node<T>>>,
    leaf: bool,
    order: usize,
}

impl<T: PartialOrd + Debug + Clone> BTree<T> {
    /// Constructor method for BTree
    /// 
    /// Takes in a usize parameter m representing the knuth order of a BTree
    /// 
    /// Note: Order 3 is a special case (2-3 tree) where nodes can have 1-2 keys normally                                                                     
    /// but temporarily hold 3 keys before splitting (since 2 keys cannot be split evenly) 
    pub fn new(m: usize) -> Self {
        assert!(m >= 3, "BTree order must be at least 3");
        BTree{ root: None, order: m }
    }

    /// Traverse method for BTree
    /// 
    /// Traverses through all keys for all nodes in order and prints them out
    pub fn traverse(&self) {
        match &self.root {
            Some(r) => r.traverse(),
            None => println!("=== EMPTY BTREE ==="),
        }
    }

    /// Search method for BTree
    /// 
    /// Returns true if value is present, false otherwise
    pub fn search(&self, value: T) -> bool {
        // Set node as root (as_ref() is explicitly used but not necessary to make node the Node<T> type instead of Box<Node<T>>)
        // due to automatic dereferencing
        let mut node = match &self.root {
            Some(r) => r.as_ref(),
            None => return false,
        };

        // Call search iteratively on each node
        loop {
            let (found, idx) = node.search(&value);
            // If found, return true
            if found {
                return true;
            }
            // If not found, check if leaf node (return false) or set node as child node
            if node.leaf {
                return false;
            }
            // Could use node.children[idx].as_ref() but automatic dereferncing allows use of &    
            node = &node.children[idx];
        }
    }

    /// Inserts a value into the b-tree
    pub fn insert(&mut self, value: T) {
        // Check if root is empty
        match &mut self.root {
            Some(r) => {
                // For order 3, allow root to have 3 keys temporarily before splitting
                let max_keys_before_split = if self.order == 3 { 3 } else { self.order - 1 };

                if r.keys.len() < max_keys_before_split {
                    // If root is not full, insert into root node recursively
                    r.insert_non_full(value);

                    // Check if root now needs splitting (for order 3 with 3 keys)
                    if let Some(root) = &mut self.root {
                        if self.order == 3 && root.keys.len() == 3 {
                            let old_root = self.root.take().expect("Root must exist");
                            let mut new_root: Node<T> = Node{ keys: vec![], children: vec![old_root], leaf: false, order: self.order };
                            new_root.split_child(0);
                            self.root = Some(Box::new(new_root));
                        }
                    }
                } else {
                    // Else (root is full), make a new root, make old root a child of new root, split the old root, and insert into new root recursively
                    let old_root = self.root.take().expect("Root must exist in Some branch");
                    let mut new_root: Node<T> = Node{ keys: vec![], children: vec![old_root], leaf: false, order: self.order };
                    new_root.split_child(0);
                    new_root.insert_non_full(value);
                    self.root = Some(Box::new(new_root));
                }
            },
            None => {
                // If root is empty, create a new root leaf node and insert value
                let new_node: Node<T> = Node{ keys: vec![value], children: vec![], leaf: true, order: self.order };
                self.root = Some(Box::new(new_node));
            },
        }
    }

    /// Deletes a value from the b-tree
    pub fn delete(&mut self, value: T) {
        // Check if root is empty
        let node = match &mut self.root {
            Some(r) => r.as_mut(),
            None => panic!("Cannot delete from empty BTree"),
        };

        // If not empty, call delete on root
        node.delete(&value);
        
        // Shrink tree if root is empty but has children
        // Some(root) is part of if let pattern matching that executes the block if self.root is Some
        if let Some(root) = &mut self.root {
            if root.keys.is_empty() && !root.children.is_empty() {
                self.root = Some(self.root.take().unwrap().children.remove(0));
            }
        }
    }

    /// Helper (test) function for printing b-tree structure
    #[cfg(test)]
    pub fn print_structure(&self) {
        match &self.root {
            Some(r) => {
                println!("=== BTree Structure (Order {}) ===", self.order);
                println!();
                r.print_structure(0);
            },
            None => println!("Empty tree"),
        }
    }
}


impl<T: PartialOrd + Debug + Clone> Node<T> {
    /// Traverses and prints out all of the keys recursively
    fn traverse(&self) {
        // Loop through first n children and keys
        for i in 0..self.keys.len() {
            // If not a leaf, traverse child
            if !self.leaf {
                self.children[i].traverse();
            }
            // Print out key at idx i
            print!("{:?} ", self.keys[i]);
        }
        // Traverse last child if not leaf
        if !self.leaf {
            self.children[self.keys.len()].traverse();
        }
    }

    /// Searches for value in keys (currently wraps binary search helper)
    /// 
    /// Returns true if value in keys and idx in keys
    /// 
    /// Returns false if value not in keys and idx of smallest key greater than search value
    fn search(&self, value: &T) -> (bool, usize) {
        let (found, idx) = self.binary_search(value);
        (found, idx)
    }

    /// Binary search helper for B-tree node
    fn binary_search(&self, value: &T) -> (bool, usize) {
        let mut left = 0;
        let mut right = self.keys.len();

        // Range is [left, right) - left inclusive, right exclusive 
        while left < right {
            let mid = left + (right - left) / 2;

            if self.keys[mid] == *value {
                return (true, mid);
            }
            
            if self.keys[mid] < *value {
                left = mid + 1; // Search right half (exclusive of mid)
            } else { // self.keys[mid] > value
                right = mid; // Search left half (inclusive of mid -> mid could be idx of smallest value greater than value)
            }
        }

        // If the value is not found, return false and idx of smallest value greater than value (insertion point)
        (false, left)
    }

    /// Inserts a value as a new key into a leaf node (called recursively)
    ///
    /// It assumes that the node must be non-full when the function is called
    fn insert_non_full(&mut self, value: T) {
        // Find index of where value should be placed
        let (_, mut idx) = self.search(&value);

        if self.leaf {
            // If the node is a leaf node, insert value into key (base case)
            self.keys.insert(idx, value);
        } else {
            // Else, recursively call insert_non_full on child the value should go into
            // For order 3, allow child to have 3 keys temporarily before splitting
            if self.children[idx].order == 3 && self.children[idx].keys.len() == 2 {
                // Insert into child first
                self.children[idx].insert_non_full(value);

                // Now check if child has 3 keys and needs splitting
                if self.children[idx].keys.len() == 3 {
                    self.split_child(idx);
                }
            } else if self.children[idx].keys.len() == (self.children[idx].order - 1) {
                // Standard split for full children (non-order-3 case)
                self.split_child(idx);
                // Choose left or right child depending on new middle key (from child) at idx
                if value > self.keys[idx] {
                    idx += 1;
                }
                // Insert into non-full child
                self.children[idx].insert_non_full(value);
            } else {
                // Child not full, just insert
                self.children[idx].insert_non_full(value);
            }
        }
    }

    /// Splits a full child node into 2 nodes and moves the middle key up into current node
    /// 
    /// Takes a child_idx that represents the index of the child to be split
    fn split_child(&mut self, child_idx: usize) {
        // Get child node and calculate midpoint with integer division (in even cases, midpoint is skewed to right)
        let child = &mut self.children[child_idx];
        let mid = child.keys.len() / 2;

        // Split keys: right half starts at mid+1
        let right_keys: Vec<T> = child.keys.split_off(mid + 1);

        // Remove middle key 
        let middle_key = child.keys.pop().expect("Middle key missing in split_child");

        // Split children
        let right_children: Vec<Box<Node<T>>> = if child.leaf {
            vec![]
        } else {
            child.children.split_off(mid + 1)
        };

        // Create new child node to copy second half of keys from child node into
        // Now split the remaining keys and children
        // After remove(mid), what was at mid+1 is now at mid
        let new_node: Node<T> = Node{ keys: right_keys, children: right_children, leaf: child.leaf, order: child.order };
        
        // Move and insert middle key into current node
        self.keys.insert(child_idx, middle_key);

        // Insert new child node to right of old child node (old child node borrowing is done)
        self.children.insert(child_idx + 1, Box::new(new_node));
    }

    /// Deletes a value from the node (recursively) with several different cases 
    fn delete(&mut self, value: &T) {
        // Find index of smallest key greater than value == index of child value belongs in
        let (found, idx) = self.search(value);
        if found {
            if self.leaf {
                // Case 1: The value is in a leaf node (assumes has enough keys)
                self.keys.remove(idx);
                return;
            } else {
                // Case 2: The value is in an internal node
                if self.children[idx].keys.len() >= (self.order - 1) / 2 + 1 {
                    // Case 2a: Left subtree has at least floor(K/2) + 1 keys if case 3 (merging => lose 1 key) is called on it
                    // Get predecessor
                    let pred = self.children[idx].get_rightmost().clone();
                    // Delete predecessor
                    self.children[idx].delete(&pred);
                    // Replace current value with predecessor
                    self.keys[idx] = pred;

                } else if self.children[idx + 1].keys.len() >= (self.order - 1) / 2 + 1 {
                    // Case 2b: Right subtree has at least floor(K/2) + 1 keys if case 3 (merging => lose 1 key) is called on it
                    // Get successor
                    let succ = self.children[idx + 1].get_leftmost().clone();
                    // Delete successor
                    self.children[idx + 1].delete(&succ);
                    // Replace current value with successor
                    self.keys[idx] = succ;

                } else {
                    // Case 2c: Both left and right do not have enough keys, so we merge them
                    self.merge(idx);
                    self.children[idx].delete(value);
                }
            }
        } else {
            if !self.leaf {
                // Case 3: Not found and in internal node (need to make sure subtree we call on has enough keys)
                if self.children[idx].keys.len() < (self.order - 1) / 2 + 1 {
                    if idx > 0 && self.children[idx - 1].keys.len() >= (self.order - 1) / 2 + 1  {
                        // Case 3a: Left subtree has at least floor(K/2) + 1 keys -> rotate to right
                        self.rotate_right(idx);
                    } else if idx < (self.children.len() - 1) && self.children[idx + 1].keys.len() >= (self.order - 1) / 2 + 1 {
                        // Case 3b: Right subtree has at least floor(K/2) + 1 keys -> rotate to left
                        self.rotate_left(idx);
                    } else {
                        // Case 3c: Both left and right do not have enough keys, so we merge them
                        if idx == (self.children.len() - 1) {
                            self.merge(idx - 1);
                            // Call delete on idx - 1
                            self.children[idx - 1].delete(value);
                            return;
                        } else {
                            self.merge(idx);
                        }
                    }
                }
                // Recursively call on child subtree that value belongs in
                self.children[idx].delete(value);
            } else {
                // Case 4: Not found at all (reached leaf node)
                panic!("Non-existant value cannot be deleted from BTree")
            }
        }
    }

    /// Helper to get a ref of the rightmost value in a subtree
    fn get_rightmost(&self) -> &T {
        let mut node = self;
        loop {
            if node.leaf {
                return node.keys.last().expect("Leaf node missing keys");
            }
            node = node.children.last().expect("Node missing children");
        }
    }

    /// Helper to get a ref of the leftmost value in a subtree
    fn get_leftmost(&self) -> &T {
        let mut node = self;
        loop {
            if node.leaf {
                return node.keys.first().expect("Leaf node missing keys");
            }
            node = node.children.first().expect("Node missing children");
        }
    }

    /// Helper that moves last key from left child to parent and parent key to right child's first key
    /// 
    /// Takes a child_idx that represents the right child's index
    fn rotate_right(&mut self, child_idx: usize) {
        // Remove the middle key from parent
        let middle_key = self.keys.remove(child_idx - 1);

        // Insert middle key into right child's first key
        self.children[child_idx].keys.insert(0, middle_key);

        // Remove last key from left child 
        let last_key = self.children[child_idx - 1].keys.pop().expect("Left child has no keys");

        // Insert last key into parent at child_idx
        self.keys.insert(child_idx - 1, last_key);

        // Move left child's last child to right child's first child
        if !self.children[child_idx - 1].leaf {
            let last_child = self.children[child_idx - 1].children.pop().expect("Left child has no children");
            self.children[child_idx].children.insert(0, last_child);
        }
    }

    /// Helper that moves first key from right child to parent and parent key to left child's last key
    /// 
    /// Takes a child_idx that represents the left child's index
    fn rotate_left(&mut self, child_idx: usize) {
        // Remove the middle key from parent
        let middle_key = self.keys.remove(child_idx);

        // Insert middle key into left child's last key
        self.children[child_idx].keys.push(middle_key);

        // Remove first key from right child 
        let first_key = self.children[child_idx + 1].keys.remove(0);

        // Insert first key into parent at child_idx
        self.keys.insert(child_idx, first_key);

        // Move right child's first child to left child's last child
        if !self.children[child_idx + 1].leaf {
            let first_child = self.children[child_idx + 1].children.remove(0);
            self.children[child_idx].children.push(first_child);
        }
    }

    /// Helper that merges two children nodes and inserts middle key into new child
    /// 
    /// Takes a child_idx that represents the left child
    fn merge(&mut self, child_idx: usize) {
        // Check if child index is greater than number of children
        if child_idx >= self.children.len() - 1 {
            panic!("Child index is greater than number of children");
        }

        // Remove the middle key from parent
        let middle_key = self.keys.remove(child_idx);

        // Remove right child (transfers ownership)
        let mut right_child = self.children.remove(child_idx + 1);

        // Get mutable ref of left child and then merge
        let left_child = &mut self.children[child_idx];
        left_child.keys.push(middle_key);
        left_child.keys.append(&mut right_child.keys);

        // Merge children if left child is not leaf
        if !left_child.leaf {
            left_child.children.append(&mut right_child.children);
        }

        // Right child gets automatically deallocated here (out of scope)
    }

    /// Helper (test) function for printing b-tree node structure
    #[cfg(test)]
    fn print_structure(&self, level: usize) {
        let indent = "  ".repeat(level);
        println!("{}Node (leaf={}): {:?}", indent, self.leaf, self.keys);
        
        for child in &self.children {
            child.print_structure(level + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_btree() {
        let btree: BTree<i32> = BTree::new(3);
        assert!(!btree.search(5));
    }

    #[test]
    fn test_insert_causes_root_split() {
        let mut btree = BTree::new(3);
        // Insert 3 values to force a split (max 2 keys for order 3)
        btree.insert(10);
        btree.insert(20);
        btree.insert(30);
        
        assert!(btree.search(10));
        assert!(btree.search(20));
        assert!(btree.search(30));
    }

    #[test]
    fn test_insert_ascending_order() {
        let mut btree = BTree::new(3);
        for i in 1..=10 {
            btree.insert(i);
        }
        
        for i in 1..=10 {
            assert!(btree.search(i));
        }
        assert!(!btree.search(11));
    }

    #[test]
    fn test_insert_descending_order() {
        let mut btree = BTree::new(3);
        for i in (1..=10).rev() {
            btree.insert(i);
        }
        
        for i in 1..=10 {
            assert!(btree.search(i));
        }
    }

    #[test]
    fn test_insert_random_order() {
        let mut btree = BTree::new(5);
        let values = vec![50, 30, 70, 20, 40, 60, 80, 10, 90];
        
        for val in &values {
            btree.insert(*val);
        }
        
        for val in &values {
            assert!(btree.search(*val));
        }
        assert!(!btree.search(25));
    }

    #[test]
    fn test_traverse_order() {
        let mut btree = BTree::new(3);
        btree.insert(5);
        btree.insert(3);
        btree.insert(7);
        btree.insert(1);
        btree.insert(9);
        
        // This will print to stdout - you can visually verify order
        println!("\nTraversal output:");
        btree.traverse();
    }

    #[test]
    fn test_search_empty_tree() {
        let btree: BTree<i32> = BTree::new(3);
        assert!(!btree.search(10));
    }

    #[test]
    fn test_larger_order() {
        let mut btree = BTree::new(5);
        // Order 5 means max 4 keys before split
        for i in 1..=20 {
            btree.insert(i);
        }
        
        for i in 1..=20 {
            assert!(btree.search(i));
        }
    }

    #[test]
    fn test_duplicate_search() {
        let mut btree = BTree::new(3);
        btree.insert(10);
        btree.insert(20);
        
        // Search multiple times for same value
        assert!(btree.search(10));
        assert!(btree.search(10));
        assert!(btree.search(20));
    }

    #[test]
    fn test_string_btree() {
        let mut btree = BTree::new(3);
        btree.insert("apple".to_string());
        btree.insert("banana".to_string());
        btree.insert("cherry".to_string());
        
        assert!(btree.search("apple".to_string()));
        assert!(btree.search("banana".to_string()));
        assert!(!btree.search("date".to_string()));
    }

    #[test]
    #[should_panic(expected = "BTree order must be at least 3")]
    fn test_invalid_order() {
        let _btree: BTree<i32> = BTree::new(2);
    }

    #[test]
    fn test_visualize_structure() {
        let mut btree = BTree::new(3);
        for i in 1..=7 {
            btree.insert(i);
        }
        btree.print_structure();
    }

    #[test]
    fn test_delete_from_leaf_simple() {
        let mut btree = BTree::new(3);
        btree.insert(10);
        btree.insert(20);

        btree.delete(10);
        assert!(!btree.search(10));
        assert!(btree.search(20));
    }

    #[test]
    fn test_delete_single_element() {
        let mut btree = BTree::new(3);
        btree.insert(10);

        btree.delete(10);
        assert!(!btree.search(10));
    }

    #[test]
    fn test_delete_from_leaf_with_sufficient_keys() {
        let mut btree = BTree::new(5);
        for i in 1..=10 {
            btree.insert(i);
        }

        btree.delete(5);
        assert!(!btree.search(5));
        for i in 1..=10 {
            if i != 5 {
                assert!(btree.search(i));
            }
        }
    }

    #[test]
    fn test_delete_causes_rotation_left() {
        let mut btree = BTree::new(3);
        // Build a tree that will require left rotation
        for i in 1..=7 {
            btree.insert(i);
        }

        println!("\nBefore delete:");
        btree.print_structure();

        // Delete to trigger rotation
        btree.delete(1);

        println!("\nAfter delete:");
        btree.print_structure();

        assert!(!btree.search(1));
        for i in 2..=7 {
            assert!(btree.search(i));
        }
    }

    #[test]
    fn test_delete_causes_rotation_right() {
        let mut btree = BTree::new(3);
        // Build a tree that will require right rotation
        for i in (1..=7).rev() {
            btree.insert(i);
        }

        println!("\nBefore delete:");
        btree.print_structure();

        // Delete to trigger rotation
        btree.delete(7);

        println!("\nAfter delete:");
        btree.print_structure();

        assert!(!btree.search(7));
        for i in 1..=6 {
            assert!(btree.search(i));
        }
    }

    #[test]
    fn test_delete_causes_merge() {
        let mut btree = BTree::new(3);
        // Build specific structure to test merge
        for i in 1..=6 {
            btree.insert(i);
        }

        println!("\nBefore delete (merge test):");
        btree.print_structure();

        btree.delete(6);
        btree.delete(5);

        println!("\nAfter deletes:");
        btree.print_structure();

        assert!(!btree.search(6));
        assert!(!btree.search(5));
        for i in 1..=4 {
            assert!(btree.search(i));
        }
    }

    #[test]
    fn test_delete_from_internal_node_case_2a() {
        let mut btree = BTree::new(3);
        // Insert values to create an internal node
        for i in 1..=10 {
            btree.insert(i);
        }

        println!("\nBefore delete from internal:");
        btree.print_structure();

        // Delete a value that's likely in an internal node
        btree.delete(4);

        println!("\nAfter delete from internal:");
        btree.print_structure();

        assert!(!btree.search(4));
        for i in 1..=10 {
            if i != 4 {
                assert!(btree.search(i));
            }
        }
    }

    #[test]
    fn test_delete_from_internal_node_case_2b() {
        let mut btree = BTree::new(3);
        for i in 1..=10 {
            btree.insert(i);
        }

        // Delete value that will trigger case 2b (successor replacement)
        btree.delete(7);

        assert!(!btree.search(7));
        for i in 1..=10 {
            if i != 7 {
                assert!(btree.search(i));
            }
        }
    }

    #[test]
    fn test_delete_from_internal_node_case_2c() {
        let mut btree = BTree::new(3);
        // Build tree and delete to trigger case 2c (merge children)
        for i in 1..=7 {
            btree.insert(i);
        }

        println!("\nBefore case 2c:");
        btree.print_structure();

        btree.delete(4);

        println!("\nAfter case 2c:");
        btree.print_structure();

        assert!(!btree.search(4));
    }

    #[test]
    fn test_delete_multiple_sequential() {
        let mut btree = BTree::new(3);
        for i in 1..=10 {
            btree.insert(i);
        }

        // Delete multiple values
        for i in 1..=5 {
            println!("\nDeleting {}", i);
            btree.delete(i);
            btree.print_structure();
        }

        for i in 1..=5 {
            assert!(!btree.search(i));
        }
        for i in 6..=10 {
            assert!(btree.search(i));
        }
    }

    #[test]
    fn test_delete_all_elements() {
        let mut btree = BTree::new(3);
        let values = vec![1, 2, 3, 4, 5, 6, 7];

        for val in &values {
            btree.insert(*val);
        }

        for val in &values {
            btree.delete(*val);
            assert!(!btree.search(*val));
        }
    }

    #[test]
    fn test_delete_descending_order() {
        let mut btree = BTree::new(3);
        for i in 1..=10 {
            btree.insert(i);
        }

        // Delete in descending order
        for i in (1..=10).rev() {
            btree.delete(i);
        }

        for i in 1..=10 {
            assert!(!btree.search(i));
        }
    }

    #[test]
    fn test_delete_random_order() {
        let mut btree = BTree::new(5);
        let values = vec![50, 30, 70, 20, 40, 60, 80, 10, 90];

        for val in &values {
            btree.insert(*val);
        }

        let delete_order = vec![30, 70, 10, 90, 50];
        for val in &delete_order {
            btree.delete(*val);
            assert!(!btree.search(*val));
        }

        // Check remaining values
        for val in &values {
            if delete_order.contains(val) {
                assert!(!btree.search(*val));
            } else {
                assert!(btree.search(*val));
            }
        }
    }

    #[test]
    fn test_delete_root_shrinkage() {
        let mut btree = BTree::new(3);
        // Build a tree that will shrink when root becomes empty
        for i in 1..=3 {
            btree.insert(i);
        }

        println!("\nBefore root shrinkage:");
        btree.print_structure();

        btree.delete(1);
        btree.delete(2);

        println!("\nAfter root shrinkage:");
        btree.print_structure();

        assert!(btree.search(3));
    }

    #[test]
    #[should_panic(expected = "Cannot delete from empty BTree")]
    fn test_delete_from_empty_tree() {
        let mut btree: BTree<i32> = BTree::new(3);
        btree.delete(10);
    }

    #[test]
    #[should_panic(expected = "Non-existant value cannot be deleted from BTree")]
    fn test_delete_nonexistent_value() {
        let mut btree = BTree::new(3);
        btree.insert(10);
        btree.insert(20);
        btree.delete(15); // Should panic
    }

    #[test]
    fn test_delete_with_larger_order() {
        let mut btree = BTree::new(7);
        for i in 1..=50 {
            btree.insert(i);
        }

        // Delete every third element
        for i in (1..=50).step_by(3) {
            btree.delete(i);
        }

        // Verify deletions
        for i in 1..=50 {
            if i % 3 == 1 {
                assert!(!btree.search(i));
            } else {
                assert!(btree.search(i));
            }
        }
    }

    #[test]
    fn test_delete_insert_interleaved() {
        let mut btree = BTree::new(3);

        // Insert some values
        for i in 1..=10 {
            btree.insert(i);
        }

        // Delete and insert interleaved
        btree.delete(5);
        btree.insert(15);
        btree.delete(3);
        btree.insert(13);

        assert!(!btree.search(5));
        assert!(!btree.search(3));
        assert!(btree.search(15));
        assert!(btree.search(13));
    }

    #[test]
    fn test_delete_string_values() {
        let mut btree = BTree::new(3);
        let values = vec!["apple", "banana", "cherry", "date", "elderberry"];

        for val in &values {
            btree.insert(val.to_string());
        }

        btree.delete("banana".to_string());
        btree.delete("date".to_string());

        assert!(!btree.search("banana".to_string()));
        assert!(!btree.search("date".to_string()));
        assert!(btree.search("apple".to_string()));
        assert!(btree.search("cherry".to_string()));
        assert!(btree.search("elderberry".to_string()));
    }

    #[test]
    fn test_delete_maintains_btree_properties() {
        let mut btree = BTree::new(5);

        // Insert many values
        for i in 1..=100 {
            btree.insert(i);
        }

        // Delete half of them
        for i in (1..=100).step_by(2) {
            btree.delete(i);
        }

        println!("\nAfter deleting half:");
        btree.print_structure();

        // Verify remaining values are searchable
        for i in 1..=100 {
            if i % 2 == 1 {
                assert!(!btree.search(i));
            } else {
                assert!(btree.search(i));
            }
        }
    }
}