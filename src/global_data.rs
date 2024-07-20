mod daily_distribution_data;

use spl_token_2022::{
    solana_program::{hash::Hash, slot_hashes::SlotHash},
    state::Mint,
};

use crate::{constants::*, get_current_time, normalize_time, VerifiedAccountInfo};
use daily_distribution_data::DailyDistributionData;

#[repr(C)]
#[derive(Debug)]
pub struct GlobalData {
    pub valid_blockhashes: ValidBlockhashes,
    pub daily_distribution_data: DailyDistributionData,
}

pub struct DailyDistributionValues {
    pub interest_distributed: u64,
    pub ubi_distributed: u64,
}

impl GlobalData {
    pub fn initialize(&mut self, slot_hash_account: &VerifiedAccountInfo) {
        self.valid_blockhashes.initialize(slot_hash_account);
        self.daily_distribution_data.initialize();
    }

    pub fn daily_distribution_event(
        &mut self, mint: Mint, slot_hash_account: &VerifiedAccountInfo,
    ) -> DailyDistributionValues {
        self.valid_blockhashes.update(slot_hash_account);
        self.daily_distribution_data.daily_distribution(mint)
    }
}

impl<'a> From<&VerifiedAccountInfo<'a>> for &'a mut GlobalData {
    fn from(account: &VerifiedAccountInfo) -> Self {
        let mut data = account.0.try_borrow_mut_data().unwrap();
        let result = unsafe { &mut *(data.as_mut() as *mut _ as *mut GlobalData) };
        result
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ValidBlockhashes {
    pub announced_blockhash: Hash,
    pub announced_blockhash_time: i64,
    pub valid_blockhash: Hash,
    pub valid_blockhash_time: i64,
}

impl ValidBlockhashes {
    fn initialize(&mut self, slot_hash_account: &VerifiedAccountInfo) {
        self.update(slot_hash_account);
    }

    pub fn update(&mut self, slot_hash_account: &VerifiedAccountInfo) {
        if self.is_announced_blockhash_stale() {
            self.announced_blockhash = get_most_recent_blockhash(slot_hash_account);
            // This is necessary for the case where a day's update has been "skipped"
            self.announced_blockhash_time =
                normalize_time(get_current_time() + ANNOUNCEMENT_INTERVAL) - ANNOUNCEMENT_INTERVAL;
        }
        if self.is_valid_blockhash_stale() {
            self.valid_blockhash = self.announced_blockhash;
            self.valid_blockhash_time = normalize_time(get_current_time());
        }
    }

    pub fn is_announced_blockhash_stale(&self) -> bool {
        get_current_time() > self.announced_blockhash_time + SEC_PER_DAY
    }

    pub fn is_valid_blockhash_stale(&self) -> bool {
        get_current_time() > self.valid_blockhash_time + SEC_PER_DAY
    }
}

fn get_most_recent_blockhash(slot_hash_account: &VerifiedAccountInfo) -> Hash {
    // slothashes is too large to deserialize with the normal methods
    // based on https://github.com/solana-labs/solana/issues/33015
    let data = slot_hash_account.0.try_borrow_data().unwrap();
    let len: usize = usize::from_ne_bytes(data[0..8].try_into().expect("correct size"));
    let slot_hashes: &[SlotHash] =
        unsafe { std::slice::from_raw_parts(data.as_ptr().offset(8) as *const SlotHash, len) };

    // get the hash from the most recent slot
    slot_hashes[0].1
}

// rust implements round_ties_even in version 1.77, which is more recent than
// the version (1.75) solana uses. this is a reimplementation, however rust's
// uses compiler intrinsics, so we can't just use their code
pub trait RoundEven {
    fn round_ties_even(self) -> Self;
}

impl RoundEven for f64 {
    fn round_ties_even(self) -> Self {
        let res = self.round();
        if (self - res).abs() == 0.5 && res % 2. != 0. {
            self.trunc()
        } else {
            res
        }
    }
}
