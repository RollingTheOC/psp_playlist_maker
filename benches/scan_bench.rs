use criterion::{criterion_group, criterion_main, Criterion};
use psp_playlist_maker::music::MusicLibrary;

fn bench_scan(c: &mut Criterion) {
    c.bench_function("scan_dir", |b| {
        b.iter(|| MusicLibrary::scan_dir("/mnt/psp/MUSIC"))
    });
}

criterion_group!(benches, bench_scan);
criterion_main!(benches);
