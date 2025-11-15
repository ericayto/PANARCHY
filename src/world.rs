use crate::components::{Environment, PopulationGroup, Tile};

#[derive(Clone, Debug)]
pub struct TileState {
    pub tile: Tile,
    pub environment: Environment,
}

#[derive(Clone, Debug, Default)]
pub struct WorldState {
    pub tiles: Vec<TileState>,
    pub populations: Vec<PopulationGroup>,
}

impl WorldState {
    pub fn add_tile(&mut self, tile: Tile, environment: Environment) {
        self.tiles.push(TileState { tile, environment });
    }

    pub fn add_population(&mut self, population: PopulationGroup) {
        self.populations.push(population);
    }

    pub fn total_population(&self) -> u64 {
        self.populations
            .iter()
            .map(|group| group.count as u64)
            .sum()
    }
}
