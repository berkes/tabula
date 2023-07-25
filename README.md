# Tabula

Tools to manage beancount accounting in an ergnomic and opinionated way, for small businesses.

## invoices

`tabula invoices list`

Renders an overview of all the invoices you sent.

`tabula invoices create`

Generates a ledger entry for an invoice from arguments passed in.

## Quickstart

Requirements:

* cargo (TODO: set up CI to build prebuild packages for target platforms)


### Install

TODO: Once MVP is done, deploy to crates so we can actually download and install it.

### Run

    tabula

This shows the commands and subcommands available.

### Test

After installing the dependencies, on the development machine, run

    cargo test --all

This builds and runs the tests locally.

### Release

After finishing the changes, a release can be prepared with

    cargo build --release
