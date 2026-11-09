#!/usr/bin/env python3
"""Gera cena local, executa COLMAP via NOAH e verifica proveniência/exports."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

parser=argparse.ArgumentParser()
parser.add_argument('--colmap',required=True,type=Path)
parser.add_argument('--output',required=True,type=Path,help='diretório novo para fotos e run')
parser.add_argument('--dense',action='store_true')
args=parser.parse_args()
repo=Path(__file__).resolve().parents[2]
binary=repo/'target/debug/noah'
args.output.mkdir(parents=True,exist_ok=False)
images=args.output/'images'
subprocess.run([sys.executable,str(repo/'scripts/e2/generate_scene.py'),str(images)],check=True)
original={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in images.glob('*.png')}
run=args.output/'run'
command=[str(binary),'reconstruct',str(images),str(run),'--colmap',str(args.colmap.resolve()),'--timeout','180']
if args.dense:command.append('--dense')
result=subprocess.run(command,check=False)
subprocess.run([str(binary),'run-inspect',str(run)],check=True)
manifest=json.loads((run/'run.json').read_text())
for image in manifest['inputs']:
    assert original[image['original_name']]==image['sha256']
    assert hashlib.sha256((images/image['original_name']).read_bytes()).hexdigest()==image['sha256']
if result.returncode:
    print(f'Run falhou; diagnóstico preservado em {run}',file=sys.stderr)
    sys.exit(result.returncode)
reconstruction=json.loads((run/'exports/reconstruction.json').read_text())
assert reconstruction['unit']=='unknown'
assert reconstruction['frame']=='arbitrary-right-handed-sfm'
assert reconstruction['registered_images']>=8
assert reconstruction['point_count']>0
lines=(run/'exports/sparse.ply').read_text().splitlines()
assert len(lines)-lines.index('end_header')-1==reconstruction['point_count']
assert manifest['status']=='succeeded'
print(f"Demo passou: {reconstruction['registered_images']}/12 imagens, {reconstruction['point_count']} pontos; originais preservados")
