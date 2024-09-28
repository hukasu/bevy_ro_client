// Base data folder, where uncompressed files are placed
pub const BASE_DATA_FOLDER: &str = "data/";

// Folders for each archetype of file
/// Folder prefix for World files (`.rsw`)
pub const WORLD_FILES_FOLDER: &str = "data/";
/// Folder prefix for Ground files (`.gnd`)
pub const GROUND_FILES_FOLDER: &str = "data/";
/// Folder prefix for Model (animated props) files `.rsm`
pub const MODEL_FILES_FOLDER: &str = "data/model/";
/// Folder prefix for Sprite files (`.spr`)
pub const SPR_FILES_FOLDER: &str = "data/sprite/";
/// Folder prefix for Actor files (`.act`)
pub const ACT_FILES_FOLDER: &str = SPR_FILES_FOLDER;
/// Folder prefix for Texture files (`.bmp`, `.jpg`, `.tga`)
pub const TEXTURE_FILES_FOLDER: &str = "data/texture/";
/// Folder prefix for Audio files (`.wav`)
pub const WAV_FILES_FOLDER: &str = "data/wav/";

// Folders for specific types
/// Folder prefix for Texture files for water types (`.bmp`)
pub const WATER_TEXTURE_FILES_FOLDER: &str = "data/texture/워터/";
