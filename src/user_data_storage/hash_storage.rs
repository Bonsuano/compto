use solana_program::{hash::Hash, hash::HASH_BYTES, program_error::ProgramError};

#[repr(C)]
pub struct HashStorageBase<T: ?Sized> {
    length: usize,
    blockhash: Hash,
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
    pub fn insert(&mut self, new_proof: &Hash, new_blockhash: &Hash) {
        // new_proof and new_blockhash have already been verified
        if self.blockhash != *new_blockhash {
            self.blockhash = *new_blockhash;
            self.length = 0;
        }
        assert!(!self.contains(new_proof), "proof should be new");

        eprintln!("length is : {}", self.length);
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

#[cfg(test)]
mod test {

    use super::*;
    use hex_literal::hex;
    use std::cmp::max;

    #[derive(Debug)]
    struct ProofOfWork {
        blockhash: Hash,
        proof: Hash,
    }

    #[derive(Debug)]
    struct TestValuesInput<'a> {
        data: &'a mut [u8],
        length: usize,
        stored_blockhash: Hash,
        proofs: &'a [Hash],
        new_proofs: &'a [ProofOfWork],
    }

    #[derive(Debug)]
    struct TestValuesOutput<'a> {
        length: usize,
        stored_blockhash: Hash,
        proofs: &'a [Hash],
    }

    struct TestValues<'a> {
        input: TestValuesInput<'a>,
        output: Option<TestValuesOutput<'a>>,
    }

    const POSSIBLE_BLOCKHASHES: [Hash; 2] = [
        Hash::new_from_array(hex!(
            "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"
        )),
        Hash::new_from_array(hex!(
            "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b"
        )),
    ];

    const POSSIBLE_PROOFS: [Hash; 2] = [
        Hash::new_from_array(hex!(
            "4e07408562bedb8b60ce05c1decfe3ad16b72230967de01f640b7e4729b49fce"
        )),
        Hash::new_from_array(hex!(
            "4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a"
        )),
    ];

    fn run_test(test_values: TestValues) {
        let input = test_values.input;
        let output = test_values.output;
        let max_length = max(input.length, output.as_ref().map_or(0, |o| o.length));
        assert!(
            input.data.len() >= METADATA_LEN + max_length * HASH_BYTES,
            "input data len is not large enough for the test"
        );

        let hash_storage: &mut HashStorage =
            input.data.try_into().expect("paniced already if failed");

        hash_storage.length = input.length;
        hash_storage.blockhash = input.stored_blockhash;
        hash_storage
            .into_iter()
            .zip(input.proofs)
            .for_each(|(hs_p, p)| *hs_p = *p);

        for pow in input.new_proofs {
            hash_storage.insert(&pow.proof, &pow.blockhash);
        }

        let hash_storage: &HashStorage = &hash_storage;
        let output = output.expect("paniced already if not Some");

        assert_eq!(
            hash_storage.length, output.length,
            "hash_storage is the correct length"
        );
        assert_eq!(
            hash_storage.blockhash, output.stored_blockhash,
            "hash_storage has the correct blockhash stored"
        );
        hash_storage
            .into_iter()
            .zip(output.proofs)
            .for_each(|(proof, expected_proof)| assert_eq!(proof, expected_proof))
    }

    #[test]
    fn test_try_from() {
        run_test(TestValues {
            input: TestValuesInput {
                data: &mut [0_u8; METADATA_LEN + 1 * HASH_BYTES],
                length: 0,
                stored_blockhash: POSSIBLE_BLOCKHASHES[0],
                proofs: &[],
                new_proofs: &[],
            },
            output: Some(TestValuesOutput {
                length: 0,
                stored_blockhash: POSSIBLE_BLOCKHASHES[0],
                proofs: &[],
            }),
        })
    }

    #[test]
    fn test_insert() {
        run_test(TestValues {
            input: TestValuesInput {
                data: &mut [0_u8; METADATA_LEN + 1 * HASH_BYTES],
                length: 0,
                stored_blockhash: POSSIBLE_BLOCKHASHES[0],
                proofs: &[],
                new_proofs: &[ProofOfWork {
                    blockhash: POSSIBLE_BLOCKHASHES[0],
                    proof: POSSIBLE_PROOFS[0],
                }],
            },
            output: Some(TestValuesOutput {
                length: 1,
                stored_blockhash: POSSIBLE_BLOCKHASHES[0],
                proofs: &[POSSIBLE_PROOFS[0]],
            }),
        })
    }

    #[test]
    fn test_insert_new() {
        run_test(TestValues {
            input: TestValuesInput {
                data: &mut [0_u8; METADATA_LEN + 1 * HASH_BYTES],
                length: 1,
                stored_blockhash: POSSIBLE_BLOCKHASHES[0],
                proofs: &[POSSIBLE_PROOFS[0]],
                new_proofs: &[ProofOfWork {
                    blockhash: POSSIBLE_BLOCKHASHES[1],
                    proof: POSSIBLE_PROOFS[1],
                }],
            },
            output: Some(TestValuesOutput {
                length: 1,
                stored_blockhash: POSSIBLE_BLOCKHASHES[1],
                proofs: &[POSSIBLE_PROOFS[1]],
            }),
        })
    }

    #[test]
    #[should_panic(expected = "proof should be new")]
    fn test_insert_duplicate() {
        run_test(TestValues {
            input: TestValuesInput {
                data: &mut [0_u8; METADATA_LEN + 2 * HASH_BYTES],
                length: 1,
                stored_blockhash: POSSIBLE_BLOCKHASHES[0],
                proofs: &[POSSIBLE_PROOFS[0]],
                new_proofs: &[ProofOfWork {
                    blockhash: POSSIBLE_BLOCKHASHES[0],
                    proof: POSSIBLE_PROOFS[0],
                }],
            },
            output: None,
        })
    }
}
