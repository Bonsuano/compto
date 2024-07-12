import json
import os
import signal
import subprocess
from pathlib import Path
from time import sleep, time
from types import TracebackType
from typing import Any, Self, Type

TEST_PATH = Path(__file__).parent
PROJECT_PATH = TEST_PATH.parent
CACHE_PATH = TEST_PATH / ".cache"
GENERATED_PATH = PROJECT_PATH / "src/generated"
COMPTO_MD5_JSON = CACHE_PATH / "compto_md5sum.json"
COMPTO_PROGRAM_ID_JSON = CACHE_PATH / "compto_program_id.json"
COMPTO_SO = PROJECT_PATH / "target/deploy/comptoken.so"
COMPTOKEN_MINT_JSON = CACHE_PATH / "comptoken_mint.json"
TEST_USER_ACCOUNT_JSON = CACHE_PATH / "test_user_account.json"
COMPTO_GENERATED_RS_FILE = GENERATED_PATH / "comptoken_generated.rs"
COMPTO_GLOBAL_DATA_ACCOUNT_JSON = CACHE_PATH / "compto_global_data_account.json"
COMPTO_INTEREST_BANK_ACCOUNT_JSON = CACHE_PATH / "compto_interest_bank_account.json"
COMPTO_UBI_BANK_ACCOUNT_JSON = CACHE_PATH / "compto_ubi_bank_account.json"
TOKEN_2022_PROGRAM_ID = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"


class SubprocessFailedException(Exception):
    pass


# ==== SOLANA COMMANDS ====

def checkIfProgamIdExists():
    try:
        getProgramId()
        return True
    except SubprocessFailedException:
        return False

def getProgramId():
    return run(f"solana address -k target/deploy/comptoken-keypair.json")


def createToken():
    run(
        f"spl-token --program-id {TOKEN_2022_PROGRAM_ID} create-token -v  --output json > {COMPTOKEN_MINT_JSON}"
    )


def createKeyPair(outfile: Path):
    run(f"solana-keygen new --no-bip39-passphrase --force --silent --outfile {outfile}")


def createComptoAccount():
    createKeyPair(TEST_USER_ACCOUNT_JSON)
    run(
        f"spl-token --program-id {TOKEN_2022_PROGRAM_ID} create-account {getTokenAddress()} {TEST_USER_ACCOUNT_JSON}"
    )


def getPubkey(path: Path) -> str:
    return run(f"solana-keygen pubkey {path}")


def getAccountBalance(pubkey: str):
    return run(f"solana balance {pubkey}")


def deploy():
    run(
        f"solana program deploy -v {COMPTO_SO} --output json > {COMPTO_PROGRAM_ID_JSON}"
    )


def checkIfCurrentMintAuthorityExists() -> bool:
    # TODO: find a more efficient way to do this
    try:
        json.loads(
            run(
                f"spl-token --program-id {TOKEN_2022_PROGRAM_ID} display {getTokenAddress()} --output json"
            )
        ).get("MintAuthority")
        return True
    except (FileNotFoundError, SubprocessFailedException, json.decoder.JSONDecodeError):
        return False
    except Exception as ex:
        print(f"new Exception: Type:`{type(ex)}' value: `{ex}'")
        raise ex


def getCurrentMintAuthority() -> str:
    return json.loads(
        run(
            f"spl-token --program-id {TOKEN_2022_PROGRAM_ID} display {getTokenAddress()} --output json"
        )
    ).get("MintAuthority")


def setGlobalDataPda():
    setPda("Global Data", COMPTO_GLOBAL_DATA_ACCOUNT_JSON)


def setInterestBankPda():
    setPda("Interest Bank", COMPTO_INTEREST_BANK_ACCOUNT_JSON)


def setUbiBankPda():
    setPda("UBI Bank", COMPTO_UBI_BANK_ACCOUNT_JSON)


def setPda(seed: str, outfile: Path):
    run(
        f"solana find-program-derived-address {getProgramId()} string:'{seed}' --output json > {outfile}"
    )


def getGlobalDataPDA():
    return json.loads(COMPTO_GLOBAL_DATA_ACCOUNT_JSON.read_text())


def getInterestBankPda():
    return json.loads(COMPTO_INTEREST_BANK_ACCOUNT_JSON.read_text())


def getUbiBankPda():
    return json.loads(COMPTO_UBI_BANK_ACCOUNT_JSON.read_text())


# ==== SHELL COMMANDS ====
def build():
    run("cargo build-sbf --features \"testmode\"", PROJECT_PATH)


def getComptoMd5():
    return run(f"md5sum {COMPTO_SO}", PROJECT_PATH).split()[0]


# ========================


def checkIfProgamIdChanged() -> bool:
    # Only deploy if the program id has changed
    if not COMPTO_PROGRAM_ID_JSON.exists():
        return False
    real_program_id = getProgramId()
    cached_program_id = json.loads(COMPTO_PROGRAM_ID_JSON.read_text())["programId"]
    return real_program_id != cached_program_id


def deployIfNeeded():
    # Only deploy if the md5sum of the program has changed
    md5sum = getComptoMd5()
    if (
        not COMPTO_MD5_JSON.exists()
        or json.loads(COMPTO_MD5_JSON.read_text())["md5sum"] != md5sum
    ):
        COMPTO_MD5_JSON.write_text(json.dumps({"md5sum": md5sum}))
        deploy()
    else:
        print("Program has not changed, skipping deploy.")


