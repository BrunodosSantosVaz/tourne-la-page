"""Register this Linux portable directory in the current user's application menu."""
import os
from pathlib import Path
import shutil


def install(root, data_home):
    app_id = "io.github.BrunodosSantosVaz.TourneLaPage"
    applications = data_home / "applications"
    icons = data_home / "icons/hicolor/256x256/apps"
    applications.mkdir(parents=True, exist_ok=True)
    icons.mkdir(parents=True, exist_ok=True)
    shutil.copy2(root / "tourne-la-page.png", icons / f"{app_id}.png")
    executable = str(root / "AppRun")
    for original, escaped in [("\\", "\\\\"), ('"', '\\"'), ("`", "\\`"), ("$", "\\$"), ("%", "%%")]:
        executable = executable.replace(original, escaped)
    entry = applications / f"{app_id}.desktop"
    entry.write_text(f'[Desktop Entry]\nType=Application\nName=Tourne la page\nExec="{executable}"\nIcon={app_id}\n'
                     f'StartupWMClass={app_id}\nTerminal=false\nCategories=Office;Viewer;\n', encoding="utf-8")
    return entry


if __name__ == "__main__":
    root = Path(__file__).resolve().parent
    data_home = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local/share"))
    print(install(root, data_home))
