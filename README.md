# rstracer

[![codecov](https://codecov.io/github/VictorMeyer77/rstracer/graph/badge.svg?token=MCO1XZI4OO)](https://codecov.io/github/VictorMeyer77/rstracer)
[![CI](https://github.com/VictorMeyer77/rstracer/actions/workflows/ci.yml/badge.svg)](https://github.com/VictorMeyer77/rstracer/actions/workflows/ci.yml)

**A UNIX-based system monitoring tool built with Rust and DuckDB.**

## Table of Contents

1. [About the Project](#about-the-project)
2. [Features](#features)
3. [Prerequisites](#prerequisites)
4. [Installation](#installation)
5. [Usage](#usage)
6. [Configuration](#configuration)
7. [Limitations](#limitations)

## About the Project

**rstracer** is a system monitoring tool that analyzes UNIX system activity
using the `ps`, `lsof`, and network packet commands.
Output data from these commands are stored in DuckDB for complex querying
with high performance.

The database follows a "medallion" architecture and can operate either "in-memory"
or using a file-based database.
In-memory mode is highly performant but does not retain data beyond the process runtime,
whereas file mode stores data for post-run analysis at a slight performance cost.

## Features

- Monitor system processes, open files, and network activity.
- Uses DuckDB for efficient querying and modern data modeling.
- Supports both in-memory and file-based database storage.
- Offers customizable configuration options.

## Prerequisites

- **Cargo** is required to build the project.
- **libpcap-dev** is required on Linux for network packet analysis.

```shell
sudo apt-get update
sudo apt-get install libpcap-dev
```

No additional libraries are required on macOS.

## Installation

**rstracer** is not yet available on [crates.io](https://crates.io/).

To install directly from GitHub:

```shell
git clone https://github.com/VictorMeyer77/rstracer.git
cd rstracer
cargo build --release
```

## Usage

Due to network analysis capabilities, the binary must be run
with administrative permissions:

```shell
sudo target/release/rstracer
```

## Configuration

A default configuration is automatically applied if a `rstracer.toml` file
is not found in the working directory.

Refer to the [rstracer.toml](rstracer.toml) file for customizable options.

## Limitations

1. **System Language**: The `ps` command date parsing requires the system
language to be set to English.
2. **In-Memory Database Querying**: In-memory mode currently does not
support external database queries.
3. **Platform**: Only available for UNIX-based systems.
