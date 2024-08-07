import {
    Connection,
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey,
    SYSVAR_SLOT_HASHES_PUBKEY,
    SystemProgram,
    Transaction,
    TransactionInstruction,
    sendAndConfirmTransaction
} from "@solana/web3.js";

import {
    AuthorityType,
    TOKEN_2022_PROGRAM_ID,
    setAuthority,
    unpackMint,
} from '@solana/spl-token';

import {
    Instruction,
    bs58,
    compto_program_id_pubkey,
    comptoken_mint_pubkey,
    global_data_account_pubkey,
    interest_bank_account_pubkey,
    me_keypair,
    testuser_comptoken_wallet_pubkey,
    ubi_bank_account_pubkey,
} from './common.js';

import base64 from "base64-js";

import { mintComptokens } from './comptoken_proof.js';


const testUser_keypair = Keypair.generate();

console.log("me: " + me_keypair.publicKey);
console.log("testuser comptoken wallet: " + testuser_comptoken_wallet_pubkey);
console.log("testuser: " + testUser_keypair.publicKey);
console.log("comptoken mint: " + comptoken_mint_pubkey);
console.log("compto program id: " + compto_program_id_pubkey);
console.log("global data account: " + global_data_account_pubkey);

let connection = new Connection('http://localhost:8899', 'recent');

(async () => {
    await airdrop(testUser_keypair.publicKey);
    await createGlobalDataAccount();
    await setMintAuthorityIfNeeded();
    await testMint();
    await createUserDataAccount();
    let current_block = (await getValidBlockHashes()).current_block;
    await mintComptokens(connection, testuser_comptoken_wallet_pubkey, testUser_keypair, current_block);
    await dailyDistributionEvent();
    await getOwedComptokens();
})();


async function airdrop(pubkey) {
    let airdropSignature = await connection.requestAirdrop(pubkey, 3 * LAMPORTS_PER_SOL,);
    await connection.confirmTransaction({ signature: airdropSignature });
    console.log("Airdrop confirmed");
}

async function setMintAuthorityIfNeeded() {
    const info = await connection.getAccountInfo(comptoken_mint_pubkey, "confirmed");
    const unpackedMint = unpackMint(comptoken_mint_pubkey, info, TOKEN_2022_PROGRAM_ID);
    if (unpackedMint.mintAuthority.toString() == global_data_account_pubkey.toString()) {
        console.log("Mint Authority already set, skipping setAuthority Transaction");
    } else {
        console.log("Mint Authority not set, setting Authority");
        await setMintAuthority(unpackedMint.mintAuthority);
    }
}

async function setMintAuthority(current_mint_authority_pubkey) {
    let me_signer = { publicKey: me_keypair.publicKey, secretKey: me_keypair.secretKey }
    let new_mint_authority = global_data_account_pubkey;
    const res = await setAuthority(
        connection,
        me_signer,
        comptoken_mint_pubkey,
        current_mint_authority_pubkey,
        AuthorityType.MintTokens,
        new_mint_authority,
        undefined,
        undefined,
        TOKEN_2022_PROGRAM_ID
    );
}

async function testMint() {
    let data = Buffer.from([Instruction.TEST]);
    let keys = [
        // communicates to the token program which mint (and therefore which mint authority)
        // to mint the tokens from
        { pubkey: comptoken_mint_pubkey, isSigner: false, isWritable: true },
        // the address to receive the test tokens
        { pubkey: testuser_comptoken_wallet_pubkey, isSigner: false, isWritable: true },
        // the mint authority that will sign to mint the tokens
        { pubkey: global_data_account_pubkey, isSigner: false, isWritable: false },
        // the token program that will mint the tokens when instructed by the mint authority
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
    ];
    let testMintTransaction = new Transaction();
    testMintTransaction.add(
        new TransactionInstruction({
            keys: keys,
            programId: compto_program_id_pubkey,
            data: data,
        }),
    );
    let testMintResult = await sendAndConfirmTransaction(connection, testMintTransaction, [testUser_keypair, testUser_keypair]);
    console.log("testMint transaction confirmed", testMintResult);
}

