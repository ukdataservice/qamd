#!/bin/bash
sudo apt-get -y install update && sudo apt-get -y install upgrade

# Install the required tools
# required to obtain & compile source of both qamd & readstat of which qamd depends
sudo apt-get -y install git curl build-essential gcc autoconf libtool
# required for bindgen
sudo apt-get -y install llvm-3.9-dev libclang-3.9-dev clang-3.9

clone ()
{
	if [ ! -d "$1" ]; then
		# Control will enter here if $DIRECTORY exists.
		echo "Directory $1 missing, cloning: $2"
		git clone "$2"
	fi
}

mkdir $HOME/.src
cd $HOME/.src

# check for cargo (rust build tool)
if [ ! -x "$(command -v cargo)" ]; then
	# Install Rust
	curl https://sh.rustup.rs -sSf -o installrust.sh
	chmod +x installrust.sh
	yes "1" | sh installrust.sh
	rm ./installrust.sh
	
	# set path so we can find rustup, rustc & cargo.
	echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> $HOME/.profile
	source $HOME/.profile
fi

# Get the libraries
clone "$HOME/.src/ReadStat" "https://github.com/WizardMac/ReadStat.git"
clone "$HOME/.src/qamd" "https://github.com/Raymanns/qamd.git"

# Build Readstat
cd ReadStat
./autogen.sh
./configure
make
sudo make install

cd $HOME/.src
#rm -f $HOME/.src/ReadStat

# Add to .bashrc
echo 'export LD_LIBRARY_PATH="/usr/local/lib"' >> $HOME/.profile
source $HOME/.profile

# Build QA My Data
cd $HOME/.src/qamd
cargo build --release

echo 'export PATH="$HOME/.src/qamd/target/release:$PATH"' >> $HOME/.profile
source $HOME/.profile
