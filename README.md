# æternity middleware

## Overview

This is a first implementation of a caching layer for Epoch. It reads the chain and records key- and micro-blocks, and transactions in a PostgreSQL database. 

## How to use

- Install a postgresql DB somewhere. Works with versions >= 9.5, at least.
- as the admin user, execute `scripts/prepare-db.sql` which will create the DB and user
- if you want to use a different DB name, edit `scripts/prepare-db.sql` and `.env`

## How to build

You need a nightly rust build

`rustup default nightly`

then

`cargo build`

and to install the database

```
cargo install diesel_cli
diesel database reset
```


## How to run

`cargo run -- ` + flags below

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

## Supported queries

`GET /transactions/account/<account>` all transactioms for account



