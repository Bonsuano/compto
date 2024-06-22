use solana_program::{
    hash::{Hash, Hasher, HASH_BYTES},
    pubkey,
};

type TimeStampType = [u8; 32];

const PUBLIC_KEY_SIZE: usize = pubkey::PUBKEY_BYTES;

const MIN_NUM_ZEROED_BITS: u32 = 1; // TODO: replace with permanent value

fn to_be_bytes(n: u32) -> [u8; 4] {
    use std::mem::transmute;
    let bytes: [u8; 4] = unsafe { transmute(n.to_be()) };
    bytes
}

// will need to be converted to a data account
static RECENT_BLOCKHASHES: [Hash; 4] = [unsafe { std::mem::transmute([0u8; 32]) }; 4];

fn check_if_recent_timestamp(time: &TimeStampType) -> bool {
    true
}

fn check_if_recent_blockhashes(hash: &Hash) -> bool {
    RECENT_BLOCKHASHES.contains(&hash)
}

fn verify_proof(block: Block) -> bool {
    Block::leading_zeroes(&block.hash) >= MIN_NUM_ZEROED_BITS
        && check_if_recent_blockhashes(&block.recent_block_hash)
        && check_if_recent_timestamp(&block.timestamp)
        && block.generate_hash() == block.hash
}
pub struct Block {
    recent_block_hash: Hash,
    timestamp: TimeStampType,
    pubkey: [u8; PUBLIC_KEY_SIZE],
    nonce: u32,
    hash: Hash,
}

impl Block {
    pub fn from_bytes(bytes: [u8; HASH_BYTES + 32 + PUBLIC_KEY_SIZE + 4 + HASH_BYTES]) -> Self {
        Block {
            recent_block_hash: Hash::new_from_array(bytes[0..HASH_BYTES].try_into().unwrap()),
            timestamp: bytes[HASH_BYTES..HASH_BYTES + 32].try_into().unwrap(),
            pubkey: bytes[HASH_BYTES + 32..HASH_BYTES + 32 + PUBLIC_KEY_SIZE]
                .try_into()
                .unwrap(),
            nonce: u32::from_be_bytes(
                bytes[HASH_BYTES + 32 + PUBLIC_KEY_SIZE..HASH_BYTES + 32 + PUBLIC_KEY_SIZE + 4]
                    .try_into()
                    .unwrap(),
            ),
            hash: Hash::new_from_array(
                bytes[HASH_BYTES + 32 + PUBLIC_KEY_SIZE + 4..]
                    .try_into()
                    .unwrap(),
            ),
        }
    }

    pub fn leading_zeroes(hash: &Hash) -> u32 {
        let mut leading_zeroes: u32 = 0;
        let mut iter = hash
            .to_bytes()
            .into_iter()
            .map(|byte| byte.leading_zeros() as u32);
        while let Some(i) = iter.next() {
            leading_zeroes += i;
            if i != 8 {
                break;
            }
        }
        leading_zeroes
    }

    pub fn generate_hash(&self) -> Hash {
        let mut hasher = Hasher::default();
        hasher.hash(&self.recent_block_hash.to_bytes());
        hasher.hash(&self.timestamp);
        hasher.hash(&self.pubkey);
        hasher.hash(&to_be_bytes(self.nonce));
        hasher.result()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_block(
        recent_block_hash: Hash,
        timestamp: TimeStampType,
        pubkey: [u8; PUBLIC_KEY_SIZE],
        nonce: u32,
    ) -> Block {
        let mut hasher = Hasher::default();
        hasher.hash(&recent_block_hash.to_bytes());
        hasher.hash(&timestamp);
        hasher.hash(&pubkey);
        hasher.hash(&to_be_bytes(nonce));
        let hash = hasher.result();
        Block {
            recent_block_hash,
            timestamp,
            pubkey,
            nonce,
            hash,
        }
    }

    fn mine(block: &mut Block) {
        while Block::leading_zeroes(&block.hash) < MIN_NUM_ZEROED_BITS {
            block.nonce += 1;
            block.hash = block.generate_hash();
        }
    }

    fn create_zero_block() -> Block {
        create_block(
            Hash::new_from_array([0; 32]),
            [0; 32],
            [0; PUBLIC_KEY_SIZE],
            0,
        )
    }

    #[test]
    fn test_leading_zeroes() {
        let mut hash_array = [0; 32];
        let hash = Hash::new_from_array(hash_array);
        assert_eq!(256, Block::leading_zeroes(&hash));

        hash_array[0] = 0b1000_0000;
        let hash = Hash::new_from_array(hash_array);
        assert_eq!(0, Block::leading_zeroes(&hash));

        hash_array[0] = 0b0000_1000;
        let hash = Hash::new_from_array(hash_array);
        assert_eq!(4, Block::leading_zeroes(&hash));
    }

    #[test]
    fn test_from_bytes() {
        assert_eq!(
            Block::from_bytes([0; HASH_BYTES + 32 + PUBLIC_KEY_SIZE + 4 + HASH_BYTES]).hash,
            [0; 32].into()
        );

        let recent_hash = Hash::new_from_array([1; 32]);
        let timestamp = [2; 32];
        let pubkey = [3; PUBLIC_KEY_SIZE];
        let nonce = 0x04040404;
        let mut v = Vec::<u8>::with_capacity(HASH_BYTES + 32 + PUBLIC_KEY_SIZE + 4 + HASH_BYTES);
        v.extend(recent_hash.to_bytes());
        v.extend(timestamp);
        v.extend(pubkey);
        v.extend(to_be_bytes(nonce));
        let mut hasher = Hasher::default();
        hasher.hash(&v);
        v.extend(hasher.result().to_bytes());
        let block_from_bytes = Block::from_bytes(v.try_into().unwrap());
        let block_from_data = create_block(recent_hash, timestamp, pubkey, nonce);
        assert_eq!(
            block_from_bytes.recent_block_hash,
            block_from_data.recent_block_hash
        );
        assert_eq!(block_from_bytes.timestamp, block_from_data.timestamp);
        assert_eq!(block_from_bytes.pubkey, block_from_data.pubkey);
        assert_eq!(block_from_bytes.nonce, block_from_data.nonce);
        assert_eq!(block_from_bytes.hash, block_from_data.hash);
        assert_eq!(
            block_from_bytes.hash.to_bytes(),
            hex::decode("e81bcc7f79b4610777eb637e0459bde955298013e742f3a6d44a8497683e486d")
                .unwrap()[..]
        );
    }
}
