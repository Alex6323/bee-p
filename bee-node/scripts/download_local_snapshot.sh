#!/bin/bash

url="https://dbfiles.iota.org/mainnet/hornet/latest-export.lite.bin"
data_folder_path=$(git rev-parse --show-toplevel)/bee-node/data

mkdir -p $data_folder_path
wget $url -O $data_folder_path/local_snapshot.bin -q --show-progress
