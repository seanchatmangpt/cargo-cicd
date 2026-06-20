/// Bump-allocator-style arena for complex object graphs.
///
/// Many WASM workloads need stable, index-addressable storage for trees or
/// graphs (e.g., process-event traces, dependency graphs) without the overhead
/// of heap-pointer chasing.  [`Arena<T>`] provides:
///
/// - O(1) allocation via `Vec::push`
/// - O(1) indexed access via `Vec::get`
/// - Stable [`NodeId`]s that are valid for the lifetime of the arena
/// - No unsafe code
///
/// # Example
/// ```
/// use project_wasm::arena::{Arena, NodeId};
///
/// let mut arena: Arena<String> = Arena::new();
/// let id: NodeId = arena.alloc("hello".to_owned());
/// assert_eq!(arena.get(id), Some(&"hello".to_owned()));
/// ```
///
/// # Limitations
/// Items cannot be removed individually (only the whole arena can be dropped).
/// If you need deletion, consider a free-list variant built on top of this one.

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Opaque, stable index into an [`Arena<T>`].
///
/// Uses `u32` so it fits in a JavaScript `number` without precision loss and
/// can be stored cheaply in WASM linear memory.
pub type NodeId = u32;

/// An append-only arena allocator backed by a `Vec<T>`.
#[derive(Debug)]
pub struct Arena<T> {
    items: Vec<T>,
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl<T> Arena<T> {
    /// Create an empty arena.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Create an empty arena with pre-allocated capacity to avoid
    /// reallocations when the expected number of items is known upfront.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    /// Append `value` to the arena and return its stable [`NodeId`].
    ///
    /// # Panics
    /// Panics if the arena already contains `u32::MAX` items (extremely
    /// unlikely in practice; a 32-bit index can address ~4 billion nodes).
    pub fn alloc(&mut self, value: T) -> NodeId {
        let id = u32::try_from(self.items.len())
            .expect("arena overflow: more than u32::MAX items allocated");
        self.items.push(value);
        id
    }

    /// Return a shared reference to the item at `id`, or `None` if `id` is
    /// out of range.
    pub fn get(&self, id: NodeId) -> Option<&T> {
        self.items.get(id as usize)
    }

    /// Return an exclusive reference to the item at `id`, or `None` if `id`
    /// is out of range.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.items.get_mut(id as usize)
    }

    /// Return the number of items currently stored in the arena.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Return `true` if the arena contains no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Iterate over all items in allocation order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    /// Iterate over `(NodeId, &T)` pairs in allocation order.
    pub fn iter_with_ids(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.items
            .iter()
            .enumerate()
            .map(|(i, v)| (i as NodeId, v))
    }

    /// Consume the arena and return all items as a `Vec<T>`.
    pub fn into_vec(self) -> Vec<T> {
        self.items
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_arena_is_empty() {
        let arena: Arena<i32> = Arena::new();
        assert!(arena.is_empty());
        assert_eq!(arena.len(), 0);
    }

    #[test]
    fn alloc_returns_sequential_ids() {
        let mut arena: Arena<&str> = Arena::new();
        let id0 = arena.alloc("first");
        let id1 = arena.alloc("second");
        let id2 = arena.alloc("third");
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn get_returns_correct_item() {
        let mut arena: Arena<u64> = Arena::new();
        let id = arena.alloc(42_u64);
        assert_eq!(arena.get(id), Some(&42_u64));
    }

    #[test]
    fn get_out_of_range_returns_none() {
        let arena: Arena<i32> = Arena::new();
        assert_eq!(arena.get(0), None);
        assert_eq!(arena.get(u32::MAX), None);
    }

    #[test]
    fn get_mut_allows_in_place_update() {
        let mut arena: Arena<String> = Arena::new();
        let id = arena.alloc("hello".to_owned());
        if let Some(s) = arena.get_mut(id) {
            s.push_str(", world");
        }
        assert_eq!(arena.get(id), Some(&"hello, world".to_owned()));
    }

    #[test]
    fn len_tracks_allocations() {
        let mut arena: Arena<()> = Arena::new();
        assert_eq!(arena.len(), 0);
        arena.alloc(());
        assert_eq!(arena.len(), 1);
        arena.alloc(());
        arena.alloc(());
        assert_eq!(arena.len(), 3);
    }

    #[test]
    fn iter_visits_all_items_in_order() {
        let mut arena: Arena<i32> = Arena::new();
        for v in [10, 20, 30] {
            arena.alloc(v);
        }
        let collected: Vec<i32> = arena.iter().copied().collect();
        assert_eq!(collected, vec![10, 20, 30]);
    }

    #[test]
    fn iter_with_ids_yields_correct_pairs() {
        let mut arena: Arena<&str> = Arena::new();
        arena.alloc("a");
        arena.alloc("b");
        arena.alloc("c");
        let pairs: Vec<(NodeId, &&str)> = arena.iter_with_ids().collect();
        assert_eq!(pairs[0], (0, &"a"));
        assert_eq!(pairs[1], (1, &"b"));
        assert_eq!(pairs[2], (2, &"c"));
    }

    #[test]
    fn with_capacity_does_not_affect_semantics() {
        let mut arena: Arena<u32> = Arena::with_capacity(100);
        assert!(arena.is_empty());
        let id = arena.alloc(7);
        assert_eq!(arena.get(id), Some(&7));
        assert_eq!(arena.len(), 1);
    }

    #[test]
    fn into_vec_consumes_arena() {
        let mut arena: Arena<i32> = Arena::new();
        arena.alloc(1);
        arena.alloc(2);
        arena.alloc(3);
        let v = arena.into_vec();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn default_produces_empty_arena() {
        let arena: Arena<String> = Arena::default();
        assert!(arena.is_empty());
    }

    // Simulate a simple binary-tree stored in an arena
    #[test]
    fn tree_stored_in_arena() {
        #[derive(Debug)]
        struct Node {
            value: i32,
            left: Option<NodeId>,
            right: Option<NodeId>,
        }

        let mut arena: Arena<Node> = Arena::new();

        // Build:   2
        //         / \
        //        1   3
        let left = arena.alloc(Node {
            value: 1,
            left: None,
            right: None,
        });
        let right = arena.alloc(Node {
            value: 3,
            left: None,
            right: None,
        });
        let root = arena.alloc(Node {
            value: 2,
            left: Some(left),
            right: Some(right),
        });

        assert_eq!(arena.get(root).unwrap().value, 2);
        let root_left_id = arena.get(root).unwrap().left.unwrap();
        assert_eq!(arena.get(root_left_id).unwrap().value, 1);
        let root_right_id = arena.get(root).unwrap().right.unwrap();
        assert_eq!(arena.get(root_right_id).unwrap().value, 3);
        assert_eq!(arena.len(), 3);
    }
}
