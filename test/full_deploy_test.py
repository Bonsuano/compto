import json
import subprocess
from pathlib import Path

TEST_PATH = Path(__file__).parent
PROJECT_PATH = TEST_PATH.parent
CACHE_PATH = TEST_PATH / ".cache"
COMPTO_MD5_JSON = CACHE_PATH / "compto_md5sum.json"
COMPTO_PROGRAM_ID_JSON = CACHE_PATH / "compto_program_id.json"
COMPTO_SO = PROJECT_PATH / "target/deploy/comptoken.so"
COMPTOKEN_ID_JSON = CACHE_PATH / "comptoken_id.json"
COMPTO_TEST_ACCOUNT = CACHE_PATH / "compto_test_account.json"
COMPTO_MINT_AUTHORITY_JSON = CACHE_PATH / "compto_mint_authority.json"
COMPTO_PROGRAM_SOURCE = PROJECT_PATH / "src/comptoken.rs"
COMPTO_STATIC_PDA = CACHE_PATH / "compto_static_pda.json"


# ==== SOLANA COMMANDS ====
def getProgramId():
    return run(f"solana address -k target/deploy/comptoken-keypair.json")


def createToken():
    run(f"spl-token create-token -v --output json > {COMPTOKEN_ID_JSON}")


def createKeyPair(outfile: Path):
    run(f"solana-keygen new --no-bip39-passphrase --force --silent --outfile {outfile}")


def createComptoAccount():
    createKeyPair(COMPTO_TEST_ACCOUNT)
    run(f"spl-token create-account {getTokenAddress()} {COMPTO_TEST_ACCOUNT}")


def getPubkey(path: Path) -> str:
    return run(f"solana-keygen pubkey {path}")


def getAccountBalance(pubkey: str):
    return run(f"solana balance {pubkey}")


def deploy():
    run(
        f"solana program deploy -v {COMPTO_SO} --output json > {COMPTO_PROGRAM_ID_JSON}"
    )


def setComptoMintAuthority():
    run(
        f"spl-token authorize {getTokenAddress()} mint {getProgramId()} --output json > {COMPTO_MINT_AUTHORITY_JSON}"
    )


def getCurrentMintAuthority() -> str:
    return json.loads(run(f"spl-token display {getTokenAddress()} --output json")).get(
        "MintAuthority"
    )


def setStaticPda():
    run(
        f"solana find-program-derived-address {getProgramId()} --output json > {COMPTO_STATIC_PDA}"
    )


def getStaticPda():
    return json.loads(COMPTO_STATIC_PDA.read_text())


# ==== SHELL COMMANDS ====
def build():
    run("cargo build-sbf", PROJECT_PATH)


def getComptoMd5():
    return run(f"md5sum {COMPTO_SO}", PROJECT_PATH).split()[0]


# ========================


def checkIfProgamIdChanged():
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


def hardcodeComptoAddress():
    comptoken_id = getTokenAddress()
    # place the new token address in the source code of the program
    with open(COMPTO_PROGRAM_SOURCE, "r") as file:
        lines = file.readlines()
        for i, line in enumerate(lines):
            if "static COMPTOKEN_ADDRESS: Pubkey = pubkey!(" in line:
                if comptoken_id is None or comptoken_id not in line:
                    print("Hardcoding comptoken address...")
                    lines[i] = (
                        f'static COMPTOKEN_ADDRESS: Pubkey = pubkey!("{comptoken_id}");\n'
                    )
                    with open(COMPTO_PROGRAM_SOURCE, "w") as write_file:
                        write_file.writelines(lines)
                break


def hardcodeComptoStaticSeed():
    setStaticPda()
    seed = getStaticPda()["bumpSeed"]
    with open(COMPTO_PROGRAM_SOURCE, "r") as file:
        lines = file.readlines()
        for i, line in enumerate(lines):
            if "static COMPTO_STATIC_ADDRESS_SEED: u8 = " in line:
                if str(seed) not in line:
                    print("Hardcoding compto static seed...")
                    lines[i] = f"static COMPTO_STATIC_ADDRESS_SEED: u8 = {seed};\n"
                    with open(COMPTO_PROGRAM_SOURCE, "w") as write_file:
                        write_file.writelines(lines)
                break


def checkIfTokenAddressExists() -> bool:
    return COMPTOKEN_ID_JSON.exists()


def getTokenAddress():
    return json.loads(COMPTOKEN_ID_JSON.read_text()).get("commandOutput").get("address")


def createTokenIfNeeded():
    # TODO: put TokenCreation and MintAuthorityCreation together
    if tokenAddressExists():
        print("Creating new Comptoken...")
        createToken()
    # If a new program id is created, the mint authority will not match.
    # Rather than have the old mint authority sign over the new authority, we will just create a new token.
    elif getCurrentMintAuthority() != getStaticPda()["address"]:
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
        raise Exception(
            f"Failed to run command! command: {command} stdout: {result.stdout} stderr: {result.stderr}"
        )
    return result.stdout.rstrip()


def runTestClient():
    return run("node compto-test-client/test_client.js", TEST_PATH)


if __name__ == "__main__":
    print("Cargo Build...")
    run("cargo build-sbf", PROJECT_PATH)
    createTokenIfNeeded()
    print("Checking Compto Program for hardcoded Comptoken Address and static seed...")
    hardcodeComptoAddress()
    hardcodeComptoStaticSeed()
    print("Creating Token Account...")
    createComptoAccount()
    print("Building...")
    build()
    print("Deploying...")
    deployIfNeeded()
    print("Running Test Client...")
    output = runTestClient()
    print(output)
    test_account = getPubkey(COMPTO_TEST_ACCOUNT)
    print(f"Test Account {test_account} Balance: {getAccountBalance(test_account)}")
