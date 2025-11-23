use std::{
    alloc::System,
    fs::File,
    hint::black_box,
    io::{
        BufReader,
        Read,
    },
    path::PathBuf,
    sync::LazyLock,
};

use blink_alloc::BlinkAlloc;
use bumpalo::Bump;
use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use thread_local_allocator::bumpalo::ThreadLocalBump;
use ferreline::celeste_map::{
    CelesteMap,
    element::Element,
    lookup::Lookup,
};

static CELESTE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    std::env::var_os("CELESTE_PATH")
        .expect("CELESTE_PATH envir should be set")
        .into()
});

fn visit<A: allocator_api2::alloc::Allocator>(elem: &Element<A>) {
    black_box(&elem.name);
    for a in elem.attributes.iter() {
        black_box(a);
    }
    for c in elem.children.iter() {
        visit(c);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let map_path = CELESTE_PATH.join("Content/Maps/LostLevels.bin");
    let map_file = File::open(map_path).unwrap();
    let mut buf = vec![];
    BufReader::new(&map_file).read_to_end(&mut buf).unwrap();
    let mut bump = Bump::new();
    c.bench_function("bump create", |b| {
        b.iter(|| {
            bump.reset();
            let mut reader = buf.as_slice();
            let _package_name = CelesteMap::read_package_name_in(&bump, &mut reader).unwrap();
            let lookup = Lookup::read_in(&bump, &mut reader).unwrap();
            let map = CelesteMap::read_in(&bump, &mut reader, &lookup).unwrap();
            black_box(map);
        })
    });
    let mut blink = BlinkAlloc::new();
    c.bench_function("blink create", |b| {
        b.iter(|| {
            blink.reset();
            let mut reader = buf.as_slice();
            let _package_name = CelesteMap::read_package_name_in(&blink, &mut reader).unwrap();
            let lookup = Lookup::read_in(&blink, &mut reader).unwrap();
            let map = CelesteMap::read_in(&blink, &mut reader, &lookup).unwrap();
            black_box(map);
        })
    });
    c.bench_function("thread local bump create", |b| {
        b.iter(|| {
            ThreadLocalBump::BUMP.with_borrow_mut(|bump| bump.reset());
            let mut reader = buf.as_slice();
            let _package_name = CelesteMap::read_package_name_in(ThreadLocalBump, &mut reader).unwrap();
            let lookup = Lookup::read_in(ThreadLocalBump, &mut reader).unwrap();
            let map = CelesteMap::read_in(ThreadLocalBump, &mut reader, &lookup).unwrap();
            black_box(map);
        })
    });
    c.bench_function("system create", |b| {
        b.iter(|| {
            let mut reader = buf.as_slice();
            let _package_name = CelesteMap::read_package_name_in(System, &mut reader).unwrap();
            let lookup = Lookup::read_in(System, &mut reader).unwrap();
            let map = CelesteMap::read_in(System, &mut reader, &lookup).unwrap();
            black_box(map);
        })
    });
    bump.reset();
    let mut reader = buf.as_slice();
    let _package_name = CelesteMap::read_package_name_in(&bump, &mut reader).unwrap();
    let lookup = Lookup::read_in(&bump, &mut reader).unwrap();
    let map = CelesteMap::read_in(&bump, &mut reader, &lookup).unwrap();
    c.bench_function("bump visit", |b| {
        b.iter(|| {
            visit(&map.tree);
        })
    });
    blink.reset();
    let mut reader = buf.as_slice();
    let _package_name = CelesteMap::read_package_name_in(&blink, &mut reader).unwrap();
    let lookup = Lookup::read_in(&blink, &mut reader).unwrap();
    let map = CelesteMap::read_in(&blink, &mut reader, &lookup).unwrap();
    c.bench_function("blink visit", |b| {
        b.iter(|| {
            visit(&map.tree);
        })
    });
    ThreadLocalBump::BUMP.with_borrow_mut(|bump| bump.reset());
    let mut reader = buf.as_slice();
    let _package_name = CelesteMap::read_package_name_in(ThreadLocalBump, &mut reader).unwrap();
    let lookup = Lookup::read_in(ThreadLocalBump, &mut reader).unwrap();
    let map = CelesteMap::read_in(ThreadLocalBump, &mut reader, &lookup).unwrap();
    c.bench_function("thread local bump visit", |b| {
        b.iter(|| {
            visit(&map.tree);
        })
    });
    let mut reader = buf.as_slice();
    let _package_name = CelesteMap::read_package_name_in(System, &mut reader).unwrap();
    let lookup = Lookup::read_in(System, &mut reader).unwrap();
    let map = CelesteMap::read_in(System, &mut reader, &lookup).unwrap();
    c.bench_function("system visit", |b| {
        b.iter(|| {
            visit(&map.tree);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
