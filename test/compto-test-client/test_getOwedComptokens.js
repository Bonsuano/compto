import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import { Clock, start } from "solana-bankrun";

import {
    BIG_NUMBER,
    get_default_comptoken_mint, get_default_comptoken_wallet, get_default_global_data, get_default_unpaid_interest_bank,
    get_default_unpaid_ubi_bank, get_default_user_data_account, programId,
    TokenAccount,
    UserDataAccount
} from "./accounts.js";
import { Assert } from "./assert.js";
import { DEFAULT_ANNOUNCE_TIME, DEFAULT_DISTRIBUTION_TIME, DEFAULT_START_TIME, Instruction, SEC_PER_DAY, testuser_comptoken_wallet_pubkey } from "./common.js";

async function test_getOwedComptokens() {
    let comptoken_mint = get_default_comptoken_mint();
    comptoken_mint.supply = 292_004n
    let user_wallet = get_default_comptoken_wallet(testuser_comptoken_wallet_pubkey, PublicKey.unique());
    user_wallet.amount = 2;
    let user_data_account_address = PublicKey.findProgramAddressSync([user_wallet.address.toBytes()], programId)[0];
    let user_data = get_default_user_data_account(user_data_account_address);
    user_data.lastInterestPayoutDate = DEFAULT_DISTRIBUTION_TIME - SEC_PER_DAY;
    let global_data = get_default_global_data();
    global_data.dailyDistributionData.historicInterests[0] = 0.5;
    global_data.dailyDistributionData.oldestInterest = 1n;
    global_data.dailyDistributionData.yesterdaySupply = 292_004n;
    let interest_bank = get_default_unpaid_interest_bank();
    interest_bank.amount = 146_000n;
    let ubi_bank = get_default_unpaid_ubi_bank();
    ubi_bank.amount = 146_000n;

    const context = await start(
        [{ name: "comptoken", programId }],
        [
            user_data.toAccount(),
            user_wallet.toAccount(),
            comptoken_mint.toAccount(),
            global_data.toAccount(),
            interest_bank.toAccount(),
            ubi_bank.toAccount(),
        ]
    );

    const client = context.banksClient;
    const payer = context.payer;
    const blockhash = context.lastBlockhash;
    const rent = await client.getRent();
    const keys = [
        //  User's Data Account
        { pubkey: user_data.address, isSigner: false, isWritable: true },
        //  User's Comptoken Wallet
        { pubkey: user_wallet.address, isSigner: false, isWritable: true },
        //  Comptoken Mint
        { pubkey: comptoken_mint.address, isSigner: false, isWritable: false },
        //  Comptoken Global Data (also mint authority)
        { pubkey: global_data.address, isSigner: false, isWritable: false },
        //  Comptoken Interest Bank 
        { pubkey: interest_bank.address, isSigner: false, isWritable: true },
        //  Comptoken UBI Bank
        { pubkey: ubi_bank.address, isSigner: false, isWritable: true },
        //  Token 2022 Program
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
    ];

    let data = Buffer.from([Instruction.GET_OWED_COMPTOKENS]);

    const ixs = [new TransactionInstruction({ programId, keys, data })];
    const tx = new Transaction();
    tx.recentBlockhash = blockhash;
    tx.add(...ixs);
    tx.sign(payer);
    context.setClock(new Clock(0n, 0n, 0n, 0n, DEFAULT_START_TIME));
    const meta = await client.processTransaction(tx);

    let account = await client.getAccount(user_wallet.address);
    Assert.assertNotNull(account);
    let finalUserWallet = TokenAccount.fromAccountInfoBytes(user_wallet.address, account);
    Assert.assertEqual(finalUserWallet.amount, BigInt(user_wallet.amount + user_wallet.amount * global_data.dailyDistributionData.historicInterests[0]), "interest amount");

    account = await client.getAccount(user_data.address);
    Assert.assertNotNull(account);
    let finalUserData = UserDataAccount.fromAccountInfoBytes(user_data.address, account);
    Assert.assertEqual(finalUserData.lastInterestPayoutDate, DEFAULT_DISTRIBUTION_TIME, "last interest payout date updated");
}

(async () => { await test_getOwedComptokens(); })();