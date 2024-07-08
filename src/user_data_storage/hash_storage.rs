use solana_program::{hash::Hash, hash::HASH_BYTES, program_error::ProgramError};

#[repr(C)]
pub struct HashStorageBase<T: ?Sized> {
    length: usize,
    recent_blockhash: Hash,
    data: T,
}

const METADATA_LEN: usize = std::mem::size_of::<HashStorageBase<()>>();

pub type HashStorage = HashStorageBase<[Hash]>;

impl TryFrom<&mut [u8]> for &mut HashStorage {
    type Error = ProgramError;

    fn try_from(data: &mut [u8]) -> Result<Self, Self::Error> {
        // TODO validity checks
        let new_len = (data.len() - METADATA_LEN) / HASH_BYTES;
        let data_hashes =
            unsafe { std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut Hash, new_len) };
        Ok(unsafe { &mut *(data_hashes as *mut _ as *mut HashStorage) })
    }
}

pub struct Iter<'a> {
    iter: std::iter::Take<std::slice::Iter<'a, Hash>>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Hash;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct MutIter<'a> {
    iter: std::iter::Take<std::slice::IterMut<'a, Hash>>,
}

impl<'a> Iterator for MutIter<'a> {
    type Item = &'a mut Hash;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> IntoIterator for &'a HashStorage {
    type Item = &'a Hash;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.data.into_iter().take(self.length),
        }
    }
}

impl<'a> IntoIterator for &'a mut HashStorage {
    type Item = &'a mut Hash;
    type IntoIter = MutIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MutIter {
            iter: self.data.iter_mut().take(self.length),
        }
    }
}

impl HashStorage {
    pub fn insert(self: &mut Self, new_proof: &Hash, recent_blockhash: &Hash) {
        // new_proof and recent_blockhash have already been verified
        if self.recent_blockhash != *recent_blockhash {
            self.recent_blockhash = *recent_blockhash;
            self.length = 0;
        }
        assert!(!self.contains(new_proof), "proof should be new");

        match self.data.get_mut(self.length) {
            Some(proof) => *proof = *new_proof,
            None => panic!("User Data Account not large enough, consider reallocing"),
        }
        self.length += 1;
    }

    fn contains(&self, new_proof: &Hash) -> bool {
        self.into_iter().any(|proof| proof == new_proof)
    }
}
