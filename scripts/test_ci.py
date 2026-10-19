import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import ci


class ReleaseChecks(unittest.TestCase):
    def test_architecture_rejects_a_domain_dependency_on_sqlite_adapter(self):
        metadata = {"packages": [{"name": "app-core", "dependencies": [{"name": "storage", "path": "/workspace/crates/storage"}]}]}
        with patch("scripts.ci.subprocess.check_output", return_value=json.dumps(metadata)):
            with self.assertRaises(SystemExit):
                ci.architecture()

    def test_acceptance_rejects_a_named_scenario_that_failed_or_never_executed(self):
        with tempfile.TemporaryDirectory() as folder:
            results = Path(folder)
            (results / "test-results.log").write_text("test installed_builds_use_platform_data_without_requiring_a_checkout ... FAILED\n")
            with patch("scripts.ci.RESULTS", results):
                with self.assertRaises(SystemExit):
                    ci.acceptance()

    def test_macos_ci_provides_gnu_tools_to_the_next_workflow_step(self):
        with tempfile.TemporaryDirectory() as folder:
            github_path = Path(folder) / "github-path"
            with patch.dict("scripts.ci.os.environ", {"GITHUB_ACTIONS": "true", "GITHUB_PATH": str(github_path)}), \
                 patch("scripts.ci.sys.platform", "darwin"), \
                 patch("scripts.ci.subprocess.run") as run, \
                 patch("scripts.ci.subprocess.check_output", side_effect=["/opt/homebrew/opt/bash\n", "/opt/homebrew/opt/coreutils\n"]):
                ci.install()
            self.assertTrue(github_path.exists(), "macOS must promote GNU tools before the compiled wrapper runs")
            self.assertEqual(github_path.read_text().splitlines(), [
                "/opt/homebrew/opt/bash/bin", "/opt/homebrew/opt/coreutils/libexec/gnubin"])
            run.assert_any_call(["brew", "install", "bash", "coreutils"], cwd=ci.ROOT, check=True)
            run.assert_any_call(["/opt/homebrew/opt/bash/bin/bash", "--version"], cwd=ci.ROOT, check=True)
            run.assert_any_call(["/opt/homebrew/opt/coreutils/libexec/gnubin/sha256sum", "--version"], cwd=ci.ROOT, check=True)