async function createGlobalDataAccount() {
    // MAGIC NUMBER: CHANGE NEEDS TO BE REFLECTED IN comptoken.rs
    const GLOBAL_DATA_SIZE = 3032;
    const globalDataRentExemptAmount = await connection.getMinimumBalanceForRentExemption(GLOBAL_DATA_SIZE);
    const interestBankRentExemptAmount = await connection.getMinimumBalanceForRentExemption(256);
    const ubiBankRentExemptAmount = await connection.getMinimumBalanceForRentExemption(256);
    console.log("Rent exempt amount: ", globalDataRentExemptAmount);
    // 1 byte for instruction 3 x 8 bytes for rent exemptions
    let data = Buffer.alloc(25);
    data.writeUInt8(Instruction.INITIALIZE_STATIC_ACCOUNT, 0);
    data.writeBigInt64LE(BigInt(globalDataRentExemptAmount), 1);
    data.writeBigInt64LE(BigInt(interestBankRentExemptAmount), 9);
    data.writeBigInt64LE(BigInt(ubiBankRentExemptAmount), 17);
    console.log("data: ", data);
    let keys = [
        // the payer of the rent for the account
        { pubkey: testUser_keypair.publicKey, isSigner: true, isWritable: true },
        // the address of the global data account to be created
        { pubkey: global_data_account_pubkey, isSigner: false, isWritable: true },
        // the address of the interest bank account to be created
        { pubkey: interest_bank_account_pubkey, isSigner: false, isWritable: true },
        // the address of the ubi bank account to be created
        { pubkey: ubi_bank_account_pubkey, isSigner: false, isWritable: true },
        // the comptoken mint account
        { pubkey: comptoken_mint_pubkey, isSigner: false, isWritable: false },
        // needed because compto program interacts with the system program to create the account
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        // the token program that will mint the tokens when instructed by the mint authority
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
        // program will pull a recent hash from slothashes sysvar if a new valid blockhash is needed.  
        { pubkey: SYSVAR_SLOT_HASHES_PUBKEY, isSigner: false, isWritable: false },
    ];
    let createGlobalDataAccountTransaction = new Transaction();
    createGlobalDataAccountTransaction.add(
        new TransactionInstruction({
            keys: keys,
            programId: compto_program_id_pubkey,
            data: data,
        }),
    );
    let createGlobalDataAccountResult = await sendAndConfirmTransaction(connection, createGlobalDataAccountTransaction, [testUser_keypair, testUser_keypair]);
    console.log("createGlobalDataAccount transaction confirmed", createGlobalDataAccountResult);
}

async function createUserDataAccount() {
    // MAGIC NUMBER: CHANGE NEEDS TO BE REFLECTED IN user_data.rs
    const PROOF_STORAGE_MIN_SIZE = 88;
    const rentExemptAmount = await connection.getMinimumBalanceForRentExemption(PROOF_STORAGE_MIN_SIZE);
    console.log("Rent exempt amount: ", rentExemptAmount);

    let user_data_account = PublicKey.findProgramAddressSync([testuser_comptoken_wallet_pubkey.toBytes()], compto_program_id_pubkey)[0];

    let createKeys = [
        // the payer of the rent for the account
        { pubkey: testUser_keypair.publicKey, isSigner: true, isWritable: true },
        // the data account tied to the comptoken wallet
        { pubkey: user_data_account, isSigner: false, isWritable: true },
        // the payers comptoken wallet (comptoken token acct)
        { pubkey: testuser_comptoken_wallet_pubkey, isSigner: false, isWritable: false },
        // system account is used to create the account
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ];
    // 1 byte for the instruction, 8 bytes for the rent exempt amount, 8 bytes for the proof storage min size
    let createData = Buffer.alloc(17);
    createData.writeUInt8(Instruction.CREATE_USER_DATA_ACCOUNT, 0);
    createData.writeBigInt64LE(BigInt(rentExemptAmount), 1);
    createData.writeBigInt64LE(BigInt(PROOF_STORAGE_MIN_SIZE), 9);
    console.log("createData: ", createData);
    let createUserDataAccountTransaction = new Transaction();
    createUserDataAccountTransaction.add(
        new TransactionInstruction({
            keys: createKeys,
            programId: compto_program_id_pubkey,
            data: createData,
        }),
    );
    let createUserDataAccountResult = await sendAndConfirmTransaction(connection, createUserDataAccountTransaction, [testUser_keypair]);
    console.log("createUserDataAccount transaction confirmed", createUserDataAccountResult);
}

