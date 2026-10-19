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
