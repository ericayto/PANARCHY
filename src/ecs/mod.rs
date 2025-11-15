//! Entity Component System (ECS) implementation
//! 
//! Simple ECS with Structure of Arrays (SoA) layout for cache efficiency.

pub mod entity;
pub mod component;
pub mod world;

pub use entity::{Entity, EntityId};
pub use component::{Component, TypedComponentStorage};
pub use world::World;
