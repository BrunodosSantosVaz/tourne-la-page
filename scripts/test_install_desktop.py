from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from scripts.install_desktop import APP_ID, install


class DesktopRegistration(unittest.TestCase):
    def test_replacing_an_old_icon_refreshes_desktop_cache_once(self):
        with tempfile.TemporaryDirectory() as temporary, patch(
            'scripts.install_desktop.refresh_icon_cache'
        ) as refresh:
            data_home = Path(temporary)
            root = Path(__file__).resolve().parents[1]
            install(root, data_home, refresh_cache=True)
            refresh.assert_called_once_with(data_home)
            refresh.reset_mock()
            install(root, data_home, refresh_cache=True)
            refresh.assert_not_called()
            icon = data_home / f'icons/hicolor/32x32/apps/{APP_ID}.png'
            icon.write_bytes(b'old icon')
            install(root, data_home, refresh_cache=True)
            refresh.assert_called_once_with(data_home)
            self.assertEqual(
                icon.read_bytes(),
                (root / 'assets/brand/identity/03-app/circulo/azul/png/icone-circulo-azul-32.png').read_bytes(),
            )

    def test_launcher_with_spaces_uses_same_id_as_window_and_has_valid_icons(self):
        with tempfile.TemporaryDirectory(prefix='tourne la page ') as temporary:
            data_home = Path(temporary) / 'desktop data'
            root = Path(__file__).resolve().parents[1]
            entry = install(root, data_home)
            source = entry.read_text()
            self.assertIn('Name=Tourne la page\n', source)
            self.assertIn(f'Icon={APP_ID}\nStartupWMClass={APP_ID}\n', source)
            self.assertTrue((data_home / f'icons/hicolor/256x256/apps/{APP_ID}.png').is_file())
            self.assertTrue((data_home / f'icons/hicolor/scalable/apps/{APP_ID}.svg').is_file())
            subprocess.run(['desktop-file-validate', str(entry)], check=True)
            self.assertEqual(entry, install(root, data_home))


if __name__ == '__main__':
    unittest.main()
