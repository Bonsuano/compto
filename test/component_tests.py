import json
import os
from argparse import Action, ArgumentParser, Namespace
from typing import Any, Sequence

from common import *


def generateFiles():
    print("generating files...")
    programId = randAddress()
    # programId
    generateProgramIdFile(programId)
    # mint
    mint_address = generateMint()
    # pdas
    globalDataPDA = setGlobalDataPDA(programId)
    interestBankPDA = setInterestBankPDA(programId)
    UBIBankPDA = setUBIBankPDA(programId)
    # test user
    generateTestUser()
    # rust file
    generateComptokenAddressFile(
        globalDataPDA["bumpSeed"], interestBankPDA["bumpSeed"], UBIBankPDA["bumpSeed"], mint_address
    )
    print("done generating files")

def generateProgramIdFile(programId: str):
    write(COMPTO_PROGRAM_ID_JSON, json.dumps({"programId": programId}))

def generateMint() -> str:
    address = randAddress()
    file_data = f'''\
{{
    "commandName": "CreateToken",
    "commandOutput": {{
        "address": "{address}",
        "decimals": {MINT_DECIMALS},
        "transactionData": {{
            "signature": ""
        }}
    }}
}}\
'''
    write(COMPTOKEN_MINT_JSON, file_data)
    return address

def runTest(test: str, file: str) -> bool:
    print(f"running {test}")
    env = os.environ
    env["SBF_OUT_DIR"] = str(PROJECT_PATH / "target/deploy/")
    try:
        run(f"node {TEST_PATH / f'compto-test-client/{file}'}", env=env)
        print(f"{test} passed")
        return True
    except SubprocessFailedException as e:
        print(f"{test} failed")
        print(e)
        return False

def runTests(args: Namespace):
    print("running tests...")

    passed = 0
    skipped = 0
    for (test, val) in args._get_kwargs():
        if val:
            passed += runTest(test, f'test_{test}')
        else:
            skipped += 1
            print(f"{test} skipped")
    failed = len(args._get_kwargs()) - passed - skipped
    print()
    print(f"passed: {passed}    failed: {failed}    skipped: {skipped}")

# from https://stackoverflow.com/questions/48834678/python-argparse-is-there-a-clean-way-to-add-a-flag-that-sets-multiple-flags
def store_const_multiple(const: Any, *destinations: str):
    """Returns an `Action` class that sets multiple argument destinations (`destinations`) to `const`."""

    class store_const_multiple_action(Action):

        def __init__(self, *args, **kwargs):  # type: ignore
            super(store_const_multiple_action,
                  self).__init__(metavar=None, nargs=0, const=const, *args, **kwargs)  # type: ignore

        def __call__(
            self,
            parser: ArgumentParser,
            namespace: Namespace,
            values: str | Sequence[Any] | None,
            option_string: str | None = None
        ):
            for destination in destinations:
                setattr(namespace, destination, const)

    return store_const_multiple_action

def store_true_multiple(*destinations: str):
    """Returns an `Action` class that sets multiple argument destinations (`destinations`) to `True`."""
    return store_const_multiple(True, *destinations)

def store_false_multiple(*destinations: str):
    """Returns an `Action` class that sets multiple argument destinations (`destinations`) to `True`."""
    return store_const_multiple(False, *destinations)

def parseArgs(tests: list[str]):
    parser = ArgumentParser(prog="comptoken component tests")
    for argument in tests:
        parser.add_argument(f"--no-{argument.replace('_', '-')}", action="store_false", dest=argument)
        parser.add_argument(f"--{argument.replace('_', '-')}", action="store_true", dest=argument)
    parser.add_argument("--none", action=store_false_multiple(*tests), dest="mint")

    return parser.parse_args()

if __name__ == "__main__":
    tests = [
        "mint", "initialize_comptoken_program", "create_user_data_account", "proof_submission", "get_valid_blockhashes",
        "get_owed_comptokens", "daily_distribution_event"
    ]
    args = parseArgs(tests)
    # create cache if it doesn't exist
    run(f"[ -d {CACHE_PATH} ] || mkdir {CACHE_PATH} ")
    run(f"[ -d {GENERATED_PATH} ] || mkdir {GENERATED_PATH} ")
    generateFiles()
    build()
    runTests(args)
