
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};

use moc::qty::Hpx;
use moc::moc::range::RangeMOC;
use moc::deser::fits::{
    from_fits_ivoa,
    MocIdxType, MocQtyType, MocType
};
use moc::moc::{HasMaxDepth, RangeMOCIntoIterator, CellMOCIntoIterator, CellMOCIterator};
use moc::moc::range::op::or::or;
use moc::ranges::SNORanges;

fn load_moc(filename: &str) -> RangeMOC<u32, Hpx<u32>> {
    let path_buf1 = PathBuf::from(format!("resources/{}", filename));
    let path_buf2 = PathBuf::from(format!("../resources/{}", filename));
    let file = File::open(&path_buf1).or_else(|_| File::open(&path_buf2)).unwrap();
    let reader = BufReader::new(file);
    match from_fits_ivoa(reader) {
        Ok(MocIdxType::U32(MocQtyType::Hpx(MocType::Ranges(moc)))) => {
            let moc = RangeMOC::new(moc.depth_max(), moc.collect());
            moc
        },
        Ok(MocIdxType::U32(MocQtyType::Hpx(MocType::Cells(moc)))) => {
            let moc = RangeMOC::new(
                moc.depth_max(),
                moc.into_cell_moc_iter().ranges().collect()
            );
            moc
        },
        _ => unreachable!("Type not supposed to be != from U32"),
    }
}

fn load_mocs() -> (RangeMOC<u32, Hpx<u32>>, RangeMOC<u32, Hpx<u32>>) {
    let sdss = load_moc("V_147_sdss12.moc.fits");
    let other = load_moc("CDS-I-125A-catalog_MOC.fits");
    (sdss, other)
}

fn test_or_ranges(moc_l: RangeMOC<u32, Hpx<u32>>, moc_r: RangeMOC<u32, Hpx<u32>>) -> RangeMOC<u32, Hpx<u32>> {
    let depth = u8::max(moc_l.depth_max(), moc_r.depth_max());
    let ranges_l = moc_l.into_moc_ranges();
    let ranges_r = moc_r.into_moc_ranges();
    RangeMOC::new(depth, ranges_l.union(&ranges_r))
}

// we could also perform the operation without having first collected the iteartor we obtain from
// the FITS file
fn test_or_ranges_it(moc_l: RangeMOC<u32, Hpx<u32>>, moc_r: RangeMOC<u32, Hpx<u32>>) -> RangeMOC<u32, Hpx<u32>> {
    let or = or(moc_l.into_range_moc_iter(), moc_r.into_range_moc_iter());
    RangeMOC::new(or.depth_max(), or.collect())
}

fn test_or_ranges_it_ref(moc_l: RangeMOC<u32, Hpx<u32>>, moc_r: RangeMOC<u32, Hpx<u32>>) -> RangeMOC<u32, Hpx<u32>> {
    let or = or((&moc_l).into_range_moc_iter(), (&moc_r).into_range_moc_iter());
    RangeMOC::new(or.depth_max(), or.collect())
}


fn bench_or(c: &mut Criterion) {
    // https://bheisler.github.io/criterion.rs/book/user_guide/comparing_functions.html
    let mut group = c.benchmark_group("or");
    let (sdss, other) = load_mocs();
    group.bench_function("Ranges UNION",
                         |b| b.iter(|| test_or_ranges(sdss.clone(), other.clone())));
    group.bench_function("Ranges Iter OR",
                         |b| b.iter(|| test_or_ranges_it(sdss.clone(), other.clone())));
    group.bench_function("Ranges Ref Iter OR",
                         |b| b.iter(|| test_or_ranges_it_ref(sdss.clone(), other.clone())));
    /*group.bench_function("Ranges 2 UNION",
                         |b| b.iter(|| test_or_ranges(sdss.clone(), other.clone())));
    group.bench_function("Ranges Iter 2 OR",
                         |b| b.iter(|| test_or_ranges_it(sdss.clone(), other.clone())));
    group.bench_function("Ranges Ref Iter 2 OR",
                         |b| b.iter(|| test_or_ranges_it_ref(sdss.clone(), other.clone())));*/
}

criterion_group!(benches, bench_or);
criterion_main!(benches);
