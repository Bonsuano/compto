import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import { Clock, start } from "solana-bankrun";

import { get_default_comptoken_mint, get_default_comptoken_wallet, get_default_global_data, get_default_user_data_account, programId } from "./accounts.js";
import { Assert } from "./assert.js";
import { DEFAULT_START_TIME, Instruction, testuser_comptoken_wallet_pubkey } from "./common.js";
import { ComptokenProof } from "./comptoken_proof.js";

async function test_proofSubmission() {
    let global_data_account = get_default_global_data();
    let mint_account = get_default_comptoken_mint();
    let destination_comptoken_wallet = get_default_comptoken_wallet(testuser_comptoken_wallet_pubkey, PublicKey.unique());
    const user_data_pda = PublicKey.findProgramAddressSync([destination_comptoken_wallet.address.toBytes()], programId)[0];
    let user_data_account = get_default_user_data_account(user_data_pda);

    const context = await start(
        [{ name: "comptoken", programId }],
        [
            mint_account.toAccount(),
            global_data_account.toAccount(),
            destination_comptoken_wallet.toAccount(),
            user_data_account.toAccount(),
        ]
    );

    const client = context.banksClient;
    const payer = context.payer;
    const blockhash = context.lastBlockhash;
    const rent = await client.getRent();
    const keys = [
        // will mint some comptokens
        { pubkey: mint_account.address, isSigner: false, isWritable: true },
        // will store minted comptoken
        { pubkey: destination_comptoken_wallet.address, isSigner: false, isWritable: true },
        // stores the current valid blockhashes
        { pubkey: global_data_account.address, isSigner: false, isWritable: false },
        // stores the proof to prevent duplicate submissions
        { pubkey: user_data_account.address, isSigner: false, isWritable: true },
        // for the actual minting
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
    ];

    let proof = new ComptokenProof(destination_comptoken_wallet.address, global_data_account.validBlockhashes.validBlockhash);
    proof.mine();
    let data = Buffer.concat([
        Buffer.from([Instruction.COMPTOKEN_MINT]),
        proof.serializeData(),
    ]);

    const ixs = [new TransactionInstruction({ programId, keys, data })];
    const tx = new Transaction();
    tx.recentBlockhash = blockhash;
    tx.add(...ixs);
    tx.sign(payer);
    context.setClock(new Clock(0n, 0n, 0n, 0n, DEFAULT_START_TIME));
    const meta = await client.processTransaction(tx);
    Assert.assert(true);
}

(async () => { await test_proofSubmission(); })();
