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

pub const SPRITE_GROUND: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/sprites/terrain_grass.png"
));
pub const SPRITE_SOIL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/sprites/terrain_mud.png"
));
pub const SPRITE_WATER: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/sprites/water.png"
));
pub const SPRITE_ROAD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/sprites/road.png"
));
pub const SPRITE_RESIDENTIAL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/sprites/residential.png"
));
pub const SPRITE_COMMERCIAL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/sprites/commercial.png"
));
pub const SPRITE_INDUSTRIAL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/sprites/industrial.png"
));
pub const SPRITE_PARK: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/web/assets/sprites/park.png"
));

pub fn sprite(name: &str) -> Option<&'static [u8]> {
    match name {
        "terrain_grass.png" | "ground.png" => Some(SPRITE_GROUND),
        "terrain_mud.png" | "soil.png" => Some(SPRITE_SOIL),
        "water.png" => Some(SPRITE_WATER),
        "road.png" => Some(SPRITE_ROAD),
        "residential.png" => Some(SPRITE_RESIDENTIAL),
        "commercial.png" => Some(SPRITE_COMMERCIAL),
        "industrial.png" => Some(SPRITE_INDUSTRIAL),
        "park.png" => Some(SPRITE_PARK),
        _ => None,
    }
}
