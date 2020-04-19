#!/bin/bash

data_folder_path=$(git rev-parse --show-toplevel)/bee-node/data

mkdir -p $data_folder_path/.tmp

wget "https://dbfiles.iota.org/mainnet/iri/latest-LS.tar" -P $data_folder_path/.tmp
tar xf $data_folder_path/.tmp/latest-LS.tar --directory $data_folder_path/.tmp

mv $data_folder_path/.tmp/mainnet.snapshot.meta $data_folder_path/mainnet.snapshot.meta
mv $data_folder_path/.tmp/mainnet.snapshot.state $data_folder_path/mainnet.snapshot.state

rm -r $data_folder_path/.tmp
