"""Fast, isolated development commands for the Tourne la page desktop app."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

from scripts.install_desktop import install_current_user


ROOT = Path(__file__).resolve().parent
DATA_DIR = ROOT / ".local" / "tournepage"


def cargo_executable() -> Path:
    """Find Cargo even when the shell did not load rustup's bin directory."""
    found = shutil.which("cargo")
    fallback = Path.home() / ".cargo" / "bin" / "cargo"
    if found:
        return Path(found)
    if fallback.is_file():
        return fallback
    raise SystemExit("Rust/Cargo não encontrado. Instale pelo https://rustup.rs/ e abra um novo terminal.")


def run_cargo(arguments: list[str], *, environment: dict[str, str] | None = None) -> int:
    """Run Cargo from this checkout so local state cannot target another project."""
    env = dict(os.environ if environment is None else environment)
    cargo_bin = str(cargo_executable().parent)
    env["PATH"] = os.pathsep.join([cargo_bin, env.get("PATH", "")])
    try:
        return subprocess.run(["cargo", *arguments], cwd=ROOT, env=env, check=False).returncode
    except OSError as error:
        print(f"Não foi possível iniciar Cargo: {error}", file=sys.stderr)
        return 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("check", help="check the workspace without packaging")
    test = commands.add_parser("test", help="run workspace tests or one package's tests")
    test.add_argument("package", nargs="?", help="optional Cargo package name")
    commands.add_parser("build", help="build the app in the fast local profile")
    commands.add_parser("run", help="build and open the app with isolated local data")
    commands.add_parser("clean", help="remove Cargo build cache only")
    options = parser.parse_args()
    if options.command == "check":
        return run_cargo(["check", "--workspace", "--locked"])
    if options.command == "test":
        arguments = ["test", "--locked"]
        arguments += ["--workspace"] if not options.package else ["-p", options.package]
        return run_cargo(arguments)
    if options.command == "build":
        return run_cargo(["build", "-p", "app", "--locked"])
    if options.command == "run":
        install_current_user(ROOT)
        environment = os.environ.copy()
        environment["TOURNEPAGE_ENV"] = "local"
        environment["TOURNEPAGE_DATA_DIR"] = str(DATA_DIR)
        return run_cargo(["run", "-p", "app", "--locked"], environment=environment)
    return run_cargo(["clean"])


if __name__ == "__main__":
    raise SystemExit(main())
