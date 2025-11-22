use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::LazyLock,
};

use directed_visit::{
    Direct,
    Visitor,
};
use ferreline::celeste_map::{
    CelesteMap,
    element::Element,
};

static CELESTE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    std::env::var_os("CELESTE_PATH")
        .expect("CELESTE_PATH envir should be set")
        .into()
});

#[test]
fn test() {
    let map_path = CELESTE_PATH.join("Content/Maps/LostLevels.bin");
    let map_file = File::open(map_path).expect("Unable to open map file");
    let reader = BufReader::new(map_file);
    let bump = bumpalo::Bump::new();
    let map = CelesteMap::read(&bump, reader).expect("Failed to load map");
    assert_eq!("LostLevels".as_bytes(), map.package_name);
}

#[test]
fn test_visit() {
    use ferreline::celeste_map::visit::ElementDirector;

    struct TestElementVisitor;

    impl<Rc: shared_vector::RefCount, A: allocator_api2::alloc::Allocator>
        directed_visit::Visit<Element<Rc, A>> for TestElementVisitor
    {
        fn visit<D>(visitor: Visitor<'_, D, Self>, node: &Element<Rc, A>)
        where
            D: Direct<Self, Element<Rc, A>> + ?Sized,
        {
            println!("{:?}", str::from_utf8(&node.name).ok());
            Visitor::visit(visitor, node)
        }
    }

    let map_path = CELESTE_PATH.join("Content/Maps/LostLevels.bin");
    let map_file = File::open(map_path).expect("Unable to open map file");
    let reader = BufReader::new(map_file);
    let bump = bumpalo::Bump::new();
    let map = CelesteMap::read(&bump, reader).expect("Failed to load map");
    directed_visit::visit(&mut ElementDirector, &mut TestElementVisitor, &map.tree);
    println!("{}", bump.allocated_bytes());
}
