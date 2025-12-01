use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::LazyLock,
};

use ferreline::celeste_map::codec::CelesteMap;

static CELESTE_PATH: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    std::env::var_os("CELESTE_PATH").map(Into::into)
});

/// XML deserialization not yet supported
#[test]
fn test_serde() -> anyhow::Result<()> {
    let celeste_path = CELESTE_PATH
        .as_ref()
        .ok_or(anyhow::anyhow!("CELESTE_PATH not set"))?;
    let map_path = celeste_path.join("Content/Maps/LostLevels.bin");
    let map_file = File::open(map_path)?;
    let mut reader = BufReader::new(map_file);
    let map = CelesteMap::read(&mut reader)?;
    let dede = serde_json::to_string(&map)?;
    let sered: CelesteMap = serde_json::from_str(&dede)?;
    assert_eq!("LostLevels", sered.package_name.as_str());
    let dede = ron::to_string(&map)?;
    let sered: CelesteMap = ron::from_str(&dede)?;
    assert_eq!("LostLevels", sered.package_name.as_str());
    // let dede = serde_xml_rs::to_string(&map)?;
    // let sered: CelesteMap = serde_xml_rs::from_str(&dede)?;
    // assert_eq!("LostLevels", sered.package_name.as_str());
    Ok(())
}
