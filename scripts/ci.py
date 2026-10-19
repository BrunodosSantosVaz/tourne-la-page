"""Real stack commands for Big Bang; coverage and acceptance reuse the single test execution."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent
RESULTS = ROOT / ".local" / "ci"
LLVM_COV_VERSION = "0.9.1"


def run(arguments):
    subprocess.run(arguments, cwd=ROOT, check=True)


def cargo(*arguments):
    executable = shutil.which("cargo") or str(Path.home() / ".cargo/bin/cargo")
    return [executable, *arguments]


def install():
    if sys.platform.startswith("linux") and os.environ.get("GITHUB_ACTIONS") == "true":
        run(["sudo", "apt-get", "update", "-qq"])
        run(["sudo", "apt-get", "install", "-y", "pkg-config", "libfontconfig1-dev",
             "libxkbcommon-dev", "libwayland-dev", "libx11-dev", "libxcursor-dev",
             "libxi-dev", "libxrandr-dev", "libgl1-mesa-dev", "libvulkan-dev", "desktop-file-utils"])
    version = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    run(["rustup", "toolchain", "install", version, "--profile", "minimal", "--component",
         "rustfmt,clippy,llvm-tools-preview"])
    run(cargo("fetch", "--locked"))


def architecture():
    metadata = json.loads(subprocess.check_output(cargo("metadata", "--no-deps", "--format-version", "1"), cwd=ROOT))
    allowed = {"app-core": {"epub-core"}, "epub-core": set(), "storage": set(), "app": {"app-core", "epub-core", "storage"}}
    for package in metadata["packages"]:
        internal = {d["name"] for d in package["dependencies"] if d.get("path")}
        if not internal <= allowed[package["name"]]:
            raise SystemExit(f"Forbidden dependency: {package['name']} -> {sorted(internal)}")
    print("Workspace dependency directions validated.")


def test():
    RESULTS.mkdir(parents=True, exist_ok=True)
    if subprocess.run(cargo("llvm-cov", "--version"), capture_output=True, check=False).returncode:
        run(cargo("install", "cargo-llvm-cov", "--version", LLVM_COV_VERSION, "--locked"))
    log = RESULTS / "test-results.log"
    arguments = cargo("llvm-cov", "--workspace", "--all-targets", "--locked", "--no-report")
    with log.open("w", encoding="utf-8") as output:
        with subprocess.Popen(arguments, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True) as process:
            for line in process.stdout:
                print(line, end="", flush=True)
                output.write(line)
            if process.wait():
                raise SystemExit("Rust tests failed; the incomplete log cannot authorize production.")
    run([sys.executable, "-m", "unittest", "discover", "-s", "scripts", "-p", "test_*.py", "-v"])


def acceptance():
    output = (RESULTS / "test-results.log").read_text(encoding="utf-8")
    required = ("installed_builds_use_platform_data_without_requiring_a_checkout",
                "closing_from_library_flushes_ratings_favorites_and_book_comments_without_a_reader",
                "switching_books_immediately_preserves_the_previous_books_last_comment",
                "reading_lists_every_started_unfinished_book_in_recent_order",
                "repeated_chapters_cannot_amplify_the_total_text_budget",
                "opening_epub_from_the_library_preserves_a_valid_widget_tree")
    for name in required:
        if not any(name in line and line.endswith(" ... ok") for line in output.splitlines()):
            raise SystemExit(f"Missing successful acceptance scenario: {name}")
    print("Release acceptance scenarios passed in the recorded test execution.")


def coverage():
    # UI rendering, SQL adapter and test harness are outside domain/application coverage.
    run(cargo("llvm-cov", "report", "--ignore-filename-regex", r"(/app/|/crates/storage/|/tests/)",
              "--fail-under-lines", "80", "--json", "--output-path", str(RESULTS / "coverage.json")))
    print("Domain/application line coverage is at least 80%; no tests were rerun.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("install", "lint", "types", "architecture", "test", "acceptance", "coverage", "build"))
    command = parser.parse_args().command
    if command == "lint":
        run(cargo("fmt", "--all", "--", "--check"))
        run(cargo("clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"))
    elif command == "types":
        run(cargo("check", "--workspace", "--all-targets", "--locked"))
    elif command == "build":
        run(cargo("build", "--package", "app", "--release", "--locked"))
    else:
        globals()[command]()


if __name__ == "__main__":
    main()
