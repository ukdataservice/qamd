#!/bin/bash
sudo apt-get -y install update && sudo apt-get -y install upgrade

# Install the required tools
# required to obtain & compile source of both qamd & readstat of which qamd depends
sudo apt-get -y install git curl build-essential gcc autoconf libtool
# required for bindgen
sudo apt-get -y install llvm-3.9-dev libclang-3.9-dev clang-3.9

