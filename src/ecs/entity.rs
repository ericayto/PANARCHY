//! Entity management

use std::collections::HashSet;

/// Entity ID type - simple numeric ID
pub type EntityId = u64;

/// Entity struct
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: EntityId,
    pub generation: u32,
}

impl Entity {
    pub fn new(id: EntityId) -> Self {
        Self { id, generation: 0 }
    }
}

/// Entity allocator
pub struct EntityAllocator {
    next_id: EntityId,
    free_list: Vec<EntityId>,
    alive: HashSet<EntityId>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            free_list: Vec::new(),
            alive: HashSet::new(),
        }
    }

    pub fn allocate(&mut self) -> Entity {
        let id = if let Some(id) = self.free_list.pop() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        };
        self.alive.insert(id);
        Entity::new(id)
    }

    pub fn deallocate(&mut self, entity: Entity) {
        if self.alive.remove(&entity.id) {
            self.free_list.push(entity.id);
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.alive.contains(&entity.id)
    }

    pub fn count(&self) -> usize {
        self.alive.len()
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_allocation() {
        let mut allocator = EntityAllocator::new();
        
        let e1 = allocator.allocate();
        assert_eq!(e1.id, 0);
        assert!(allocator.is_alive(e1));
        
        let e2 = allocator.allocate();
        assert_eq!(e2.id, 1);
        assert!(allocator.is_alive(e2));
        
        assert_eq!(allocator.count(), 2);
    }

    #[test]
    fn test_entity_deallocation() {
        let mut allocator = EntityAllocator::new();
        
        let e1 = allocator.allocate();
        let e2 = allocator.allocate();
        
        allocator.deallocate(e1);
        assert!(!allocator.is_alive(e1));
        assert!(allocator.is_alive(e2));
        assert_eq!(allocator.count(), 1);
        
        // Should reuse the deallocated ID
        let e3 = allocator.allocate();
        assert_eq!(e3.id, 0);
    }
}