def generateComptokenAddressFile():
    setGlobalDataPda()
    setInterestBankPda()
    setUbiBankPda()
    globalDataSeed = getGlobalDataPDA()["bumpSeed"]
    interestBankSeed = getInterestBankPda()["bumpSeed"]
    UBIBankSeed = getUbiBankPda()["bumpSeed"]
    comptoken_id = getTokenAddress()

    print(f"Generating {COMPTO_GENERATED_RS_FILE}...")
    file_data = f"""\
// AUTOGENERATED DO NOT TOUCH
// generated by test/full_deploy_test.py

// A given seed and program id have a 50% chance of creating a valid PDA.
// Before building/deploying, we find the canonical seed by running 
//      `solana find-program-derived-address <program_id>`
// This is an efficiency optimization. We are using a static seed to create the PDA with no bump.
// We ensure when deploying that the program id is one that only needs the seed above and no bump.
// This is because 
//      (1) create_program_address is not safe if using a user provided bump.
//      (2) find_program_address is expensive and we want to avoid iterations.

use spl_token_2022::solana_program::{{pubkey, pubkey::Pubkey}};

pub const COMPTOKEN_MINT_ACCOUNT_ADDRESS: Pubkey = pubkey!("{comptoken_id}");

pub const COMPTO_GLOBAL_DATA_ACCOUNT_BUMP: u8 = {globalDataSeed};
pub const COMPTO_INTEREST_BANK_ACCOUNT_BUMP: u8 = {interestBankSeed};
pub const COMPTO_UBI_BANK_ACCOUNT_BUMP: u8 = {UBIBankSeed};\
"""

    with open(COMPTO_GENERATED_RS_FILE, "w") as file:
        file.write(file_data)


def checkIfTokenAddressExists() -> bool:
    return COMPTOKEN_MINT_JSON.exists()


def getTokenAddress():
    return (
        json.loads(COMPTOKEN_MINT_JSON.read_text()).get("commandOutput").get("address")
    )


def createTokenIfNeeded():
    # TODO: put TokenCreation and MintAuthorityCreation together
    if not checkIfTokenAddressExists() or not checkIfCurrentMintAuthorityExists():
        print("Creating new Comptoken...")
        createToken()
    # If a new program id is created, the mint authority will not match.
    # Rather than have the old mint authority sign over the new authority, we will just create a new token.
    elif getCurrentMintAuthority() != getGlobalDataPDA()["address"]:
        print("Mint Authority doesn't match. Creating new Comptoken...")
        createToken()
    elif checkIfProgamIdChanged():
        print("Program ID has changed. Creating new Comptoken...")
        createToken()
    else:
        print("Using existing Comptoken...")


def run(command: str | list[str], cwd: Path | None = None):
    result = subprocess.run(
        command, shell=True, cwd=cwd, capture_output=True, text=True
    )
    if result.returncode != 0:
        raise SubprocessFailedException(
            f"Failed to run command! command: {command} stdout: {result.stdout} stderr: {result.stderr}"
        )
    return result.stdout.rstrip()


def runTestClient():
    return run("node compto-test-client/test_client.js", TEST_PATH)


class BackgroundProcess:
    _cmd: str | list[str]
    _kwargs: dict[str, Any]
    _process: subprocess.Popen[Any] | None = None

    def __init__(self, cmd: str | list[str], **kwargs: Any):
        self._cmd = cmd
        self._kwargs = kwargs

    def __enter__(self) -> Self:
        self._process = subprocess.Popen(self._cmd, **self._kwargs)
        return self

    def __exit__(
        self,
        exc_type: Type[BaseException] | None,
        exc_value: BaseException | None,
        exc_tb: TracebackType,
    ) -> bool:
        print("Killing Background Process...")
        if self._process is not None and self.checkIfProcessRunning():
            os.killpg(os.getpgid(self._process.pid), signal.SIGTERM)
        return False

    def checkIfProcessRunning(self):
        return self._process is not None and self._process.poll() is None


def checkIfValidatorReady(validator: BackgroundProcess) -> bool:
    if not validator.checkIfProcessRunning():
        return False
    try:
        run("solana ping -c 1")
        return True
    except Exception:
        return False


def waitTillValidatorReady(validator: BackgroundProcess):
    TIMEOUT = 10
    t1 = time()
    while not checkIfValidatorReady(validator):
        if t1 + TIMEOUT < time():
            print("Validator Timeout, Exiting...")
            exit(1)
        print("Validator Not Ready")
        sleep(1)


if __name__ == "__main__":
    # create cache if it doesn't exist
    run(f"[ -d {CACHE_PATH} ] || mkdir {CACHE_PATH} ")
    run(f"[ -d {GENERATED_PATH} ] || mkdir {GENERATED_PATH} ")
    # If ProgramId doesn't exist, we need to build WITHOUT the testmode feature.
    # This is because the static seed in testmode depends on ProgramId and ProgramId
    # is generated on the first build.
    print("Checking if Comptoken ProgramId exists...")
    if not checkIfProgamIdExists():
        print("Creating Comptoken ProgramId...")
        run("cargo build-sbf", PROJECT_PATH)
    print("Creating Validator...")
    with BackgroundProcess(
        "solana-test-validator --reset",
        shell=True,
        cwd=CACHE_PATH,
        stdout=subprocess.DEVNULL,
        preexec_fn=os.setsid,
    ) as validator:
        print("Checking Validator Ready...")
        waitTillValidatorReady(validator)
        print("Validator Ready")
        createTokenIfNeeded()
        print(
            "Checking Compto Program for hardcoded Comptoken Address and static seed..."
        )
        generateComptokenAddressFile()
        print("Creating Token Account...")
        createComptoAccount()
        print("Building...")
        build()
        print("Deploying...")
        deployIfNeeded()
        print("Running Test Client...")
        output = runTestClient()
        print(output)
        test_account = getPubkey(TEST_USER_ACCOUNT_JSON)
        print(f"Test Account {test_account} Balance: {getAccountBalance(test_account)}")
