#!/usr/bin/env python3
"""Cena sintética CC0: superfícies texturizadas, câmera pinhole e oclusão por profundidade."""
import argparse
import json
from pathlib import Path
import random
import numpy as np
from PIL import Image, ImageDraw

parser=argparse.ArgumentParser()
parser.add_argument('output',type=Path)
args=parser.parse_args()
args.output.mkdir(parents=True,exist_ok=False)
width,height,focal=800,600,700.
rng=random.Random(20261109)
def texture():
    img=Image.new('RGB',(1024,1024),(190,180,160));draw=ImageDraw.Draw(img)
    for _ in range(2400):
        x,y=rng.randrange(1024),rng.randrange(1024);size=rng.randrange(3,26)
        color=tuple(rng.randrange(25,240) for _ in range(3))
        if rng.random()<.5:draw.rectangle((x,y,x+size,y+size),fill=color)
        else:draw.ellipse((x,y,x+size,y+size),fill=color)
    return np.asarray(img)
planes=[
    ([-2,-1.6,-.8],[4,0,0],[0,3.2,0]),
    ([-2,1.1,-.8],[4,0,0],[0,0,2.3]),
    ([-.45,-.35,-.8],[.9,0,0],[0,0,1.]),
    ([-.45,-.35,.2],[.9,0,0],[0,.9,0]),
    ([.45,-.35,-.8],[0,.9,0],[0,0,1.]),
    ([-.45,-.35,-.8],[0,.9,0],[0,0,1.]),
]
planes=[(np.array(o,float),np.array(u,float),np.array(v,float),texture()) for o,u,v in planes]
y,x=np.mgrid[0:height,0:width]
local=np.stack([(x-width/2)/focal,(y-height/2)/focal,np.ones_like(x)],axis=-1)
truth=[]
for i in range(12):
    center=np.array([-.75+1.5*i/11,-2.8,.65+.12*np.cos(i*.4)])
    forward=np.array([0,.2,-.1])-center;forward/=np.linalg.norm(forward)
    right=np.cross(forward,[0,0,1.]);right/=np.linalg.norm(right)
    down=np.cross(forward,right)
    rotation=np.stack([right,down,forward])
    rays=local@rotation
    image=np.full((height,width,3),225,dtype=np.uint8)
    depth=np.full((height,width),np.inf)
    for origin,u,v,tex in planes:
        normal=np.cross(u,v);denom=rays@normal
        with np.errstate(divide='ignore',invalid='ignore'):
            t=((origin-center)@normal)/denom
            point=center+rays*t[...,None]-origin
            a=(point@u)/(u@u);b=(point@v)/(v@v)
        mask=(t>0)&(t<depth)&(a>=0)&(a<=1)&(b>=0)&(b<=1)
        aa=np.clip(np.nan_to_num(a),0,1);bb=np.clip(np.nan_to_num(b),0,1)
        image[mask]=tex[(bb[mask]*1023).astype(int),(aa[mask]*1023).astype(int)]
        depth[mask]=t[mask]
    name=f'view-{i:02}.png';Image.fromarray(image).save(args.output/name)
    truth.append({'image':name,'world_to_camera_rotation':rotation.tolist(),'translation':(-rotation@center).tolist()})
(args.output/'truth.json').write_text(json.dumps({'seed':20261109,'width':width,'height':height,'focal':focal,'views':truth},indent=2))
print(f'12 imagens sintéticas geradas em {args.output}')
