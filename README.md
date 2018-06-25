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

#### Linux & MacOS

Before starting, I like to make a director `~/.src` to contain source files for
building from.

```
mkdir ~/.src
cd ~/.src
```

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
cargo build --release
echo 'export PATH="$PATH:$HOME/.src/target/release"' | cat >> ~/.profile
```

#### Windows

Comming soon.

## Running the tests

To run the unit & documentation tests run,

`cargo test`

## Authors

* **Myles Offord**

See also the list of [contributors](https://github.com/raymanns/qamd/contributors)
who participated in this project.

## License

Licenced under the Creative Commons Attribution-NonCommercial 4.0 International (CC BY-NC 4.0).

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

