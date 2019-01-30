# æternity middleware

## Overview

This is a caching layer for Epoch. It reads the chain and records key- and micro-blocks, and transactions in a PostgreSQL database.

## How to use

- Install a postgresql DB somewhere. Works with versions >= 9.5, at least.
- as the admin user, execute `scripts/prepare-db.sql` which will create the DB and user
- copy 'Rocket.example.toml' to 'Rocket.toml'
- if you want to use a different DB name, edit `scripts/prepare-db.sql`, `.env` and `Rocket.toml`

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

NB: The diesel framework causes many many compiler warnings. It can be good to suppress them with
`export RUSTFLAGS="-Aproc-macro-derive-resolution-fallback"`so that you can see what's actually going on.

## How to run

`cargo run -- ` + flags below

```
FLAGS:
        --help        Prints help information
    -p, --populate    Populate DB
    -s, --server      Start server
    -v, --verify      Check the DB against the chain
    -V, --version     Prints version information

OPTIONS:
    -h, --start <START_HASH>    Hash to start from.
    -u, --url <URL>             URL of æternity node.

```

## Supported queries
```
GET /middleware/transactions/account/<account>?<limit>&<page>
GET /middleware/transactions/interval/<from>/<to>?<limit>&<page>
GET /middleware/transactions/account/<account>/count
GET /middleware/contracts/transactions/address/<address>
GET /v2/generations/current
GET /v2/key-blocks/current/height
GET /api (epoch_api_handler)
GET /v2/generations/height/<height>
GET /v2/key-blocks/height/<height>
GET /v2/key-blocks/hash/<hash>
GET /v2/micro-blocks/hash/<hash>/header
GET /v2/transactions/<hash>
GET /v2/micro-blocks/hash/<hash>/transactions/count
GET /v2/micro-blocks/hash/<hash>/transaction
```
