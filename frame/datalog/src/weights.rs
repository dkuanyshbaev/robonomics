use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub trait WeightInfo {
    fn record() -> Weight;
    fn erase(win: u64) -> Weight;
}

#[allow(clippy::unnecessary_cast)]
impl WeightInfo for () {
    fn record() -> Weight {
        Weight::from_ref_time(1_000_000_u64)
            .saturating_add(DbWeight::get().reads(2_u64))
            .saturating_add(DbWeight::get().writes(3_u64))
    }

    fn erase(win: u64) -> Weight {
        Weight::from_ref_time(10_000_000_u64)
            .saturating_add(DbWeight::get().reads(1_u64))
            .saturating_add(DbWeight::get().writes(1_u64 + win))
    }
}
