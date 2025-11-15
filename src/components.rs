#[derive(Clone, Debug)]
pub struct Tile {
    pub id: u32,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Environment {
    pub temperature_c: f32,
    pub precipitation_mm: f32,
    pub fertility: f32,
    pub seasonal_phase: f32,
}

impl Environment {
    pub fn new(temperature_c: f32, precipitation_mm: f32, fertility: f32) -> Self {
        Self {
            temperature_c,
            precipitation_mm,
            fertility,
            seasonal_phase: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PopulationGroup {
    pub tile_id: u32,
    pub count: u32,
    pub mean_age_years: f32,
}

impl PopulationGroup {
    pub fn new(tile_id: u32, count: u32, mean_age_years: f32) -> Self {
        Self {
            tile_id,
            count,
            mean_age_years,
        }
    }
}
