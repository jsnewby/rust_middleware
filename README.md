# rust-swagger

## Overview

This is a first implementation of an interface between Epoch using Rust, and a simple cache for the æternity blockchain, in order to provide various services which are useful, yet missing from Epoch.

## How to use

- Install a postgresql DB somewhere. Works with versions >= 9.5, at least.
- as the admin user, execute `scripts/prepare-db.sql` which will create the DB and user
- if you want to use a different DB name, edit `scripts/prepare-db.sql` and `.env`

## How to build

`cargo build`

## How to run

```
FLAGS:
        --help        Prints help information
    -p, --populate    Populate DB
    -s, --server      Start server
    -V, --version     Prints version information

OPTIONS:
    -h, --start <START_HASH>    Hash to start from.
    -u, --url <URL>             URL of æternity node.

```





