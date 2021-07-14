# BEEFY
**BEEFY** (**B**ridge **E**fficiency **E**nabling **F**inality **Y**ielder) is a secondary
protocol running along GRANDPA Finality to support efficient bridging with non-Substrate
blockchains, currently mainly ETH mainnet.

It can be thought of as an (optional) Bridge-specific Gadget to the GRANDPA Finality protocol.
The Protocol piggybacks on many assumptions provided by GRANDPA, and is required to be built
on top of it to work correctly.

🚧 BEEFY is currently under construction - a hardhat is recommended beyond this point 🚧

## Contents
- [Build](#build)
- [Documentation](#documentation)
- [Project Layout](#project-layout)
- [BEEFY Key](#beefy-key)
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

[BEEFY brainstorming](./docs/beefy.md) is a collection of early notes and ideas, still worth checking out.

## Project Layout

What follows is an overview of how the project repository is laid out. The main components are the
`beefy-gadget` which is a POC of the BEEFY round logic. `beefy-pallet` which is mainly a thin
integration layer over the session pallet and keeps track of the current authorities.
Finally the BEEFY `primitives` crate which contains most of the type definitions for the
BEEFY protocol.

The `primitives` crate also contains a test [light_client](.primitives/tests/light_client/) which demonstrates how BEEFY would
be utilized by a light client implementation.

```
├── beefy-cli         // BEEFY utilities and testing aids
│  └── ...
├── beefy-gadget      // The BEEFY gadget
│  └── ...
├── beefy-merkle-tree // A Binary Merkle-Tree for Substrate runtime usage
│  └──  ...
├── beefy-mmr-pallet  // BEEFY and Merkle Moutain Range (MMR) together in one pallet
│  └──  ...
├── beefy-node        // A Substrate node running the BEEFY gadget
│  └──  ...
├── beefy-pallet      // The BEEFY pallet.
│  └──  ...
├── beefy-primitives  // The BEEFY primitives crate includig a test light client
│  └──  ...
├── beefy-test        // The BEEFY test support library
│  └──  ...
├── docs              // Documentation
│  └──  ...
 ```

## BEEFY Key

The current cryptographic scheme used by BEEFY is `ecdsa`. This is **different** from other schemes like `sr25519` and `ed25519` which are commonly used in Substrate configurations for other pallets (BABE, GRANDPA, AuRa, etc). The most noticeable difference is that an `ecdsa` public key
is `33` bytes long, instead of `32` bytes for a `sr25519` based public key. So, a BEEFY key [sticks out](https://github.com/paritytech/polkadot/blob/25951e45b1907853f120c752aaa01631a0b3e783/node/service/src/chain_spec.rs#L738) among the other public keys a bit.

For other crypto (using the default Substrate configuration) the `AccountId` (32-bytes) matches the `PublicKey`, but note that it's not the case for BEEFY. As a consequence of this, you can **not** convert the `AccountId` raw bytes into a BEEFY `PublicKey`.

The easiest way to generate or view hex-encoded or SS58-encoded BEEFY Public Key is by using the [Subkey](https://substrate.dev/docs/en/knowledgebase/integrate/subkey) tool. Generate a BEEFY key using the following command

```sh
subkey generate --scheme ecdsa
```

The output will look something like

```sh
Secret phrase `sunset anxiety liberty mention dwarf actress advice stove peasant olive kite rebuild` is account:
  Secret seed:       0x9f844e21444683c8fcf558c4c11231a14ed9dea6f09a8cc505604368ef204a61
  Public key (hex):  0x02d69740c3bbfbdbb365886c8270c4aafd17cbffb2e04ecef581e6dced5aded2cd
  Public key (SS58): KW7n1vMENCBLQpbT5FWtmYWHNvEyGjSrNL4JE32mDds3xnXTf
  Account ID:        0x295509ae9a9b04ade5f1756b5f58f4161cf57037b4543eac37b3b555644f6aed
  SS58 Address:      5Czu5hudL79ETnQt6GAkVJHGhDQ6Qv3VWq54zN1CPKzKzYGu

```

In case your BEEFY keys are using the wrong cryptographic scheme, you will see an invalid public key format message at node startup. Basically something like

```sh
...
2021-05-28 12:37:51  [Relaychain] Invalid BEEFY PublicKey format!
...
```

## Running BEEFY

Currently the easiest way to run BEEFY is to use a 3-node local testnet using `beefy-node`. We will call those nodes `Alice`, `Bob` and
`Charlie`. Each node will use the built-in development account with the same name, i.e. node `Alice` will use the `Alice` development
account and so on. Each of the three accounts has been configured as an initial authority at genesis. So, we are using three validators
for our testnet.

`Alice` is our bootnode is is started like so:

```
$ RUST_LOG=beefy=trace ./target/debug/beefy-node --tmp --alice
```

`Bob` is started like so:

```
RUST_LOG=beefy=trace ./target/debug/beefy-node --tmp --bob
```

`Charlie` is started like so:

```
RUST_LOG=beefy=trace ./target/debug/beefy-node --tmp --charlie
```

Note that the examples above use an ephemeral DB due to the `--tmp` CLI option. If you want a persistent DB, use `--/tmp/[node-name]`
instead. Replace `node-name` with the actual node name (e.g. `alice`) in order to assure separate dirctories for the DB.
