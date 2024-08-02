import { Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction, } from "@solana/web3.js";
import { Clock, start } from "solana-bankrun";

import { createInitializeTransferHookInstruction, initializeTransferHookInstructionData, TokenInstruction, TransferHookInstruction } from "@solana/spl-token";
import { get_default_comptoken_mint, get_default_global_data } from "../accounts.js";
import { Assert } from "../assert.js";
import { compto_transfer_hook_id_pubkey, DEFAULT_START_TIME, } from "../common.js";

async function test_initializeExtraAccountMetaList() {
    let comptoken_mint = get_default_comptoken_mint();
    const mint_authority = Keypair.generate();
    comptoken_mint.mintAuthority = mint_authority.publicKey;

    const context = await start(
        [{ name: "comptoken_transfer_hook", programId: compto_transfer_hook_id_pubkey }],
        [
            comptoken_mint.toAccount(),
        ]
    );

    const client = context.banksClient;
    const payer = context.payer;
    const blockhash = context.lastBlockhash;

    const [extraAccountMetaListPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("extra-account-metas"), comptoken_mint.address.toBuffer()],
        compto_transfer_hook_id_pubkey
    );

    const keys = [
        // the account that stores the extra account metas
        { pubkey: extraAccountMetaListPDA, isSigner: false, isWritable: true },
        // the mint account associated with the transfer hook
        { pubkey: comptoken_mint.address, isSigner: false, isWritable: true },
        // the mint authority for the mint
        { pubkey: mint_authority.publicKey, isSigner: true, isWritable: false },
        // system account is used to create the account
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        // the account who pays for the creation
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    ];

    let data = Buffer.from([43, 34, 13, 49, 167, 88, 235, 235, 0, 0, 0, 0]);

    const ixs = [new TransactionInstruction({ programId: compto_transfer_hook_id_pubkey, keys, data })];
    const tx = new Transaction();
    tx.recentBlockhash = blockhash;
    tx.add(...ixs);
    tx.sign(payer, mint_authority);
    context.setClock(new Clock(0n, 0n, 0n, 0n, DEFAULT_START_TIME));
    const meta = await client.processTransaction(tx);

    console.log("logMessages: %s", meta.logMessages);
    console.log("computeUnitsConsumed: %d", meta.computeUnitsConsumed);
    console.log("returnData: %s", meta.returnData);

    const finalMetaListAccount = await client.getAccount(extraAccountMetaListPDA);

    console.log(finalMetaListAccount);

}

(async () => { await test_initializeExtraAccountMetaList(); })();