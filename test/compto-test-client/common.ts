import {
    Keypair,
    PublicKey,
} from '@solana/web3.js';

const bs58 = require('bs58');

const Instruction = {
    TEST: 0,
    COMPTOKEN_MINT: 1,
    INITIALIZE_STATIC_ACCOUNT: 2
};

// Read Cache Files
let static_pda_str = require("../.cache/compto_static_pda.json")["address"];
let compto_token_id_str = require("../.cache/comptoken_id.json")["commandOutput"]["address"]
let compto_program_id_str = require("../.cache/compto_program_id.json")['programId'];
let test_account = require("../.cache/compto_test_account.json");

// Pubkeys
const destination_pubkey = Keypair.fromSecretKey(new Uint8Array(test_account)).publicKey;
const static_pda_pubkey = new PublicKey(bs58.decode(static_pda_str));
const comptoken_pubkey = new PublicKey(bs58.decode(compto_token_id_str));
const compto_program_id_pubkey = new PublicKey(bs58.decode(compto_program_id_str));