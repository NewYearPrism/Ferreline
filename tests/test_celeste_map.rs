use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::LazyLock,
};

use ferreline::celeste_map::{
    CelesteMap,
    lookup::Lookup,
};
use thread_local_allocator::bumpalo::ThreadLocalBump;

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
    let bump = bumpalo::Bump::new();
    let package_name = CelesteMap::read_package_name_in(&bump, &mut reader)?;
    let lookup = Lookup::read_in(&bump, &mut reader)?;
    let _map = CelesteMap::read_in(&bump, &mut reader, &lookup)?;
    assert_eq!("LostLevels".as_bytes(), &*package_name);
    println!("Bumped: {}", bump.allocated_bytes());
    Ok(())
}

#[test]
fn test_zst_alloc() -> anyhow::Result<()> {
    let celeste_path = CELESTE_PATH
        .as_ref()
        .ok_or(anyhow::anyhow!("CELESTE_PATH not set"))?;
    let map_path = celeste_path.join("Content/Maps/LostLevels.bin");
    let map_file = File::open(map_path)?;
    let mut reader = BufReader::new(map_file);
    let package_name = CelesteMap::read_package_name_in(ThreadLocalBump, &mut reader)?;
    let lookup = Lookup::read_in(ThreadLocalBump, &mut reader)?;
    let _map = CelesteMap::read_in(ThreadLocalBump, &mut reader, &lookup)?;
    assert_eq!("LostLevels".as_bytes(), &*package_name);
    ThreadLocalBump::BUMP
        .with_borrow(|bump| println!("Thread local bumped: {}", bump.allocated_bytes()));
    Ok(())
}
