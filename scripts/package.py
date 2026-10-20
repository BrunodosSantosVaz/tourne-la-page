"""Build one native portable distribution using the Big Bang compiled-artifact contract."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parent.parent
APP_ID = "io.github.BrunodosSantosVaz.TourneLaPage"
TARGETS = {
    "linux-x64": ("Linux", "x86_64-unknown-linux-gnu"),
    "linux-arm64": ("Linux", "aarch64-unknown-linux-gnu"),
    "windows-x64": ("Windows", "x86_64-pc-windows-msvc"),
    "windows-arm64": ("Windows", "aarch64-pc-windows-msvc"),
    "macos-x64": ("Darwin", "x86_64-apple-darwin"),
    "macos-arm64": ("Darwin", "aarch64-apple-darwin"),
}


def extract_zip(archive, destination):
    members = archive.infolist()
    for member in members:
        name = member.filename
        if (name.startswith("/") or "\\" in name or ":" in name or ".." in name.split("/")
                or (member.external_attr >> 16) & 0o170000 == 0o120000):
            raise ValueError(f"Unsafe ZIP member: {name}")
    for member in members:
        path = destination / member.filename
        if member.is_dir():
            path.mkdir(parents=True, exist_ok=True)
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            with archive.open(member) as source, path.open("wb") as output:
                shutil.copyfileobj(source, output)


def bundle(binary, system, version, output):
    if system not in TARGETS or not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise ValueError("Unsupported platform or invalid release version")
    binary, output = Path(binary), Path(output)
    if not binary.is_file():
        raise FileNotFoundError(binary)
    if output.exists() and any(output.iterdir()):
        raise ValueError("The Big Bang output directory must be empty")
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="tourne-la-page-package-") as temporary:
        folder = Path(temporary) / "Tourne-la-page"
        folder.mkdir()
        if system.startswith("linux-"):
            destination = folder / "bin/tourne-la-page"
            destination.parent.mkdir()
            launcher = folder / "AppRun"
            launcher.write_text('#!/bin/sh\nset -eu\nAPP_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)\nexec "$APP_DIR/bin/tourne-la-page" "$@"\n')
            launcher.chmod(0o755)
            shutil.copy2(ROOT / "scripts/install_portable.py", folder / "install.py")
            shutil.copy2(ROOT / "assets/brand/icons/app-256.png", folder / "tourne-la-page.png")
        elif system.startswith("windows-"):
            destination = folder / "tourne-la-page.exe"
            shutil.copy2(ROOT / "assets/brand/icons/tourne-la-page.ico", folder / "tourne-la-page.ico")
        else:
            contents = folder / "Tourne la page.app/Contents"
            destination = contents / "MacOS/tourne-la-page"
            destination.parent.mkdir(parents=True)
            (contents / "Resources").mkdir()
            shutil.copy2(ROOT / "assets/brand/icons/tourne-la-page.icns", contents / "Resources/tourne-la-page.icns")
            with (contents / "Info.plist").open("wb") as stream:
                plistlib.dump({"CFBundleIdentifier": APP_ID, "CFBundleName": "Tourne la page",
                    "CFBundleDisplayName": "Tourne la page", "CFBundleExecutable": "tourne-la-page",
                    "CFBundlePackageType": "APPL", "CFBundleShortVersionString": version,
                    "CFBundleVersion": version, "CFBundleIconFile": "tourne-la-page.icns",
                    "LSMinimumSystemVersion": "12.0", "NSHighResolutionCapable": True}, stream)
        shutil.copy2(binary, destination)
        destination.chmod(0o755)
        licenses = (folder / "Tourne la page.app/Contents/Resources/licenses"
                    if system.startswith("macos-") else folder / "licenses")
        licenses.mkdir()
        shutil.copytree(ROOT / "assets/licenses/rust", licenses / "rust")
        shutil.copy2(ROOT / "LICENSE", licenses / "Tourne-la-page-MIT.txt")
        for font in ("Inter", "Literata", "Atkinson", "Lexend"):
            shutil.copy2(ROOT / f"assets/fonts/{font}/OFL.txt", licenses / f"{font}-OFL.txt")
        shutil.copy2(ROOT / "assets/icons/lucide/LICENSE", licenses / "Lucide-ISC.txt")
        (folder / "LEIA-ME.txt").write_text(
            f"Tourne la page {version} — {system}\n\n"
            "Linux: execute ./AppRun. Opcional: python3 install.py registra o atalho desta pasta.\n"
            "Windows: abra tourne-la-page.exe. macOS: arraste Tourne la page.app para Aplicativos.\n"
            "Pacotes sem certificado comercial; o sistema pode solicitar autorização para abrir.\n"
            "Dados pessoais ficam no diretório do usuário; EPUBs não são alterados.\n"
            "A remoção do aplicativo não apaga sua biblioteca pessoal.\n"
            "Manuais, limites, dados e segurança: https://github.com/BrunodosSantosVaz/tourne-la-page/wiki\n",
            encoding="utf-8")
        if system.startswith("macos-") and platform.system() == "Darwin":
            subprocess.run(["codesign", "--force", "--deep", "--sign", "-", str(folder / "Tourne la page.app")], check=True)
            subprocess.run(["codesign", "--verify", "--deep", "--strict", str(folder / "Tourne la page.app")], check=True)
        manifest = {"product": "Tourne la page", "version": version, "system": system,
                    "files": {p.relative_to(folder).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                              for p in sorted(folder.rglob("*")) if p.is_file()}}
        (folder / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
        extension = ".tar.gz" if system.startswith("linux-") else ".zip"
        result = output / f"tourne-la-page-{version}-{system}{extension}"
        if extension == ".tar.gz":
            with tarfile.open(result, "w:gz") as archive:
                archive.add(folder, arcname=folder.name)
        else:
            with zipfile.ZipFile(result, "w", zipfile.ZIP_DEFLATED) as archive:
                for entry in sorted(folder.rglob("*")):
                    if entry.is_file():
                        archive.write(entry, entry.relative_to(folder.parent))
        return result



def verify_installation(launcher, package_root, environment, version):
    package_root = Path(package_root).resolve(strict=True)
    executable = Path(launcher).resolve(strict=True)
    allowed = {package_root / "AppRun", package_root / "tourne-la-page.exe",
               package_root / "Tourne la page.app/Contents/MacOS/tourne-la-page"}
    if executable not in allowed or not executable.is_file():
        raise ValueError("The smoke executable must be a known launcher inside the extracted package")
    # Windows does not resolve a relative executable against cwd with shell=False.
    subprocess.run(["tourne-la-page", "--verify-install", "install-verification.json"],
                   executable=str(executable), cwd=package_root, env=environment, check=True)
    details = json.loads((package_root / "install-verification.json").read_text())
    if details["database"] != "ok" or details["version"] != version:
        raise SystemExit("The native installation verification failed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("system", choices=TARGETS)
    system = parser.parse_args().system
    expected_os, target = TARGETS[system]
    if platform.system() != expected_os:
        raise SystemExit(f"{system} must be compiled on its native {expected_os} runner")
    version = os.environ.get("BB_VERSAO", tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"])
    output = Path(os.environ.get("BB_SAIDA", ROOT / "dist" / system))
    environment = os.environ.copy()
    environment.setdefault("CARGO_BUILD_JOBS", "2")
    if expected_os == "Windows":
        environment["RUSTFLAGS"] = environment.get("RUSTFLAGS", "") + " -C target-feature=+crt-static"
    if expected_os == "Darwin":
        environment["MACOSX_DEPLOYMENT_TARGET"] = "12.0"
    subprocess.run(["rustup", "target", "add", target], cwd=ROOT, check=True)
    if expected_os == "Linux":
        image = "ubuntu@sha256:0a9887931b0e38af2e3487ed39e29436b0535c4f3f52e57745baaf79d1898956"
        cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo")).resolve()
        rustup_home = Path(os.environ.get("RUSTUP_HOME", Path.home() / ".rustup")).resolve()
        subprocess.run(["docker", "run", "--rm", "--volume", f"{ROOT}:/src", "--volume",
                        f"{cargo_home}:/cargo", "--volume", f"{rustup_home}:/rustup:ro",
                        "--workdir", "/src", "--env", "CARGO_HOME=/cargo", "--env", "RUSTUP_HOME=/rustup",
                        "--env", "PATH=/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin", "--env", "CARGO_BUILD_JOBS=2",
                        image, "bash", "packaging/linux/build.sh", target], check=True)
    else:
        subprocess.run(["cargo", "build", "--package", "app", "--bin", "tourne-la-page", "--release", "--locked", "--target", target], cwd=ROOT, env=environment, check=True)
    binary = ROOT / "target" / target / "release" / ("tourne-la-page.exe" if expected_os == "Windows" else "tourne-la-page")
    if expected_os == "Linux":
        symbols = subprocess.check_output(["readelf", "--version-info", str(binary)], text=True)
        versions = [tuple(map(int, v.split("."))) for v in re.findall(r"GLIBC_([0-9.]+)", symbols)]
        if versions and max(versions) > (2, 35):
            raise SystemExit("The binary requires glibc newer than the promised 2.35 baseline")
    archive_path = bundle(binary, system, version, output)
    with tempfile.TemporaryDirectory(prefix="tourne-la-page-smoke-") as temporary:
        extracted = Path(temporary) / "extracted"
        extracted.mkdir()
        if system.startswith("linux-"):
            with tarfile.open(archive_path) as archive:
                archive.extractall(extracted, filter="data")
            launcher = extracted / "Tourne-la-page/AppRun"
        else:
            with zipfile.ZipFile(archive_path) as archive:
                if expected_os == "Darwin":
                    member = archive.getinfo("Tourne-la-page/Tourne la page.app/Contents/MacOS/tourne-la-page")
                    if not (member.external_attr >> 16) & 0o111:
                        raise SystemExit("The distributed macOS executable lost its executable permissions")
                extract_zip(archive, extracted)
            launcher = extracted / "Tourne-la-page" / ("tourne-la-page.exe" if expected_os == "Windows" else
                       "Tourne la page.app/Contents/MacOS/tourne-la-page")
            launcher.chmod(0o755)
        manifest = json.loads((extracted / "Tourne-la-page/manifest.json").read_text())
        for name, digest in manifest["files"].items():
            if hashlib.sha256((extracted / "Tourne-la-page" / name).read_bytes()).hexdigest() != digest:
                raise SystemExit(f"Extracted package hash mismatch: {name}")
        if expected_os == "Darwin":
            subprocess.run(["codesign", "--verify", "--deep", "--strict", str(extracted / "Tourne-la-page/Tourne la page.app")], check=True)
        verification = environment.copy()
        verification.update({"TOURNEPAGE_ENV": "homologacao", "XDG_DATA_HOME": temporary,
                             "LOCALAPPDATA": temporary, "APPDATA": temporary, "HOME": temporary})
        package_root = extracted / "Tourne-la-page"
        verify_installation(launcher, package_root, verification, version)
        print("Extracted native package initialized its database outside the checkout.")
    print(archive_path)


if __name__ == "__main__":
    main()
