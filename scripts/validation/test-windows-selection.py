#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import tempfile
import shutil
import subprocess
import unittest

spec = importlib.util.spec_from_file_location('windows_selection', Path(__file__).with_name('windows-selection.py'))
policy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(policy)


class WindowsSelection(unittest.TestCase):
    def test_current_owners_and_global_inputs_retain_native_checks(self):
        native, known = policy.dependency_closure(policy.ROOT)
        for package in native:
            self.assertTrue(policy.needs_windows([f'crates/{package}/src/lib.rs']))
        for package in known - native:
            self.assertFalse(policy.needs_windows([f'crates/{package}/src/lib.rs']))
        for path in ['Cargo.lock', 'Cargo.toml', 'rust-toolchain.toml', '.cargo/config.toml',
                     'crates/atlas-cli/Cargo.toml', 'unknown.config', '.gitattributes']:
            self.assertTrue(policy.needs_windows([path]))
        self.assertTrue(policy.needs_windows(['crates/atlas-cli/src/main.rs', 'crates/atlas-storage/src/lib.rs']))

    def test_workspace_aliases_target_optional_and_build_edges_follow_new_dependencies(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'Cargo.toml').write_text('[workspace]\n[workspace.dependencies]\nalias = { package = "new-storage", path = "crates/new" }\n')
            for name in ['atlas-storage', 'atlas-index', 'atlas-runtime', 'new', 'other']:
                path = root / 'crates' / name
                path.mkdir(parents=True)
                package = 'new-storage' if name == 'new' else name
                extra = '\n[target."cfg(windows)".build-dependencies]\nalias = { workspace = true, optional = true }\n' if name == 'atlas-index' else ''
                (path / 'Cargo.toml').write_text(f'[package]\nname = "{package}"\n{extra}')
            self.assertTrue(policy.needs_windows(['crates/new/src/lib.rs'], root))
            self.assertFalse(policy.needs_windows(['crates/other/src/lib.rs'], root))
            self.assertTrue(policy.needs_windows(['crates/not-yet-known/src/lib.rs'], root))

    def test_real_git_mode_and_symlink_changes_keep_the_shared_conservative_fallback(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            common = root / 'scripts/git-hooks/common.sh'
            common.parent.mkdir(parents=True)
            shutil.copyfile(policy.ROOT / 'scripts/git-hooks/common.sh', common)
            source = root / 'crates/atlas-cli/src/main.rs'
            source.parent.mkdir(parents=True)
            source.write_text('fixture')

            def git(*args, data=None):
                return subprocess.check_output(['git', *args], input=data, cwd=root).decode().strip()

            git('init', '--quiet')
            git('config', 'user.name', 'Native selection fixture')
            git('config', 'user.email', 'fixture@example.invalid')
            git('config', 'core.hooksPath', '/dev/null')
            git('config', 'commit.gpgsign', 'false')
            git('add', '.')
            git('commit', '--quiet', '-m', 'base')
            base = git('rev-parse', 'HEAD')
            source.write_text('ordinary source change')
            git('add', '.')
            git('commit', '--quiet', '-m', 'source')
            self.assertEqual(policy.changed_paths(base, root), ['crates/atlas-cli/src/main.rs'])
            self.assertFalse(policy.needs_windows(policy.changed_paths(base, root)))
            for mode in ['100755', '120000']:
                blob = git('hash-object', '-w', '--stdin', data=b'fixture-target')
                git('update-index', '--cacheinfo', f'{mode},{blob},crates/atlas-cli/src/main.rs')
                git('commit', '--quiet', '-m', f'mode {mode}')
                paths = policy.changed_paths(base, root)
                self.assertIn('__atlas_special_file_mode__', paths)
                self.assertTrue(policy.needs_windows(paths))


if __name__ == '__main__':
    unittest.main()
