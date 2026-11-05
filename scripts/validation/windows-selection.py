#!/usr/bin/env python3
"""Select native file/publication checks from their workspace dependency closure."""
import argparse
from pathlib import Path
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]


def dependency_closure(root):
    workspace = tomllib.loads((root / 'Cargo.toml').read_text())['workspace']
    shared = workspace.get('dependencies', {})
    packages = {}
    for path in (root / 'crates').glob('*/Cargo.toml'):
        data = tomllib.loads(path.read_text())
        packages[data['package']['name']] = (path.parent.name, data)
    selected = set()

    def visit(name):
        if name in selected or name not in packages:
            return
        selected.add(name)
        data = packages[name][1]
        for scope in [data, *data.get('target', {}).values()]:
            for kind in ['dependencies', 'dev-dependencies', 'build-dependencies']:
                for alias, spec in scope.get(kind, {}).items():
                    if isinstance(spec, dict) and spec.get('workspace'):
                        spec = shared[alias]
                    visit(spec.get('package', alias) if isinstance(spec, dict) else alias)

    for name in ['atlas-storage', 'atlas-index', 'atlas-runtime']:
        if name not in packages:
            raise ValueError(f'Missing native check owner: {name}')
        visit(name)
    return {packages[name][0] for name in selected}, {row[0] for row in packages.values()}


def needs_windows(paths, root=ROOT):
    native, known = dependency_closure(root)
    for path in paths:
        parts = Path(path).parts
        if path.endswith('/Cargo.toml') or path in {'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.gitattributes'}:
            return True
        if len(parts) >= 3 and parts[0] == 'crates' and parts[1] in known:
            if parts[1] in native:
                return True
        else:
            # Global/unknown inputs retain the conservative native gate.
            return True
    return False


def changed_paths(base, root=ROOT):
    with tempfile.TemporaryDirectory(prefix='atlas-native-selection-') as directory:
        paths = Path(directory) / 'paths'
        subprocess.run(['sh', '-c', '. "$1/scripts/git-hooks/common.sh"; collect_changed_paths "$2" "$3"',
                        'sh', str(root), str(paths), f'{base}...HEAD'], cwd=root, check=True)
        return paths.read_text().splitlines()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', default='')
    args = parser.parse_args()
    paths = changed_paths(args.base) if args.base else None
    print('true' if paths is None or needs_windows(paths) else 'false')
