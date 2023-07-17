#!/bin/bash

if [ -z "$1" ]; then
  echo "usage: $0 <data_directory>"
  exit 1
fi

set -ex

mkdir -vp "$1"
cd "$1"
curl -fL https://github.com/FNNDSC/SAG-anon/tarball/master | tar -xz

python -m venv pypx
source pypx/bin/activate
pip install pypx==3.10.16

exec px-repack --xcrdir ./FNNDSC-SAG-anon-* --parseAllFilesWithSubStr , \
     --datadir px-repack-output/data --logdir px-repack-output/log      \
     --verbosity 0
