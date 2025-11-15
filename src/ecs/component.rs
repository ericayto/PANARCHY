//! Component storage with Structure of Arrays (SoA) layout

use std::collections::HashMap;
use super::EntityId;

/// Trait for components
pub trait Component: Send + Sync + 'static {}

/// Type-erased component storage
pub trait ComponentStorage: Send + Sync {
    fn remove(&mut self, entity_id: EntityId);
    fn has(&self, entity_id: EntityId) -> bool;
    fn clear(&mut self);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Concrete storage for a specific component type
pub struct TypedComponentStorage<T: Component> {
    pub(crate) data: HashMap<EntityId, T>,
}

impl<T: Component> TypedComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity_id: EntityId, component: T) {
        self.data.insert(entity_id, component);
    }

    pub fn get(&self, entity_id: EntityId) -> Option<&T> {
        self.data.get(&entity_id)
    }

    pub fn get_mut(&mut self, entity_id: EntityId) -> Option<&mut T> {
        self.data.get_mut(&entity_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.data.iter().map(|(id, comp)| (*id, comp))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        self.data.iter_mut().map(|(id, comp)| (*id, comp))
    }
}

impl<T: Component> Default for TypedComponentStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> ComponentStorage for TypedComponentStorage<T> {
    fn remove(&mut self, entity_id: EntityId) {
        self.data.remove(&entity_id);
    }

    fn has(&self, entity_id: EntityId) -> bool {
        self.data.contains_key(&entity_id)
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[test]
    fn test_component_storage() {
        let mut storage = TypedComponentStorage::<Position>::new();
        
        storage.insert(1, Position { x: 1.0, y: 2.0 });
        storage.insert(2, Position { x: 3.0, y: 4.0 });
        
        assert_eq!(storage.len(), 2);
        assert!(storage.has(1));
        assert!(storage.has(2));
        assert!(!storage.has(3));
        
        let pos = storage.get(1).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        
        storage.remove(1);
        assert!(!storage.has(1));
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn test_component_iteration() {
        let mut storage = TypedComponentStorage::<Position>::new();
        
        storage.insert(1, Position { x: 1.0, y: 2.0 });
        storage.insert(2, Position { x: 3.0, y: 4.0 });
        
        let count = storage.iter().count();
        assert_eq!(count, 2);
        
        for (_id, pos) in storage.iter_mut() {
            pos.x += 1.0;
        }
        
        let pos = storage.get(1).unwrap();
        assert_eq!(pos.x, 2.0);
    }
}
