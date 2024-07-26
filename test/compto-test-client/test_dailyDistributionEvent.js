import { SYSVAR_SLOT_HASHES_PUBKEY, Transaction, TransactionInstruction } from "@solana/web3.js";
import { Clock, start } from "solana-bankrun";

import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { get_default_comptoken_mint, get_default_global_data, get_default_unpaid_interest_bank, get_default_unpaid_ubi_bank, GlobalDataAccount, MintAccount, programId } from "./accounts.js";
import { Assert } from "./assert.js";
import { DEFAULT_START_TIME, Instruction, SEC_PER_DAY } from "./common.js";

async function test_dailyDistributionEvent() {
    let comptoken_mint = get_default_comptoken_mint();
    comptoken_mint.supply += 1n;
    let global_data = get_default_global_data();
    let interest_bank = get_default_unpaid_interest_bank();
    let ubi_bank = get_default_unpaid_ubi_bank();
    const context = await start(
        [{ name: "comptoken", programId }],
        [
            comptoken_mint.toAccount(),
            global_data.toAccount(),
            interest_bank.toAccount(),
            ubi_bank.toAccount(),
        ]
    );

    const client = context.banksClient;
    const payer = context.payer;
    const blockhash = context.lastBlockhash;
    const keys = [
        // so the token program knows what kind of token
        { pubkey: comptoken_mint.address, isSigner: false, isWritable: true },
        // stores information for/from the daily distribution
        { pubkey: global_data.address, isSigner: false, isWritable: true },
        // comptoken token account used as bank for unpaid interest
        { pubkey: interest_bank.address, isSigner: false, isWritable: true },
        // comptoken token account used as bank for unpaid Universal Basic Income
        { pubkey: ubi_bank.address, isSigner: false, isWritable: true },
        // the token program that will mint the tokens when instructed by the mint authority
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
        // program will pull a recent hash from slothashes sysvar if a new valid blockhash is needed.  
        { pubkey: SYSVAR_SLOT_HASHES_PUBKEY, isSigner: false, isWritable: false },
    ];

    let data = Buffer.from([Instruction.DAILY_DISTRIBUTION_EVENT])

    const ixs = [new TransactionInstruction({ programId, keys, data })];
    const tx = new Transaction();
    tx.recentBlockhash = blockhash;
    tx.add(...ixs);
    tx.sign(payer);
    context.setClock(new Clock(0n, 0n, 0n, 0n, DEFAULT_START_TIME));
    const result = await client.simulateTransaction(tx);

    // TODO: make this assert less brittle
    Assert.assert(result.meta.logMessages[3].includes("daily distribution already called today"), "daily distribution already called");

    let account = await client.getAccount(comptoken_mint.address);
    Assert.assertNotNull(account);
    const failMint = MintAccount.fromAccountInfoBytes(comptoken_mint.address, account);
    Assert.assertEqual(failMint.supply, comptoken_mint.supply, "interest has not been issued");

    context.setClock(new Clock(0n, 0n, 0n, 0n, DEFAULT_START_TIME + SEC_PER_DAY));
    const meta = await client.processTransaction(tx);
}

(async () => { await test_dailyDistributionEvent(); })();
