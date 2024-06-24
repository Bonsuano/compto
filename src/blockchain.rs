use std::mem;

use solana_program::{
    clock::UnixTimestamp, hash::{Hash, Hasher, HASH_BYTES}, pubkey::{Pubkey, PUBKEY_BYTES}
};

// i64 is the type of Solana's GetBlockTime RPC result
type TimeStampType = UnixTimestamp; 

const MIN_NUM_ZEROED_BITS: u32 = 1; // TODO: replace with permanent value

// will need to be converted to a data account
static RECENT_BLOCKHASHES: [Hash; 4] = [unsafe { std::mem::transmute([0u8; 32]) }; 4];

fn check_if_recent_timestamp(time: &TimeStampType) -> bool {
    true
}

fn check_if_recent_blockhashes(hash: &Hash) -> bool {
    RECENT_BLOCKHASHES.contains(&hash)
}

pub fn verify_proof(block: Block) -> bool {
    Block::leading_zeroes(&block.hash) >= MIN_NUM_ZEROED_BITS
        && check_if_recent_blockhashes(&block.recent_block_hash)
        && check_if_recent_timestamp(&block.timestamp)
        && block.generate_hash() == block.hash
}
pub struct Block {
    pubkey: Pubkey,
    recent_block_hash: Hash,
    timestamp: TimeStampType,
    nonce: u32,
    hash: Hash,
}

impl Block {
    const PUBLIC_KEY_SIZE: usize = PUBKEY_BYTES;
    const VERIFY_DATA_SIZE: usize = mem::size_of::<Hash>() + mem::size_of::<TimeStampType>() + mem::size_of::<u32>() + mem::size_of::<Hash>();

    pub fn from_bytes(key: Pubkey, bytes: [u8; Self::VERIFY_DATA_SIZE]) -> Self {
        let range_1 = 0..mem::size_of::<Hash>();
        let range_2 = range_1.end..range_1.end + mem::size_of::<TimeStampType>();
        let range_3 = range_2.end..range_2.end + mem::size_of::<u32>();
        let range_4 = range_3.end..range_3.end + mem::size_of::<Hash>();
        
        let recent_block_hash = Hash::new_from_array(bytes[range_1].try_into().unwrap());
        let timestamp = TimeStampType::from_be_bytes(bytes[range_2].try_into().unwrap());
        let nonce = u32::from_be_bytes(
            bytes[range_3]
                .try_into()
                .unwrap(),
        );
        let hash = Hash::new_from_array(bytes[range_4].try_into().unwrap());

        Block {
            pubkey: key,
            recent_block_hash,
            timestamp,
            nonce,
            hash,
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
        hasher.hash(&self.pubkey.to_bytes());
        hasher.hash(&self.recent_block_hash.to_bytes());
        hasher.hash(&self.timestamp.to_be_bytes());
        hasher.hash(&self.nonce.to_be_bytes());
        hasher.result()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_arbitrary_block(
        recent_block_hash: Hash,
        timestamp: TimeStampType,
        pubkey: Pubkey,
        nonce: u32,
        hash: Hash
    ) -> Block {
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
        create_arbitrary_block(
            Hash::new_from_array([0; 32]),
            0,
            Pubkey::new_from_array([0; Block::PUBLIC_KEY_SIZE]),
            0,
            Hash::new_from_array([0; 32]),
        )
    }

    #[test]
    fn test_leading_zeroes() {
        let mut hash_array = [0; 32];
        let mut hash = Hash::new_from_array(hash_array);
        assert_eq!(256, Block::leading_zeroes(&hash));

        hash_array[0] = 0b1000_0000;
        hash = Hash::new_from_array(hash_array);
        assert_eq!(0, Block::leading_zeroes(&hash));

        hash_array[0] = 0b0000_1000;
        hash = Hash::new_from_array(hash_array);
        assert_eq!(4, Block::leading_zeroes(&hash));
    }

    #[test]
    fn test_from_bytes() {
        assert_eq!(
            Block::from_bytes(
                Pubkey::new_from_array([0; Block::PUBLIC_KEY_SIZE]),
                [0; Block::VERIFY_DATA_SIZE]
            )
            .hash,
            [0; 32].into()
        );

        let recent_hash = Hash::new_from_array([1; 32]);
        let timestamp: i64 = 0x02020202_02020202;
        let pubkey = Pubkey::new_from_array([3; Block::PUBLIC_KEY_SIZE]);
        let nonce: u32 = 0x04040404;
        let mut v = Vec::<u8>::with_capacity(Block::VERIFY_DATA_SIZE);
        let mut hasher = Hasher::default();

        hasher.hash(&pubkey.to_bytes());
        v.extend(recent_hash.to_bytes());
        v.extend(timestamp.to_be_bytes());
        v.extend(nonce.to_be_bytes());
        hasher.hash(&v);
        let hash = hasher.result();
        v.extend(hash.to_bytes());

        let block_from_bytes = Block::from_bytes(pubkey, v.try_into().unwrap());
        let block_from_data = create_arbitrary_block(recent_hash, timestamp, pubkey, nonce, hash);
        assert_eq!(
            block_from_bytes.recent_block_hash,
            block_from_data.recent_block_hash,
            "recent_block_hashes are different"
        );
        assert_eq!(block_from_bytes.timestamp, block_from_data.timestamp, "timestampss are different");
        assert_eq!(block_from_bytes.pubkey, block_from_data.pubkey, "pubkeys are different");
        assert_eq!(block_from_bytes.nonce, block_from_data.nonce, "nonces are different");
        assert_eq!(block_from_bytes.hash, block_from_data.hash, "hashes are different");
    }
}
