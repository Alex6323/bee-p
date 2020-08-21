#!/bin/bash

url="https://dbfiles.iota.org/mainnet/hornet/latest-export.bin"
data_folder_path=$(git rev-parse --show-toplevel)/bee-node/snapshots/mainnet

mkdir -p $data_folder_path
wget $url -O $data_folder_path/export.bin -q --show-progress
