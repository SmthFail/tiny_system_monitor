# Tiny system monitor (tsm)

A simple customizable cross-platform system monitor for terminal. \
Supports Linux and Windows. \
Inspired by [htop](https://github.com/htop-dev/htop), [bottom](https://github.com/ClementTsang/bottom), 

![Example](./thumbnail.png)

## Table of contents
- [Usage](#Usage)
- [Features](#Features)
- [Installaton](#Installation)
  - [Linux](#Linux)

## Usage
You can use tiny system monitor with `tsm`
- For run with config file use `tsm <config_file_name>`. Note that config file must be placed in `~/.config/tsm`
- For help use `tsm -h`

## Features:
- customizing widget behaviour;
- monitor cpu usage;
- monitor gpu usage (only Nvidia for now)

## Installation

### From source
Clone project and use cargo for buid 
> cargo run

### Download latest version from releases

Linux
```
curl -fsSL https://raw.githubusercontent.com/SmthFail/tiny_system_monitor/main/install.sh | bash -s linux
```
For running from everywhere update rcfile (.bashrc, .zshrc etc)
> export PATH=$PATH:~/.tsm/bin

Windows
Download binary from release and install it. Script install not tested yet

### Use config file
Example of config file can be found in repo/config_example/cats.json \
Note that "symbol" field must be placed in config but didn't used (yet) \
More description will be added soon
