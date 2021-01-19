# BEEFY
**BEEFY** (**B**ridge **E**fficiency **E**nabling **F**inality **Y**ielder) is a secondary
protocol running along Grandpa Finality to support efficient bridging with non-Substrate
blockchains, currently mainly ETH mainnet.

It can be thought of as an (optional) Bridge-specific Gadget to the Grandpa Finality protocol.
The Protocol piggybacks on many assumptions provided by Grandpa, and is required to be built
on top of it to work correctly.

🚧 BEEFY is currently under construction - a hardhat is recommended beyond this point 🚧

## Contents
- [Build](#build)
- [Documentation](#documentation)
- [Project Layout](#project-layout)
- [Running BEEFY](#running-beefy)

## Build
To get up and running you need both stable and nightly Rust. Rust nightly is used to build the Web
Assembly (WASM) runtime for the node. You can configure the WASM support as so:

```
rustup install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Once this is configured you can build and test the repo as follows:

```
git clone https://https://github.com/paritytech/grandpa-bridge-gadget.git
cd grandpa-bridge-gadget
cargo build --all
cargo test --all
```

If you need more information about setting up your development environment Substrate's
[Getting Started](https://substrate.dev/docs/en/knowledgebase/getting-started/) page is a good
resource.

## Documentation

The best way to get going with BEEFY is by reading the [walkthrough](./docs/walkthrough.md) document!
This document puts BEEFY into context and provides motivation for why this project has been started.
In addition to that the current status as well as a preliminary roadmap is presented.

[beefy](./docs/beefy.md) is a collection of early notes and ideas, stil worth checking out.

## Project Layout

What follows is an overview of how the project repository is laid out. The main components are the
`beefy-gadget` which is a POC of the BEEFY round logic. The BEEFY `pallet` which is mainly a thin
integration layer over the session pallet and keeps track of the current authorities.
Finally the BEEFY `primitives` crate which contains most of the type definitions for the 
BEEFY protocol.

The `primitives` crate also contains a test [light_client](.primitives/tests/light_client/) which demonstarates how BEEFY would
be utilized by a light client implementation.

```
├── beefy-gadget  // The BEEFY gadget
│  └── ...
├── docs          // Documentation
│  └──  ...
├── node-example  // A Substrate node running the BEEFY gadget
│  └──  ...
├── pallet        // The BEEFY pallet.
│  └──  ...
├── primitives    // The BEEFY primitives crate includig a test light client
│  └──  ...
 ```

## Running BEEFY

Currently the easiest way to see BEEFY in action is to run a single dev node like so:

```
$ RUST_LOG=beefy=trace ./target/debug/node-template --tmp --dev --alice --validator
```

Expect additional (more usefull) deployment options to be added soon.