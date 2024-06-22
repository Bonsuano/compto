use sha2::{Digest, Sha256};

type TimeStampType = [u8; 32];
type Hash = [u8; 32];

const MAX_DATA_SIZE: usize = 256;

const MIN_NUM_ZEROED_BITS: u8 = 1; // TODO: replace with permanent value

fn to_be_bytes(n: u32) -> [u8; 4] {
    use std::mem::transmute;
    let bytes: [u8; 4] = unsafe { transmute(n.to_be()) };
    bytes
}

struct Block {
    previous_hash: Hash,
    timestamp: TimeStampType,
    data: [u8; MAX_DATA_SIZE],
    nonce: u32,
    hash: Hash
}

impl Block {
    fn new(previous_block: Self, data: [u8; MAX_DATA_SIZE]) -> Self {
        let timestamp: TimeStampType = [0; 32]; // some kind of now()
        let mut hasher = Sha256::new();
        hasher.update(&previous_block.hash);
        hasher.update(&timestamp);
        hasher.update(&data);
        hasher.update(to_be_bytes(0));
        let hash = hasher.finalize().into();
        Block{previous_hash: previous_block.hash, timestamp, data, nonce: 0, hash}
    }

    fn leading_zeroes(&self) -> u8 {
        let mut leading_zeroes: u8 = 0;
        let mut iter = self.hash.iter().map(|byte| byte.leading_zeros() as u8);
        while let Some(i) = iter.next() {
            leading_zeroes += i;
            if i != 8 {
                break;
            }
        }
        leading_zeroes
    }

    fn generate_hash(&self) -> Hash{
        let mut hasher = Sha256::new();
        hasher.update(&self.previous_hash);
        hasher.update(&self.timestamp);
        hasher.update(&self.data);
        hasher.update(&to_be_bytes(self.nonce));
        hasher.finalize().into()
    }

    fn mine(&mut self) {
        while self.leading_zeroes() < MIN_NUM_ZEROED_BITS {
            self.nonce += 1;
            self.hash = self.generate_hash();
        }
    }
}