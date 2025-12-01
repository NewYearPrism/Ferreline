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

#[test]
fn test() -> anyhow::Result<()> {
    let celeste_path = CELESTE_PATH
        .as_ref()
        .ok_or(anyhow::anyhow!("CELESTE_PATH not set"))?;
    let map_path = celeste_path.join("Content/Maps/LostLevels.bin");
    let map_file = File::open(map_path)?;
    let mut reader = BufReader::new(map_file);
    let map = CelesteMap::read(&mut reader)?;
    assert_eq!("LostLevels", map.package_name.as_str());
    Ok(())
}
