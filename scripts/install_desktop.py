"""Register the local app and its icons in the current Linux user's desktop menu."""
import os
from pathlib import Path
import shutil
import subprocess
import sys

APP_ID = 'io.github.BrunodosSantosVaz.TourneLaPage'


def quote_exec(value: str) -> str:
    escaped = value.replace('\\', '\\\\').replace('"', '\\"').replace('`', '\\`').replace('$', '\\$').replace('%', '%%')
    return f'"{escaped}"'


def write_if_changed(target: Path, contents: bytes) -> bool:
    if target.exists() and target.read_bytes() == contents:
        return False
    target.write_bytes(contents)
    return True


def refresh_icon_cache(data_home: Path) -> None:
    if shutil.which('gtk-update-icon-cache'):
        try:
            subprocess.run(['gtk-update-icon-cache', '--force', '--ignore-theme-index', str(data_home / 'icons/hicolor')],
                           check=False, timeout=5, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        except (OSError, subprocess.TimeoutExpired):
            pass  # Optional desktop refresh must not prevent opening a book.
    if 'KDE' in os.environ.get('XDG_CURRENT_DESKTOP', '').split(':'):
        cache_home = Path(os.environ.get('XDG_CACHE_HOME', Path.home() / '.cache'))
        (cache_home / 'icon-cache.kcache').unlink(missing_ok=True)
        if shutil.which('dbus-send'):
            for group in (0, 4):
                try:
                    subprocess.run(['dbus-send', '--session', '--type=signal', '/KIconLoader',
                                    'org.kde.KIconLoader.iconChanged', f'int32:{group}'], check=False, timeout=5,
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                except (OSError, subprocess.TimeoutExpired):
                    pass  # Optional notification; the icon files are already registered.


def install(root: Path, data_home: Path, *, refresh_cache: bool = False) -> Path:
    applications = data_home / 'applications'
    applications.mkdir(parents=True, exist_ok=True)
    desktop = applications / f'{APP_ID}.desktop'
    command = ' '.join(map(quote_exec, (sys.executable, str(root / 'local.py'), 'run')))
    contents = ('[Desktop Entry]\nType=Application\nName=Tourne la page\n'
                'Comment=Sua biblioteca EPUB, no seu ritmo\n'
                f'Exec={command}\nIcon={APP_ID}\nStartupWMClass={APP_ID}\n'
                'Terminal=false\nCategories=Office;Viewer;\nKeywords=EPUB;Livros;Leitura;\n')
    changed = write_if_changed(desktop, contents.encode('utf-8'))
    for source in (root / 'assets/brand/icons').glob('app-*.png'):
        size = int(source.stem.split('-')[1])
        target = data_home / f'icons/hicolor/{size}x{size}/apps/{APP_ID}.png'
        target.parent.mkdir(parents=True, exist_ok=True)
        changed |= write_if_changed(target, source.read_bytes())
    scalable = data_home / 'icons/hicolor/scalable/apps'
    scalable.mkdir(parents=True, exist_ok=True)
    changed |= write_if_changed(scalable / f'{APP_ID}.svg', (root / 'assets/brand/mark.svg').read_bytes())
    if changed and refresh_cache:
        refresh_icon_cache(data_home)
    return desktop


def install_current_user(root: Path) -> None:
    if sys.platform.startswith('linux'):
        data_home = Path(os.environ.get('XDG_DATA_HOME', Path.home() / '.local/share'))
        install(root, data_home, refresh_cache=True)


if __name__ == '__main__':
    install_current_user(Path(__file__).resolve().parents[1])
