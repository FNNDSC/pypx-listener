# pypx-listener

[![MIT License](https://img.shields.io/github/license/fnndsc/pypx-listener)](https://github.com/FNNDSC/pypx-listener/blob/main/LICENSE)
[![Version](https://img.shields.io/docker/v/fnndsc/pypx-listener?sort=semver)](https://hub.docker.com/r/fnndsc/pypx-listener)
[![CI](https://github.com/FNNDSC/pypx-listener/actions/workflows/ci.yml/badge.svg)](https://github.com/FNNDSC/pypx-listener/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/FNNDSC/pypx-listener/branch/master/graph/badge.svg?token=1YQRZWW95S)](https://codecov.io/gh/FNNDSC/pypx-listener)

An instance handler for `storescp` which reorganizes incoming DICOM files.
Rust re-write of `px-repack` from [pypx](https://github.com/FNNDSC/pypx).

## Testing

Get data:

```shell
./get_examples.sh examples
```

Run tests:

```shell
cargo test
```

## _Blazingly Fast!_

`rx-repack` runs faster than the rate at which `storescp` receives DICOM instances,
so it is unlikely to have a resource deficit.

### Benchmarking: Setup

```shell
./get_examples.sh examples
cargo build --release
```

### Benchmarking: v.s. `px-repack`

`rx-repack` called per-instance is ~10x faster than `px-repack` called per-series.

```shell
source examples/pypx/bin/activate
export PATH="$PWD/target/release:$PATH"
hyperfine --prepare 'rm -rf /tmp/dicom' \
          "px-repack --xcrdir $PWD/examples/FNNDSC-SAG-anon-3d6e850 --parseAllFilesWithSubStr , --verbosity 0 --datadir /tmp/dicom" \
"fd --exec rx-repack --xcrdir '{//}' --xcrfile '{/}' --verbosity 0 --datadir /tmp/dicom \; --threads=1 --type f '.*\.dcm$' $PWD/examples/FNNDSC-SAG-anon-3d6e850"
```

However, for a more direct comparison, we need to call `px-repack` and `rs-repack`
both exactly the same way: one process per series, sequential execution.

```shell
source examples/pypx/bin/activate
export PATH="$PWD/target/release:$PATH"
hyperfine --prepare 'rm -rf /tmp/dicom' -L xx-repack px-repack,rx-repack \
   "fd --exec {xx-repack} --xcrdir '{//}' --xcrfile '{/}' --verbosity 0 --datadir /tmp/dicom \; \
   --threads=1 --type f '.*\.dcm$' $PWD/examples/FNNDSC-SAG-anon-3d6e850"
```

### Benchmarking: Results

The performance of `rs-repack` is **200 times faster** than `px-repack`.

```
Benchmark 1: fd --exec px-repack --xcrdir '{//}' --xcrfile '{/}' --verbosity 0 --datadir /tmp/dicom \;    --threads=1 --type f '.*\.dcm$' /home/jenni/fnndsc/pypx-listener/examples/FNNDSC-SAG-anon-3d6e850
  Time (mean ± σ):     41.000 s ±  0.230 s    [User: 35.948 s, System: 4.861 s]
  Range (min … max):   40.669 s … 41.473 s    10 runs
 
Benchmark 2: fd --exec rx-repack --xcrdir '{//}' --xcrfile '{/}' --verbosity 0 --datadir /tmp/dicom \;    --threads=1 --type f '.*\.dcm$' /home/jenni/fnndsc/pypx-listener/examples/FNNDSC-SAG-anon-3d6e850
  Time (mean ± σ):     204.6 ms ±   5.6 ms    [User: 149.0 ms, System: 53.5 ms]
  Range (min … max):   198.9 ms … 216.7 ms    13 runs
 
Summary
  fd --exec rx-repack --xcrdir '{//}' --xcrfile '{/}' --verbosity 0 --datadir /tmp/dicom \;    --threads=1 --type f '.*\.dcm$' /home/jenni/fnndsc/pypx-listener/examples/FNNDSC-SAG-anon-3d6e850 ran
  200.35 ± 5.58 times faster than fd --exec px-repack --xcrdir '{//}' --xcrfile '{/}' --verbosity 0 --datadir /tmp/dicom \;    --threads=1 --type f '.*\.dcm$' /home/jenni/fnndsc/pypx-listener/examples/FNNDSC-SAG-anon-3d6e850
```

Amusingly, `rx-repack` runs ~12x faster than it takes Python 3.11.3 to do _literally nothing_.

```shell
hyperfine -N "python -c ''" "target/release/rx-repack --xcrdir $PWD/examples/FNNDSC-SAG-anon-3d6e850 --xcrfile 0002-1.3.12.2.1107.5.2.19.45152.2013030808110261698786039.dcm  --datadir /tmp/dicom"
```
