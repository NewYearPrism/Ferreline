use std::{
    fs::File,
    hint::black_box,
    io::{
        BufReader,
        Read,
    },
    path::PathBuf,
    sync::LazyLock,
};

use allocator_api2::alloc::Global;
use bumpalo::Bump;
use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use directed_visit::{
    Direct,
    Visitor,
};
use ferreline::celeste_map::{
    CelesteMap,
    element::Element,
    visit::ElementDirector,
};

static CELESTE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    std::env::var_os("CELESTE_PATH")
        .expect("CELESTE_PATH envir should be set")
        .into()
});

struct BenchElementVisitor;

impl<Rc: shared_vector::RefCount, A: allocator_api2::alloc::Allocator>
    directed_visit::Visit<Element<Rc, A>> for BenchElementVisitor
{
    fn visit<D: Direct<Self, Element<Rc, A>> + ?Sized>(
        visitor: Visitor<'_, D, Self>,
        node: &Element<Rc, A>,
    ) {
        black_box(&node.name);
        black_box(&node.attributes);
        Visitor::visit(visitor, node)
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let map_path = CELESTE_PATH.join("Content/Maps/LostLevels.bin");
    let map_file = File::open(map_path).expect("Cannot open map file");
    let mut buf = vec![];
    BufReader::new(&map_file)
        .read_to_end(&mut buf)
        .expect("Failed to read map file");
    let mut bump = Bump::new();
    c.bench_function("bump create", |b| {
        b.iter(|| {
            bump.reset();
            let map = CelesteMap::read(&bump, buf.as_slice()).expect("Cannot load map");
            black_box(map);
        })
    });
    c.bench_function("global create", |b| {
        b.iter(|| {
            let map = CelesteMap::read(&Global, buf.as_slice()).expect("Cannot load map");
            black_box(map);
        })
    });
    let map_bump = CelesteMap::read(&bump, buf.as_slice()).expect("Cannot load map");
    c.bench_function("bump visit", |b| {
        b.iter(|| {
            directed_visit::visit(
                &mut ElementDirector,
                &mut BenchElementVisitor,
                &map_bump.tree,
            );
        })
    });
    let map_global = CelesteMap::read(&Global, buf.as_slice()).expect("Cannot load map");
    c.bench_function("global visit", |b| {
        b.iter(|| {
            directed_visit::visit(
                &mut ElementDirector,
                &mut BenchElementVisitor,
                &map_global.tree,
            );
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
