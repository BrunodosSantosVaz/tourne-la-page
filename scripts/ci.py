"""Real stack commands for Big Bang; coverage and acceptance reuse the single test execution."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent
RESULTS = ROOT / ".local" / "ci"
LLVM_COV_VERSION = "0.9.1"


def install():
    if sys.platform == "darwin" and os.environ.get("GITHUB_ACTIONS") == "true":
        path_file = os.environ.get("GITHUB_PATH")
        if not path_file or not Path(path_file).is_absolute():
            raise SystemExit("macOS CI packaging requires an absolute GITHUB_PATH")
        subprocess.run(["brew", "install", "bash", "coreutils"], cwd=ROOT, check=True)
        directories = []
        for formula, suffix in (("bash", "bin"), ("coreutils", "libexec/gnubin")):
            prefix = subprocess.check_output(["brew", "--prefix", formula], cwd=ROOT, text=True).strip()
            if not Path(prefix).is_absolute() or "\n" in prefix or "\r" in prefix:
                raise SystemExit("Homebrew returned an invalid tool prefix")
            directory = Path(prefix) / suffix
            directories.append(str(directory))
        tool_environment = os.environ.copy()
        tool_environment["PATH"] = os.pathsep.join(directories + [os.environ.get("PATH", "")])
        subprocess.run(["bash", "--version"], cwd=ROOT, env=tool_environment, check=True)
        subprocess.run(["sha256sum", "--version"], cwd=ROOT, env=tool_environment, check=True)
        # The runner applies this file before choosing the next step's Bash executable.
        with Path(path_file).open("a", encoding="utf-8") as output:
            output.write("\n".join(directories) + "\n")
    if sys.platform.startswith("linux") and os.environ.get("GITHUB_ACTIONS") == "true":
        subprocess.run(["sudo", "apt-get", "update", "-qq"], cwd=ROOT, check=True)
        subprocess.run(["sudo", "apt-get", "install", "-y", "pkg-config", "libfontconfig1-dev",
             "libxkbcommon-dev", "libwayland-dev", "libx11-dev", "libxcursor-dev",
             "libxi-dev", "libxrandr-dev", "libgl1-mesa-dev", "libvulkan-dev", "desktop-file-utils"], cwd=ROOT, check=True)
    version = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    subprocess.run(["rustup", "toolchain", "install", version, "--profile", "minimal", "--component",
         "rustfmt,clippy,llvm-tools-preview"], cwd=ROOT, check=True)
    subprocess.run(["cargo", "fetch", "--locked"], cwd=ROOT, check=True)


def architecture():
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=ROOT))
    allowed = {"app-core": {"epub-core"}, "epub-core": set(), "storage": set(), "app": {"app-core", "epub-core", "storage"}}
    for package in metadata["packages"]:
        internal = {d["name"] for d in package["dependencies"] if d.get("path")}
        if not internal <= allowed[package["name"]]:
            raise SystemExit(f"Forbidden dependency: {package['name']} -> {sorted(internal)}")
    print("Workspace dependency directions validated.")


def test():
    RESULTS.mkdir(parents=True, exist_ok=True)
    if subprocess.run(["cargo", "llvm-cov", "--version"], capture_output=True, check=False).returncode:
        subprocess.run(["cargo", "install", "cargo-llvm-cov", "--version", LLVM_COV_VERSION, "--locked"], cwd=ROOT, check=True)
    log = RESULTS / "test-results.log"
    with log.open("w", encoding="utf-8") as output:
        with subprocess.Popen(["cargo", "llvm-cov", "--workspace", "--all-targets", "--locked", "--no-report"], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True) as process:
            for line in process.stdout:
                print(line, end="", flush=True)
                output.write(line)
            if process.wait():
                raise SystemExit("Rust tests failed; the incomplete log cannot authorize production.")
    subprocess.run(["python", "-m", "unittest", "discover", "-s", "scripts", "-p", "test_*.py", "-v"], cwd=ROOT, check=True)


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
    subprocess.run(["cargo", "llvm-cov", "report", "--ignore-filename-regex", r"(/app/|/crates/storage/|/tests/)",
              "--fail-under-lines", "80", "--json", "--output-path", str(RESULTS / "coverage.json")], cwd=ROOT, check=True)
    print("Domain/application line coverage is at least 80%; no tests were rerun.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("install", "lint", "types", "architecture", "test", "acceptance", "coverage", "build"))
    command = parser.parse_args().command
    os.environ.setdefault("CARGO_BUILD_JOBS", "2")
    if command == "lint":
        subprocess.run(["cargo", "fmt", "--all", "--", "--check"], cwd=ROOT, check=True)
        subprocess.run(["cargo", "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"], cwd=ROOT, check=True)
    elif command == "types":
        subprocess.run(["cargo", "check", "--workspace", "--all-targets", "--locked"], cwd=ROOT, check=True)
    elif command == "build":
        subprocess.run(["cargo", "build", "--package", "app", "--release", "--locked"], cwd=ROOT, check=True)
    else:
        globals()[command]()


if __name__ == "__main__":
    main()
