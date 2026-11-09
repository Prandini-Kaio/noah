#!/usr/bin/env python3
"""Demo E1: execute cargo build --locked antes. Usa apenas Python padrão."""
import hashlib
import math
from pathlib import Path
import subprocess
import sys
import tempfile

repo = Path(__file__).resolve().parents[1]
binary = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else repo / 'target/debug/noah'
root = Path(tempfile.mkdtemp(prefix='noah-e1-demo-'))
project = root / 'project'
source = repo / 'tests/datasets/reference-rgb.ply'
original = hashlib.sha256(source.read_bytes()).hexdigest()

def run(*args):
    result = subprocess.run([str(binary), *map(str, args)], check=True, text=True, capture_output=True)
    print(f'$ noah {" ".join(map(str, args))}')
    print(result.stdout.rstrip())
    return result.stdout

run('create', project, 'Referência E1')
identity = run('import', project, source, 'Peça RGB', '--frame', 'z-up').strip().split('=', 1)[1]
run('transform', project, identity, 3, 4, 5, 0, 0, math.pi / 2, 2)
run('inspect', project)
run('validate', project)
export = root / 'transformed.ply'
run('export', project, identity, export)
lines = export.read_text().splitlines()
rows = [line.split() for line in lines[lines.index('end_header') + 1:]]
for row, expected in zip(rows, [(3, 6, 9), (1, 4, 5), (3, 4, 5)], strict=True):
    assert all(abs(float(a) - b) < 1e-12 for a, b in zip(row[:3], expected, strict=True))
assert [row[3:] for row in rows] == [['255', '0', '0'], ['0', '255', '0'], ['0', '0', '255']]
assert hashlib.sha256(source.read_bytes()).hexdigest() == original
assert hashlib.sha256(next((project / 'raw').glob('*.ply')).read_bytes()).hexdigest() == original
print(f'Demo passou: XYZ/RGB conferidos, origem preservada. Dados em {root}')
