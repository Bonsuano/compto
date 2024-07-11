import {
    Connection,
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey,
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
    compto_program_id_pubkey,
    comptoken_pubkey,
    destination_pubkey,
    global_data_pda_pubkey,
    me_keypair,
} from './common.js';

import { mintComptokens } from './comptoken_proof.js';


const temp_keypair = Keypair.generate();

console.log("me: " + me_keypair.publicKey);
console.log("destination: " + destination_pubkey);
console.log("tempkeypair: " + temp_keypair.publicKey);
console.log("compto_token: " + comptoken_pubkey);
console.log("compto_program_id: " + compto_program_id_pubkey);
console.log("global_data_pda: " + global_data_pda_pubkey);

let connection = new Connection('http://localhost:8899', 'recent');

(async () => {
    await airdrop(temp_keypair.publicKey);
    await setMintAuthorityIfNeeded();
    await createUserDataAccount();
    await testMint();
    await createGlobalDataAccount();
    await mintComptokens(connection, destination_pubkey, temp_keypair);
    // await dailyDistributionEvent();
})();


async function airdrop(pubkey) {
    let airdropSignature = await connection.requestAirdrop(pubkey, 3*LAMPORTS_PER_SOL,);
    await connection.confirmTransaction({ signature: airdropSignature });
    console.log("Airdrop confirmed");
}

async function setMintAuthorityIfNeeded() {
    const info = await connection.getAccountInfo(comptoken_pubkey, "confirmed");
    const unpackedMint = unpackMint(comptoken_pubkey, info, TOKEN_2022_PROGRAM_ID);
    if (unpackedMint.mintAuthority.toString() == global_data_pda_pubkey.toString()) {
        console.log("Mint Authority already set, skipping setAuthority Transaction");
    } else {
        console.log("Mint Authority not set, setting Authority");
        await setMintAuthority(unpackedMint.mintAuthority);
    }
}

async function setMintAuthority(mint_authority_pubkey) {
    let me_signer = { publicKey: me_keypair.publicKey, secretKey: me_keypair.secretKey }
    const res = await setAuthority(
        connection,
        me_signer,
        comptoken_pubkey,
        mint_authority_pubkey,
        AuthorityType.MintTokens,
        global_data_pda_pubkey,
        undefined,
        undefined,
        TOKEN_2022_PROGRAM_ID
    );
}

async function testMint() {
    let data = Buffer.from([Instruction.TEST]);
    let keys = [
        // the address to receive the test tokens
        { pubkey: destination_pubkey, isSigner: false, isWritable: true },
        // the mint authority that will sign to mint the tokens
        { pubkey: global_data_pda_pubkey, isSigner: false, isWritable: false},
        // the token program that will mint the tokens when instructed by the mint authority
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
        // communicates to the token program which mint (and therefore which mint authority)
        // to mint the tokens from
        { pubkey: comptoken_pubkey, isSigner: false, isWritable: true },
    ];
    let testMintTransaction = new Transaction();
    testMintTransaction.add(
        new TransactionInstruction({
            keys: keys,
            programId: compto_program_id_pubkey,
            data: data,
        }),
    );
    let testMintResult = await sendAndConfirmTransaction(connection, testMintTransaction, [temp_keypair, temp_keypair]);
    console.log("testMint transaction confirmed", testMintResult);
}

async function createGlobalDataAccount() {
    // MAGIC NUMBER: CHANGE NEEDS TO BE REFLECTED IN comptoken.rs
    const rentExemptAmount = await connection.getMinimumBalanceForRentExemption(4096);
    console.log("Rent exempt amount: ", rentExemptAmount);
    let data = Buffer.alloc(9);
    data.writeUInt8(Instruction.INITIALIZE_STATIC_ACCOUNT, 0);
    data.writeBigInt64BE(BigInt(rentExemptAmount), 1);
    console.log("data: ", data);
    let keys = [
        // the payer of the rent for the account
        { pubkey: temp_keypair.publicKey, isSigner: true, isWritable: true },
        // the address of the account to be created
        { pubkey: global_data_pda_pubkey, isSigner: false, isWritable: true},
        // needed because compto program interacts with the system program to create the account
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false}
    ];
    let createGlobalDataAccountTransaction = new Transaction();
    createGlobalDataAccountTransaction.add(
        new TransactionInstruction({
            keys: keys,
            programId: compto_program_id_pubkey,
            data: data,
        }),
    );
    let createGlobalDataAccountResult = await sendAndConfirmTransaction(connection, createGlobalDataAccountTransaction, [temp_keypair, temp_keypair]);
    console.log("createGlobalDataAccount transaction confirmed", createGlobalDataAccountResult);
}

async function createUserDataAccount() {
    // MAGIC NUMBER: CHANGE NEEDS TO BE REFLECTED IN proof_storage.rs
    const PROOF_STORAGE_MIN_SIZE = 72;
    const rentExemptAmount = await connection.getMinimumBalanceForRentExemption(PROOF_STORAGE_MIN_SIZE);
    console.log("Rent exempt amount: ", rentExemptAmount);

    let user_pda = PublicKey.findProgramAddressSync([destination_pubkey.toBytes()], compto_program_id_pubkey)[0];
    
    let createKeys = [
        // the payer of the rent for the account
        { pubkey: temp_keypair.publicKey, isSigner: true, isWritable: true },
        // the payers comptoken wallet (comptoken token acct)
        { pubkey: destination_pubkey, isSigner: false, isWritable: false },
        // the data account tied to the comptoken wallet
        { pubkey: user_pda, isSigner: false, isWritable: true },
        // system account is used to create the account
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
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
    let createUserDataAccountResult = await sendAndConfirmTransaction(connection, createUserDataAccountTransaction, [temp_keypair]);
    console.log("createUserDataAccount transaction confirmed", createUserDataAccountResult);
}

// TODO rename
async function dailyDistributionEvent() {
    // MAGIC NUMBER: CHANGE NEEDS TO BE REFLECTED IN comptoken.rs
    let data = Buffer.alloc(1);
    data.writeUInt8(Instruction.DAILY_DISTRIBUTION_EVENT, 0);
    console.log("data: ", data);
    let keys = [
        // the comptoken Mint
        { pubkey: comptoken_pubkey, isSigner: false, isWritable: false },
        // the Global Comptoken Data Account (also mint authority)
        { pubkey: global_data_pda_pubkey, isSigner: false, isWritable: true },
        // the Comptoken Interest Bank Account
        { pubkey: PublicKey.default, isSigner: false, isWritable: true }, // TODO get currect bank pubkey
    ];
    let dailyDistributionEventTransaction = new Transaction();
    dailyDistributionEventTransaction.add(
        new TransactionInstruction({
            keys: keys,
            programId: compto_program_id_pubkey,
            data: data,
        }),
    );
    let dailyDistributionEventResult = await sendAndConfirmTransaction(connection, dailyDistributionEventTransaction, [temp_keypair, temp_keypair]);
    console.log("DailyDistributionEvent transaction confirmed", dailyDistributionEventResult);
    
}