import json
import random
import subprocess
from pathlib import Path
from typing import Any

import base58

TEST_PATH = Path(__file__).parent
PROJECT_PATH = TEST_PATH.parent
CACHE_PATH = TEST_PATH / ".cache"
GENERATED_PATH = PROJECT_PATH / "src/generated"

COMPTO_PROGRAM_ID_JSON = CACHE_PATH / "compto_program_id.json"
COMPTO_SO = PROJECT_PATH / "target/deploy/comptoken.so"

COMPTOKEN_MINT_JSON = CACHE_PATH / "comptoken_mint.json"

TEST_USER_ACCOUNT_JSON = CACHE_PATH / "test_user_account.json"
COMPTO_GENERATED_RS_FILE = GENERATED_PATH / "comptoken_generated.rs"
COMPTO_GLOBAL_DATA_ACCOUNT_JSON = CACHE_PATH / "compto_global_data_account.json"
COMPTO_INTEREST_BANK_ACCOUNT_JSON = CACHE_PATH / "compto_interest_bank_account.json"
COMPTO_UBI_BANK_ACCOUNT_JSON = CACHE_PATH / "compto_ubi_bank_account.json"

class SubprocessFailedException(Exception):
    pass

class PDA(dict[str, Any]):

    def __init__(self, programId: str, *seeds: Any) -> None:
        seeds_str = ""
        for seed in seeds:
            if isinstance(seed, str):
                seeds_str += f"string:'{seed}'"
            elif isinstance(seed, int):
                seeds_str += f"hex:'{hex(seed)}'"
            elif isinstance(seed, bytes):
                seeds_str += f"pubkey:'{seed}'"
            else:
                raise TypeError(f"bad type: '{seed.__class__}'")
        super().__init__(json.loads(run(f"solana find-program-derived-address {programId} string:'{seeds_str}' --output json")))

def run(command: str | list[str], cwd: Path | None = None):
    result = subprocess.run(command, shell=True, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        raise SubprocessFailedException(
            f"Failed to run command! command: {command} stdout: {result.stdout} stderr: {result.stderr}"
        )
    return result.stdout.rstrip()

def writeProgramId(programId: str):
    write(COMPTO_PROGRAM_ID_JSON, json.dumps({"programId": programId}))

def writeGlobalDataPDA(programId: str):
    pda = PDA(programId, "Global Data")
    write(COMPTO_GLOBAL_DATA_ACCOUNT_JSON, json.dumps(pda))

def writeInterestBankPDA(programId: str):
    pda = PDA(programId, "Interest Bank")
    write(COMPTO_INTEREST_BANK_ACCOUNT_JSON, json.dumps(pda))

def writeUBIBankPDA(programId: str):
    pda = PDA(programId, "UBI Bank")
    write(COMPTO_UBI_BANK_ACCOUNT_JSON, json.dumps(pda))

def write(path: Path, jsonStr: str):
    with open(path) as file:
        file.write(jsonStr)

if __name__ == "__main__":
    programId = base58.b58encode(random.Random().randbytes(32)).decode()
    pda = PDA(programId, "")
