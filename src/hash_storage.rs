use std::cmp::min;

use solana_program::{
    account_info::AccountInfo,
    blake3::HASH_BYTES,
    entrypoint::{ProgramResult, MAX_PERMITTED_DATA_INCREASE},
    hash::Hash,
    program_error::ProgramError,
};

//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |          capacity: u32         |        size_hash_1: u32        |        size_hash_2: u32        |             padding            |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                               padding                                                             |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                                                                                                   |
//  +                                                         recent_hash_1: Hash                                                       +
//  |                                                                                                                                   |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                                                                                                   |
//  +                                                         recent_hash_2: Hash                                                       +
//  |                                                                                                                                   |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                                                                                                   |
//  +                                                           hash_1_1: Hash                                                          +
//  |                                                                                                                                   |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                                                                                                   |
//  +                                                                ...                                                                +
//  |                                                                                                                                   |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                                                                                                   |
//  +                                                     hash_1_<size_hash_1>: Hash                                                    +
//  |                                                                                                                                   |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                                                                                                   |
//  +                                                           hash_2_1: Hash                                                          +
//  |                                                                                                                                   |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                                                                                                   |
//  +                                                                ...                                                                +
//  |                                                                                                                                   |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                                                                                                   |
//  +                                                     hash_2_<size_hash_2>: Hash                                                    +
//  |                                                                                                                                   |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                                                                                                   |
//  +                                                         empty_hash_1: Hash                                                        +
//  |                                                                                                                                   |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                                                                                                   |
//  +                                                                ...                                                                +
//  |                                                                                                                                   |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+
//  |                                                                                                                                   |
//  +                                      empty_hash_<capacity - (size_hash_1 size_hash_2)>: Hash                                      +
//  |                                                                                                                                   |
//  +--------------------------------+--------------------------------+--------------------------------+--------------------------------+

// The purpose of this structure is to allow miners a small overlap of time,
// where they can either submit a hash with the old recent_hash or a hash with the new recent_hash, and both are considered valid.
// This is to prevent miners from doing useless work or having to spin down briefly once per day as the recent_hash changes.

// The provided hash is checked for validity

// If 2 hashes -> 1 hash
//    size_hash_2 = 0

// If recent_hash_1 is no longer a valid recent_hash
//    copy all the hashes in the second region to the first region

// If the provided hash matches recent_hash_2, then the hash is stored in the second region, at the end of size, increment size

// If the provided hash matches recent_hash_1, then
//      (1) the first hash in the second region is moved to the end of the second region, the size of the second region does not increment
//      (2) the hash is stored in the first region, at the end of size, increment size

// If the provided hash does not match recent_hash_1 or recent_hash_2, then this triggers the 1 hash -> 2 hashes transition
//     (1) recent_hash_1 is moved to overwrite recent_hash_2
//     (2) size_hash_1 is moved to overwrite size_hash_2
//     (3) the provided recent_hash is stored in recent_hash_1
//     (4) size_hash_1 is set to 0
//     (5) from here do the same as if the provided hash matches recent_hash_1

pub struct HashStorage {
    capacity: u32,
    size_blockhash_1: u32,
    size_blockhash_2: u32,
    _padding_1: [u8; 4],
    _padding_2: [u8; 16],
    recent_blockhash_1: Hash,
    recent_blockhash_2: Hash,
    hashes: [Hash],
}

impl<'a> TryFrom<&mut [u8]> for &mut HashStorage {
    type Error = ProgramError;

    fn try_from(data: &mut [u8]) -> Result<Self, Self::Error> {
        let capacity = u32::from_ne_bytes(data[0..4].try_into().expect("correct size"));
        let size_blockhash_1 = u32::from_ne_bytes(data[4..8].try_into().expect("correct size"));
        let size_blockhash_2 = u32::from_ne_bytes(data[8..12].try_into().expect("correct size"));
        // if data.len() != <sizeof HashStorage w/ capacity Hashes>
        if data.len() != 96 + (capacity as usize) * HASH_BYTES {
            return Err(ProgramError::InvalidAccountData);
        }
        if size_blockhash_1 + size_blockhash_2 > capacity {
            return Err(ProgramError::InvalidAccountData);
        }
        // Safety:
        //
        // capacity corresponds with length
        // size_blockhash_1 and size_blockhash_2 are within possible bounds
        // Hash's are valid with any bit pattern
        //Ok(unsafe { std::mem::transmute(data) })
        let new_len = (data.len() / 32) - 3;
        unsafe {
            let data_hashes =
                core::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut Hash, new_len);
            let result: &mut HashStorage = std::mem::transmute(data_hashes);
            Ok(result)
        }
    }
}