async function dailyDistributionEvent() {
    let data = Buffer.alloc(1);
    data.writeUInt8(Instruction.DAILY_DISTRIBUTION_EVENT, 0);
    console.log("data: ", data);
    let keys = [
        // so the token program knows what kind of token
        { pubkey: comptoken_mint_pubkey, isSigner: false, isWritable: true },
        // stores information for/from the daily distribution
        { pubkey: global_data_account_pubkey, isSigner: false, isWritable: true },
        // comptoken token account used as bank for unpaid interest
        { pubkey: interest_bank_account_pubkey, isSigner: false, isWritable: true },
        // comptoken token account used as bank for unpaid Universal Basic Income
        { pubkey: ubi_bank_account_pubkey, isSigner: false, isWritable: true },
        // the token program that will mint the tokens when instructed by the mint authority
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
        // program will pull a recent hash from slothashes sysvar if a new valid blockhash is needed.  
        { pubkey: SYSVAR_SLOT_HASHES_PUBKEY, isSigner: false, isWritable: false },
    ];
    let dailyDistributionEventTransaction = new Transaction();
    dailyDistributionEventTransaction.add(
        new TransactionInstruction({
            keys: keys,
            programId: compto_program_id_pubkey,
            data: data,
        }),
    );
    let dailyDistributionEventResult = await sendAndConfirmTransaction(connection, dailyDistributionEventTransaction, [testUser_keypair, testUser_keypair]);
    console.log("DailyDistributionEvent transaction confirmed", dailyDistributionEventResult);

}

async function getValidBlockHashes() {
    let data = Buffer.alloc(1);
    data.writeUInt8(Instruction.GET_VALID_BLOCKHASHES, 0);
    console.log("data: ", data);
    let keys = [
        // stores valid blockhashes, but may be out of date
        { pubkey: global_data_account_pubkey, isSigner: false, isWritable: true },
        // program will pull a recent hash from slothashes sysvar if a new valid blockhash is needed.  
        { pubkey: SYSVAR_SLOT_HASHES_PUBKEY, isSigner: false, isWritable: false },
    ];
    let getValidBlockhashesTransaction = new Transaction();
    getValidBlockhashesTransaction.add(
        new TransactionInstruction({
            keys: keys,
            programId: compto_program_id_pubkey,
            data: data,
        }),
    );
    let getValidBlockhashesResult = await sendAndConfirmTransaction(connection, getValidBlockhashesTransaction, [testUser_keypair, testUser_keypair]);
    console.log("getValidBlockhashes transaction confirmed", getValidBlockhashesResult);
    let result = await waitForTransactionConfirmation(getValidBlockhashesResult);
    let resultData = result.meta.returnData.data[0];
    let resultBytes = base64.toByteArray(resultData);
    let currentBlockB58 = bs58.encode(resultBytes.slice(0, 32));
    let announcedBlockB58 = bs58.encode(resultBytes.slice(32, 64));
    let validBlockHashes = { current_block: currentBlockB58, announced_block: announcedBlockB58, };
    console.log("Valid Block Hashes: ", validBlockHashes);
    return validBlockHashes;
}

async function getOwedComptokens() {
    let data = Buffer.alloc(1);
    data.writeUInt8(Instruction.GET_OWED_COMPTOKENS, 0);
    console.log("data: ", data);

    let user_data_account = PublicKey.findProgramAddressSync([testuser_comptoken_wallet_pubkey.toBytes()], compto_program_id_pubkey)[0];

    let keys = [
        //  User's Data Account
        { pubkey: user_data_account, isSigner: false, isWritable: true },
        //  User's Comptoken Wallet
        { pubkey: testuser_comptoken_wallet_pubkey, isSigner: false, isWritable: true },
        //  Comptoken Mint
        { pubkey: comptoken_mint_pubkey, isSigner: false, isWritable: false },
        //  Comptoken Global Data (also mint authority)
        { pubkey: global_data_account_pubkey, isSigner: false, isWritable: false },
        //  Comptoken Interest Bank 
        { pubkey: interest_bank_account_pubkey, isSigner: false, isWritable: true },
        //  Comptoken UBI Bank
        { pubkey: ubi_bank_account_pubkey, isSigner: false, isWritable: true },
        //  Token 2022 Program
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
    ];
    let getValidBlockhashesTransaction = new Transaction();
    getValidBlockhashesTransaction.add(
        new TransactionInstruction({
            keys: keys,
            programId: compto_program_id_pubkey,
            data: data,
        }),
    );
    let getValidBlockhashesResult = await sendAndConfirmTransaction(connection, getValidBlockhashesTransaction, [testUser_keypair, testUser_keypair]);
    console.log("getOwedComptokens transaction confirmed", getValidBlockhashesResult);
}

async function waitForTransactionConfirmation(signature) {
    let attempts = 0;
    let max_attempts = 10;
    while (attempts++ < max_attempts) {
        let result = await connection.getTransaction(signature, { commitment: 'confirmed', maxSupportedTransactionVersion: 0 })
        if (result !== null) {
            return result;
        }
    }
    throw new Error('Transaction not confirmed after ' + max_attempts + ' attempts');
}