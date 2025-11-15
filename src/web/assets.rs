pub const INDEX_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/index.html"
));
pub const STYLES_CSS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/styles.css"
));
pub const APP_JS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/app.js"
));
pub const SPRITE_TERRAIN_GRASS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/cytopia/terrain_grass.png"
));
pub const SPRITE_TERRAIN_GRAVEL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/cytopia/terrain_gravel.png"
));
pub const SPRITE_TERRAIN_MUD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/cytopia/terrain_mud.png"
));
pub const SPRITE_ROAD_EUROPEAN: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/cytopia/road_european.png"
));
pub const SPRITE_RESIDENTIAL_A: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/cytopia/residential_a.png"
));
pub const SPRITE_RESIDENTIAL_B: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/cytopia/residential_b.png"
));
pub const SPRITE_COMMERCIAL_A: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/cytopia/commercial_a.png"
));
pub const SPRITE_INDUSTRIAL_A: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/cytopia/industrial_a.png"
));
pub const SPRITE_CARS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/cytopia/cars.png"
));

pub fn sprite(name: &str) -> Option<&'static [u8]> {
    match name {
        "terrain_grass.png" | "ground.png" => Some(SPRITE_TERRAIN_GRASS),
        "terrain_gravel.png" | "gravel.png" => Some(SPRITE_TERRAIN_GRAVEL),
        "terrain_mud.png" | "soil.png" => Some(SPRITE_TERRAIN_MUD),
        "road_european.png" | "road.png" => Some(SPRITE_ROAD_EUROPEAN),
        "residential_a.png" | "residential.png" => Some(SPRITE_RESIDENTIAL_A),
        "residential_b.png" => Some(SPRITE_RESIDENTIAL_B),
        "commercial_a.png" | "commercial.png" => Some(SPRITE_COMMERCIAL_A),
        "industrial_a.png" | "industrial.png" => Some(SPRITE_INDUSTRIAL_A),
        "cars.png" => Some(SPRITE_CARS),
        _ => None,
    }
}
