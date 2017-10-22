# mongoloid

[![Build Status](https://travis-ci.org/ancamcheachta/toghchain.svg?branch=master)](https://travis-ci.org/ancamcheachta/toghchain)
[![crates.io](https://img.shields.io/crates/v/mongoloid.svg)](https://crates.io/crates/mongoloid)

A CLI for building Irish election databases in MongoDB.

`mongoloid` is responsible for validating the Travis build of `toghchain`, and
is also the fastest and easiest way to get a working copy of an Irish election
database up and running on your own instance of MongoDB.

[Documentation](https://docs.rs/mongoloid)

## Requirements
* [Rust](https://www.rust-lang.org/en-US/install.html)
* [MongoDB](https://docs.mongodb.com/manual/installation/)

## Installation
1. `git clone https://github.com/ancamcheachta/toghchain.git`
2. `cd toghchain/src/mongoloid`
3. `cargo install`

## Usage
**Note**: prior to running `mongoloid`, `cd` into one of the `/toghchain`
directories containing election data (eg. `/dail`).

```bash
Usage:
    mongoloid [OPTIONS]

Mongoloid election database builder

optional arguments:
  -h,--help             show this help message and exit
  -d,--database DATABASE
                        name of database to build (optional)
```

## Links
* [`mongoloid` crate](https://crates.io/crates/mongoloid)
* [`mongoloid` documentation](https://docs.rs/mongoloid)
* [Toghcháin Éireann repository](https://github.com/ancamcheachta/toghchain)