impl HashStorage {
    // may reallocate, which would invalidate `&mut self`, so takes `mut self: &mut Self`
    pub fn insert(
        mut self: &mut Self,
        recent_blockhash: &Hash,
        new_hash: Hash,
        data_account: &AccountInfo,
    ) -> ProgramResult {
        // The provided recent_blockhash is checked for validity
        let valid_hashes = get_valid_hashes();
        if !valid_hashes.contains(recent_blockhash) {
            return Err(ProgramError::InvalidInstructionData);
        }

        // If 2 blockhashes -> 1 blockhash
        //    size_blockhash_2 = 0
        if !valid_hashes.contains(&self.recent_blockhash_2) {
            self.size_blockhash_2 = 0;
        }

        // If recent_blockhash_1 is no longer a valid recent_hash
        //    copy all the hashes in the second region to the first region
        //      (some optimizations have been made to prevent unnecessary copies)
        if !valid_hashes.contains(&self.recent_blockhash_1) {
            for i in 0..min(self.size_blockhash_1, self.size_blockhash_2) {
                self.hashes[i as usize] =
                    self.hashes[(self.size_blockhash_1 + self.size_blockhash_2 - 1 - i) as usize];
            }
            self.size_blockhash_1 = self.size_blockhash_2;
        }

        // reallocate if necessary
        if self.capacity == self.size_blockhash_1 + self.size_blockhash_2 {
            self.realloc(data_account)?;
        }

        // If the provided hash matches recent_hash_2 then
        //      (1) the hash is checked against existing hashes
        //      (2) the hash is stored in the second region, at the end of size, increment size
        if *recent_blockhash == self.recent_blockhash_2 {
            if self.hashes[self.size_blockhash_2 as usize
                ..(self.size_blockhash_1 + self.size_blockhash_2) as usize]
                .iter()
                .any(|hash| *hash == new_hash)
            {
                return Err(ProgramError::InvalidInstructionData);
            }
            self.hashes[(self.size_blockhash_1 + self.size_blockhash_2) as usize] = new_hash;
            self.size_blockhash_2 += 1;

        // If the provided hash matches recent_hash_1, then
        //      (1) the hash is checked against existing hashes
        //      (2) the first hash in the second region is moved to the end of the second region,
        //          the size of the second region does not increment
        //      (3) the hash is stored in the first region, at the end of size, increment size
        } else if *recent_blockhash == self.recent_blockhash_1 {
            if self.hashes[0..self.size_blockhash_1 as usize]
                .iter()
                .any(|hash| *hash == new_hash)
            {
                return Err(ProgramError::InvalidInstructionData);
            }
            self.hashes[(self.size_blockhash_1 + self.size_blockhash_2) as usize] =
                self.hashes[self.size_blockhash_1 as usize];
            self.hashes[self.size_blockhash_1 as usize] = new_hash;
            self.size_blockhash_1 += 1;

        // If the provided hash does not match recent_hash_1 or recent_hash_2, then this triggers the
        // 1 hash -> 2 hashes transition
        //     (1) recent_hash_1 is moved to overwrite recent_hash_2
        //     (2) size_hash_1 is moved to overwrite size_hash_2
        //     (3) the provided recent_hash is stored in recent_hash_1
        //     (4) size_hash_1 is set to 0
        //     (5) from here do the same as if the provided hash matches recent_hash_1
        } else {
            self.recent_blockhash_2 = self.recent_blockhash_1;
            self.size_blockhash_2 = self.size_blockhash_1;
            self.recent_blockhash_1 = *recent_blockhash;
            self.size_blockhash_1 = 0;
            self.hashes[(self.size_blockhash_1 + self.size_blockhash_2) as usize] =
                self.hashes[self.size_blockhash_1 as usize];
            self.hashes[self.size_blockhash_1 as usize] = new_hash;
            self.size_blockhash_1 += 1;
        }
        Ok(())
    }

    // realloc invalidates `&mut self`, so it takes `&mut &mut self` in order to correct this
    fn realloc(self: &mut &mut Self, data_account: &AccountInfo) -> ProgramResult {
        let increase = min(
            (*self).capacity as usize * HASH_BYTES,
            MAX_PERMITTED_DATA_INCREASE,
        );
        let new_len = 96 + (*self).capacity as usize * HASH_BYTES + increase;
        (*self).capacity += (new_len / HASH_BYTES) as u32;
        data_account.realloc(new_len, false)?;

        unsafe {
            let self_ptr: *mut Self = *self;
            let data = core::slice::from_raw_parts_mut(
                self_ptr as *mut u8,
                (self.capacity * 32 + 96) as usize,
            );
            *self = data.try_into()?;
        }
        Ok(())
    }
}

enum ValidHashes {
    One(Hash),
    Two(Hash, Hash),
}

impl ValidHashes {
    fn contains(&self, hash: &Hash) -> bool {
        match self {
            Self::One(h) => h == hash,
            Self::Two(h1, h2) => h1 == hash || h2 == hash,
        }
    }
}

fn get_valid_hashes() -> ValidHashes {
    ValidHashes::One(Hash::new_from_array([0; 32]))
}
