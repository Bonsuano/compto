use solana_program::{hash::Hash, hash::HASH_BYTES, program_error::ProgramError};

pub struct HashStorageMetaData {
    capacity: usize,
    length: usize,
    _padding: [u8; 16],
}

const _: () = assert!(std::mem::size_of::<HashStorageMetaData>() % HASH_BYTES == 0);

pub struct HashStorage {
    pub meta_data: HashStorageMetaData,
    data: [Hash],
}

impl TryFrom<&mut [u8]> for &mut HashStorage {
    type Error = ProgramError;

    fn try_from(value: &mut [u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl HashStorage {
    pub fn insert(self: &mut Self, proof: Hash, recent_blockhash: &Hash, valid_blockhash: &Hash) {
        todo!()
    }
}
