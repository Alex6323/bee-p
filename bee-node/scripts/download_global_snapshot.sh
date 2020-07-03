#!/bin/bash

state_url="https://github.com/iotaledger/iri/raw/dev/src/main/resources/snapshotMainnet.txt"
sig_url="https://github.com/iotaledger/iri/raw/dev/src/main/resources/snapshotMainnet.sig"
data_folder_path=$(git rev-parse --show-toplevel)/bee-node/data

mkdir -p $data_folder_path
wget $state_url -O $data_folder_path/global_snapshot_state.txt -q --show-progress
wget $sig_url -O $data_folder_path/global_snapshot_sig.txt -q --show-progress
