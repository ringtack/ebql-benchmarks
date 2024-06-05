#!/bin/bash

echo "//<--------------------------------------------------------------"
echo "Building rust assets"
echo "-------------------------------------------------------------->//"

cargo build --release \
  -p rocksdb-application \
  -p ebpf-probe-pread
  # -p bpf-stats-collector \


echo "//<--------------------------------------------------------------"
echo "Done Building"
echo "-------------------------------------------------------------->//"
