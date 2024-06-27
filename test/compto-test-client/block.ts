import {
    Blockhash,
    Connection,
    Keypair,
    PublicKey,
    SYSVAR_SLOT_HASHES_PUBKEY,
    Transaction,
    TransactionInstruction,
    sendAndConfirmTransaction,
} from '@solana/web3.js';

//import bs58 from "bs58";
const bs58 = require("bs58");

import { assert } from "console";

import crypto from "crypto";

const {
    Instruction,
} = require('common.ts');

type Integer = Number;

const MIN_NUM_ZEROED_BITS: Integer = 1; // TODO: replace with permanent value

class Block {
    pubkey: PublicKey
    recentBlockHash: Blockhash
    nonce: Buffer // uint_64
    hash: Blockhash

    constructor(pubkey: PublicKey, recentBlockHash: Blockhash) {
        this.pubkey = pubkey;
        this.recentBlockHash = recentBlockHash;
        this.nonce = Buffer.alloc(8);
        this.nonce.writeUInt32BE(crypto.randomInt(2 ** 32));
        this.hash = this.generateHash();
    }

    generateHash(): Blockhash {
        let hasher = crypto.createHash("sha256");
        hasher.update(this.pubkey.toBuffer());
        hasher.update(bs58.decode(this.recentBlockHash));
        hasher.update(this.nonce);
        return bs58.encode(hasher.digest());
    }

    static leadingZeroes(hash: Blockhash): Integer {
        let leadingZeroes = 0;
        let iter = bs58.decode(hash).map((byte: Number) => 8 - byte.toString().replace(/^0*/, "").length);
        for (let i = 0; i < iter.length; ++i) {
            leadingZeroes += iter[i];
            if (iter[i] != 8) {
                break;
            }
        }
        return leadingZeroes;
    }

    mine() {
        while (Block.leadingZeroes(this.hash) <= MIN_NUM_ZEROED_BITS) {
            this.nonce.writeUInt32BE(this.nonce.readUInt32BE() + 1);
            this.hash = this.generateHash();
        }
    }

    serializeData() {
        // ensure this remains consistent with mintblock.rs
        let buffer = Buffer.concat([
            bs58.decode(this.recentBlockHash),
            this.nonce,
            bs58.decode(this.hash),
        ]);
        assert(buffer.length == 72);
        return buffer;
    }
}

// under construction
async function mintComptokens(connection: Connection, destination_pubkey: PublicKey, compto_program_id_pubkey: PublicKey, temp_keypair: Keypair) {
    let data = Buffer.from([Instruction.COMPTOKEN_MINT]);
    let keys = [{ pubkey: SYSVAR_SLOT_HASHES_PUBKEY, isSigner: false, isWritable: false },
    { pubkey: destination_pubkey, isSigner: false, isWritable: true },];
    let mintComptokensTransaction = new Transaction();
    mintComptokensTransaction.add(
        new TransactionInstruction({
            keys: keys,
            programId: compto_program_id_pubkey,
            data: data,
        }),
    );
    let mintComptokensResult = await sendAndConfirmTransaction(connection, mintComptokensTransaction, [temp_keypair, temp_keypair]);
    console.log("mintComptokens transaction confirmed", mintComptokensResult);
}

//================TEST STUFF=============================

//function decimalToHex(d: number, padding: number = 0): string {
//    var hex = d.toString(16);

//    while (hex.length < padding) {
//        hex = "0" + hex;
//    }

//    return hex;
//}

//function bufferBasicString(buffer: Buffer): string {
//    let s = ""
//    let i = 0
//    for (; i < buffer.length - 1; ++i) {
//        let b = buffer[i];
//        s += decimalToHex(buffer[i], 2) + ", ";
//    }
//    s += decimalToHex(buffer[i], 2);
//    return s;
//}

//function bufferToString(buffer: Buffer): string {
//    return "{\n\t" + bufferBasicString(buffer) + "\n}";
//}

//function test() {
//    let block = new Block(new PublicKey(0), '11111111111111111111111111111111');
//    console.log(block.pubkey)
//    console.log("recentBlockHash: " + bufferToString(bs58.decode(block.recentBlockHash)));
//    console.log("nonce: " + bufferToString(block.nonce));
//    console.log("hash: " + bufferToString(bs58.decode(block.hash)));
//    let buffer = block.serializeData();
//    console.log("block data {");
//    console.log("\t" + bufferBasicString(buffer.slice(0, 32)) + ",");
//    console.log("\t" + bufferBasicString(buffer.slice(32, 40)) + ",");
//    console.log("\t" + bufferBasicString(buffer.slice(40, 72)));
//    console.log("}");

//}

//if (require.main === module) {
//    test();
//}