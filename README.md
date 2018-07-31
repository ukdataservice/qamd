# QAMyData

QAMyData offers a free easy-to-use tool that automatically detects some of
the most common problems in survey and other numeric data and creates a
‘data health check’, assisting with the clean up of data and providing an
assurance that data is of a high quality.

## Getting Started

## Installing

### Prerequisites

The following things are required to build the software from source:

- [Git], to clone the repository
- [Rust], toolchain including the rust compiler (`rustc`) & the `cargo` build system
- [ReadStat], master branch, installed from source & `LD_LIBRARY_PATH` set correctly!

#### Linux

If you are on Ubuntu, install run the [`setup.sh`](./setup.sh) script. You may needed to
run `chmod +x setup.sh`.

First, install ReadStat. This is done by cloning the repository from github,
and then a standard automake installation. This requires sudo/root
permission to install.

```
git clone https://github.com/WizardMac/ReadStat.git
cd ReadStat
./autogen.sh
./configure
make
sudo make install
```

Make sure to set `LD_LIBRARY_PATH` to `/usr/local/lib` by adding running the
following commands. This only needs to be done once per install.

```
echo 'export LD_LIBRARY_PATH="/usr/local/lib"' | cat >> ~/.profile
source ~/.profile
```

Next clone the `qamd` repository:

```
git clone https://github.com/Raymanns/qamd.git

cd ./qamd
cargo build --release

echo 'export PATH="$PATH:$HOME/.src/target/release"' | cat >> ~/.profile
```

#### Windows

First install the [linux subsystem for windows]. `qamd` currently
requires Ubunut 16.04. Start it and you will be prompted to create
a new user (this can be different or the same as your windows account).

After this you will be presented with a bash prompt. Run the following:

```
sudo apt-get update && sudo apt-get upgrade
```

Next, open a command prompt (type Win+R, type `'cmd'` and press enter)
and change to the directory you downloaded the install.bat. Run the
`install.bat` and place `qamd.bat` somewhere and add it to your PATH
environment variable. The install script can take some time.

## Running the tests

To run the unit & documentation tests run,

`cargo test`

## Authors

* **Myles Offord**

See also the list of [contributors](https://github.com/raymanns/qamd/contributors)
who participated in this project.

## License

See LICENCE.md for the full license.

<a rel="license" href="http://creativecommons.org/licenses/by-nc/4.0/">
  <img alt="Creative Commons License" style="border-width:0" src="https://i.creativecommons.org/l/by-nc/4.0/88x31.png" /></a>
  <br />
  This work is licensed under a <a rel="license" href="http://creativecommons.org/licenses/by-nc/4.0/">Creative Commons Attribution-NonCommercial 4.0 International License
</a>.

## Acknowledgments

* WizardMac for the amazing [ReadStat] C library

[Git]: https://git-scm.com/
[Rust]: https://rust-lang.org/
[ReadStat]: https://github.com/WizardMac/ReadStat
[linux subsystem for windows]: https://docs.microsoft.com/en-us/windows/wsl/install-win10

