//! Spatial model - tile-based world grid

use serde::{Deserialize, Serialize};
use crate::ecs::Component;

pub type TileId = u32;

/// Tile position in the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TilePos {
    pub x: u32,
    pub y: u32,
}

/// Tile grid representing the world
pub struct TileGrid {
    width: u32,
    height: u32,
}

impl TileGrid {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn tile_count(&self) -> u32 {
        self.width * self.height
    }

    /// Convert tile position to tile ID
    pub fn pos_to_id(&self, pos: TilePos) -> Option<TileId> {
        if pos.x < self.width && pos.y < self.height {
            Some(pos.y * self.width + pos.x)
        } else {
            None
        }
    }

    /// Convert tile ID to position
    pub fn id_to_pos(&self, id: TileId) -> Option<TilePos> {
        if id < self.tile_count() {
            Some(TilePos {
                x: id % self.width,
                y: id / self.width,
            })
        } else {
            None
        }
    }

    /// Get neighboring tiles (4-connectivity)
    pub fn neighbors(&self, pos: TilePos) -> Vec<TilePos> {
        let mut neighbors = Vec::new();
        
        // North
        if pos.y > 0 {
            neighbors.push(TilePos { x: pos.x, y: pos.y - 1 });
        }
        // South
        if pos.y < self.height - 1 {
            neighbors.push(TilePos { x: pos.x, y: pos.y + 1 });
        }
        // West
        if pos.x > 0 {
            neighbors.push(TilePos { x: pos.x - 1, y: pos.y });
        }
        // East
        if pos.x < self.width - 1 {
            neighbors.push(TilePos { x: pos.x + 1, y: pos.y });
        }
        
        neighbors
    }

    /// Manhattan distance between two positions
    pub fn distance(&self, a: TilePos, b: TilePos) -> u32 {
        let dx = if a.x > b.x { a.x - b.x } else { b.x - a.x };
        let dy = if a.y > b.y { a.y - b.y } else { b.y - a.y };
        dx + dy
    }
}

/// Location component - links entity to a tile
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub tile_id: TileId,
}

impl Component for Location {}

/// Environment indices for a tile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub temp_idx: f32,        // Temperature index
    pub precip_idx: f32,      // Precipitation index
    pub soil_fertility: f32,  // Soil fertility (0..1)
}

impl Component for Environment {}

/// Land use/cover for a tile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LandUse {
    Water,
    Forest,
    Cropland,
    Urban,
    Grassland,
    Desert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandCover {
    pub land_use: LandUse,
    pub fraction_urban: f32,
    pub fraction_forest: f32,
    pub fraction_cropland: f32,
}

impl Component for LandCover {}

/// Resource stocks for a tile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStock {
    pub mineral_tonnage: f32,
    pub ore_grade: f32,
    pub water_available: f32,
    pub biomass: f32,
}

impl Component for ResourceStock {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_grid() {
        let grid = TileGrid::new(10, 5);
        
        assert_eq!(grid.width(), 10);
        assert_eq!(grid.height(), 5);
        assert_eq!(grid.tile_count(), 50);
    }

    #[test]
    fn test_pos_id_conversion() {
        let grid = TileGrid::new(10, 5);
        
        let pos = TilePos { x: 3, y: 2 };
        let id = grid.pos_to_id(pos).unwrap();
        assert_eq!(id, 23); // 2 * 10 + 3
        
        let pos2 = grid.id_to_pos(id).unwrap();
        assert_eq!(pos, pos2);
    }

    #[test]
    fn test_neighbors() {
        let grid = TileGrid::new(10, 5);
        
        // Corner tile
        let pos = TilePos { x: 0, y: 0 };
        let neighbors = grid.neighbors(pos);
        assert_eq!(neighbors.len(), 2); // Only south and east
        
        // Middle tile
        let pos = TilePos { x: 5, y: 2 };
        let neighbors = grid.neighbors(pos);
        assert_eq!(neighbors.len(), 4); // All directions
    }

    #[test]
    fn test_distance() {
        let grid = TileGrid::new(10, 5);
        
        let a = TilePos { x: 0, y: 0 };
        let b = TilePos { x: 3, y: 4 };
        
        assert_eq!(grid.distance(a, b), 7); // 3 + 4
    }
}
