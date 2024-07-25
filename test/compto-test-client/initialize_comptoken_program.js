import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import {
    SystemProgram, SYSVAR_SLOT_HASHES_PUBKEY, Transaction, TransactionInstruction
} from "@solana/web3.js";
import { start } from "solana-bankrun";

import {
    get_default_comptoken_mint,
    programId
} from "./accounts.js";
import { Assert } from "./assert.js";
import {
    comptoken_mint_pubkey, global_data_account_pubkey, Instruction, interest_bank_account_pubkey, ubi_bank_account_pubkey,
} from "./common.js";

async function initialize_comptoken_program() {
    const context = await start(
        [{ name: "comptoken", programId }],
        [get_default_comptoken_mint().toAccount()]
    );

    const client = context.banksClient;
    const payer = context.payer;
    const blockhash = context.lastBlockhash;
    const rent = await client.getRent();
    const keys = [
        // the payer of the rent for the account
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
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

    // MAGIC NUMBER: CHANGE NEEDS TO BE REFLECTED IN comptoken.rs
    const globalDataRentExemptAmount = await rent.minimumBalance(4096n);
    const interestBankRentExemptAmount = await rent.minimumBalance(256n);
    const ubiBankRentExemptAmount = await rent.minimumBalance(256n);
    console.log("Rent exempt amount: ", globalDataRentExemptAmount);
    // 1 byte for instruction 3 x 8 bytes for rent exemptions
    let data = Buffer.alloc(25);
    data.writeUInt8(Instruction.INITIALIZE_STATIC_ACCOUNT, 0);
    data.writeBigInt64LE(globalDataRentExemptAmount, 1);
    data.writeBigInt64LE(interestBankRentExemptAmount, 9);
    data.writeBigInt64LE(ubiBankRentExemptAmount, 17);

    const ixs = [new TransactionInstruction({ programId, keys, data })];
    const tx = new Transaction();
    tx.recentBlockhash = blockhash;
    tx.add(...ixs);
    tx.sign(payer);
    const meta = await client.processTransaction(tx);
    Assert.assert(true); // so my IDE doesn't remove Assert for being unused
    // TODO: add asserts
}

(async () => { await initialize_comptoken_program(); })();
