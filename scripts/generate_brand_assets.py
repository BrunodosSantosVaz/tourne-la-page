"""Export the supplied folded-page symbol; requires rsvg-convert and ImageMagick."""
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / 'assets/brand/icons'
SIZES = (16, 24, 32, 48, 64, 128, 256, 512, 1024)


def main():
    TARGET.mkdir(parents=True, exist_ok=True)
    for size in SIZES:
        subprocess.run(['rsvg-convert', '-w', str(size), '-h', str(size),
                        '-o', str(TARGET / f'app-{size}.png'), str(ROOT / 'assets/brand/mark.svg')], check=True)
    subprocess.run(['magick', *(str(TARGET / f'app-{size}.png') for size in SIZES if size <= 256),
                    str(TARGET / 'tourne-la-page.ico')], check=True)
    entries = b''
    for tag, size in ((b'ic07', 128), (b'ic08', 256), (b'ic09', 512), (b'ic10', 1024)):
        data = (TARGET / f'app-{size}.png').read_bytes()
        entries += tag + struct.pack('>I', len(data) + 8) + data
    (TARGET / 'tourne-la-page.icns').write_bytes(b'icns' + struct.pack('>I', len(entries) + 8) + entries)


if __name__ == '__main__':
    main()
