use solana_program::{blake3::HASH_BYTES, hash::Hash};

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

// If 2 hashes -> 1 hash
//    size_hash_2 = 0

// If recent_hash_1 is no longer a valid recent_hash
//    copy all the hashes in the second region to the first region

// The provided hash is assumed to use a valid recent_hash

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


struct HashStorage {
    capacity: u32,
    size_hash_1: u32,
    size_hash_2: u32,
    recent_hash_1: Hash,
    recent_hash_2: Hash,
    hashes: Vec<Hash>,
}

impl TryFrom<&mut [u8]> for HashStorage {
    type Error = ();

    fn try_from(data: &mut [u8]) -> Result<Self, Self::Error> {
        if data.len() % HASH_BYTES != 0 {
            Err(())
        } else {
            Ok(HashStorage {
                capacity: u32::from_be_bytes(data[0..4].try_into().expect("correct size")),
                size_hash_1: u32::from_be_bytes(data[4..8].try_into().expect("correct size")),
                size_hash_2: u32::from_be_bytes(data[8..12].try_into().expect("correct size")),
                recent_hash_1: Hash::new_from_array(data[32..64].try_into().expect("correct size")),
                recent_hash_2: Hash::new_from_array(data[64..96].try_into().expect("correct size")),
                hashes: data[96..]
                    .chunks_exact(32)
                    .map(|chunk| Hash::new_from_array(chunk.try_into().expect("correct_size")))
                    .collect(),
            })
        }
    }
}

impl Into<Vec<u8>> for HashStorage {
    fn into(self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.capacity as usize * 32 + 96);
        v.extend(self.capacity.to_be_bytes());
        v.extend(self.size_hash_1.to_be_bytes());
        v.extend(self.size_hash_2.to_be_bytes());
        v.extend([0, 0, 0, 0]);
        v.extend([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        v.extend(self.recent_hash_1.to_bytes());
        v.extend(self.recent_hash_2.to_bytes());
        v.extend(self.hashes.into_iter().flat_map(|hash| hash.to_bytes()));
        v
    }
}

impl HashStorage {}
