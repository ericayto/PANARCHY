//! World - central ECS container

use std::any::{Any, TypeId};
use std::collections::HashMap;
use super::{Entity, Component, TypedComponentStorage};
use super::entity::EntityAllocator;

/// World holds all entities and components
pub struct World {
    entities: EntityAllocator,
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::new(),
            components: HashMap::new(),
        }
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> Entity {
        self.entities.allocate()
    }

    /// Destroy an entity and remove all its components
    pub fn destroy_entity(&mut self, entity: Entity) {
        if !self.entities.is_alive(entity) {
            return;
        }

        // For Phase 0, we skip component cleanup during entity destruction
        // In a full implementation, we'd iterate over all storages and remove the entity

        self.entities.deallocate(entity);
    }

    /// Check if entity is alive
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    /// Get number of alive entities
    pub fn entity_count(&self) -> usize {
        self.entities.count()
    }

    /// Add a component to an entity
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        if !self.entities.is_alive(entity) {
            return;
        }

        let type_id = TypeId::of::<T>();
        let storage = self.components
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedComponentStorage::<T>::new()));

        if let Some(storage) = storage.downcast_mut::<TypedComponentStorage<T>>() {
            storage.insert(entity.id, component);
        }
    }

    /// Get a component from an entity
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let storage = self.components.get(&type_id)?;
        
        if let Some(storage) = storage.downcast_ref::<TypedComponentStorage<T>>() {
            return storage.get(entity.id);
        }
        None
    }

    /// Get a mutable component from an entity
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let storage = self.components.get_mut(&type_id)?;
        
        if let Some(storage) = storage.downcast_mut::<TypedComponentStorage<T>>() {
            return storage.get_mut(entity.id);
        }
        None
    }

    /// Get storage for a component type
    pub fn get_storage<T: Component>(&self) -> Option<&TypedComponentStorage<T>> {
        let type_id = TypeId::of::<T>();
        let storage = self.components.get(&type_id)?;
        storage.downcast_ref::<TypedComponentStorage<T>>()
    }

    /// Get mutable storage for a component type
    pub fn get_storage_mut<T: Component>(&mut self) -> Option<&mut TypedComponentStorage<T>> {
        let type_id = TypeId::of::<T>();
        let storage = self.components.get_mut(&type_id)?;
        storage.downcast_mut::<TypedComponentStorage<T>>()
    }

    /// Check if entity has a component
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        if let Some(storage) = self.get_storage::<T>() {
            return storage.data.contains_key(&entity.id);
        }
        false
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position { x: f32, y: f32 }
    impl Component for Position {}

    #[derive(Debug, PartialEq)]
    struct Velocity { dx: f32, dy: f32 }
    impl Component for Velocity {}

    #[test]
    fn test_world_entity_lifecycle() {
        let mut world = World::new();
        
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        
        assert!(world.is_alive(e1));
        assert!(world.is_alive(e2));
        assert_eq!(world.entity_count(), 2);
        
        world.destroy_entity(e1);
        assert!(!world.is_alive(e1));
        assert!(world.is_alive(e2));
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_world_components() {
        let mut world = World::new();
        
        let entity = world.create_entity();
        world.add_component(entity, Position { x: 1.0, y: 2.0 });
        world.add_component(entity, Velocity { dx: 0.5, dy: 0.5 });
        
        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Velocity>(entity));
        
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 1.0);
        
        if let Some(vel) = world.get_component_mut::<Velocity>(entity) {
            vel.dx = 1.0;
        }
        
        let vel = world.get_component::<Velocity>(entity).unwrap();
        assert_eq!(vel.dx, 1.0);
    }
}
