# BEEFY

Related issues:
- [Merkle Mountain Range for Efficient Bridges](https://github.com/paritytech/parity-bridges-common/issues/263)
- [Grandpa Super Light Client](https://github.com/paritytech/parity-bridges-common/issues/323)

BEEFY is a consensus protocol designed with efficient trustless bridging in mind. It means
that building a light client of BEEFY protocol should be optimized for restricted environments
like Ethereum Smart Contracts or On-Chain State Transition Function (e.g. Substrate Runtime).
Note that BEEFY is not a standalone protocol, it is meant to be running alongside GRANDPA, a
finality gadget created for Substrate/Polkadot ecosystem. More details about GRANDPA can be found in
the [whitepaper](https://github.com/w3f/consensus/blob/master/pdf/grandpa.pdf).

Following document is a semi-formal description of the protocol augmented with high-level
implementation suggestions and notes. Any changes to the implementation in Substrate repository
should be preceded by updates to this document.

Current Version: 0.1.0

# Introduction

The original motiviation for introducing BEEFY is stemming from inefficiencies of building
GRANDPA light client in restricted environments.

1. GRANDPA uses `ed25519` signatures and finality proof requires `2N/3 + 1` of valid signatures
   (where `N` is the size of current validator set).
1. GRANDPA finalizes `Headers`, which by default in Substrate is at least `100 bytes` (3 hashes +
   block number). Additionally the finality proof may contain `votes_ancestries` which are also
   headers, so the total size is inflated by that. This extraneuous data is useless for the light
   client though.
1. Since GRANDPA depends on the header format, which is customizable in Substrate, it makes it hard
   to build a "Generic Light Client" which would be able to support multiple different chains.

Hence the goals of BEEFY are:

1. Allow customisation of crypto to adapt for different targets. Support thresholds signatures as
   well eventually.
1. Minimize the size of the "signed payload" and the finality proof.
1. Unify data types and use backward-compatible versioning so that the protocol can be extended
   (additional payload, different crypto) without breaking existing light clients.

BEEFY is required to be running on top of GRANDPA. This allows us to take couple of shortcuts:
1. BEEFY validator set is **the same** as GRANDPA's (i.e. the same bonded actors), they might be
   identified by different session keys though.
1. BEEFY runs on **finalized** canonical chain, i.e. no forks (note [Misbehavior](#2-misbehavior)
   section though).
1. From a single validator perspective, BEEFY has at most one active voting round. Since GRANDPA
   validators are reaching finality, we assume they are on-line and well-connected and have
   similar view of the state of the blockchain.

## Ethereum

Initial version of BEEFY was made to enable efficient bridging with Ethereum, where the light client
is a Solidity Smart Contract compiled to EVM bytecode. Hence the choice of the initial cryptography
for BEEFY: `secp256k1` and usage of `keccak256` hashing function. See more in
[Data Formats](#2-data-formats) section.

<details>
<summary>Future: Supporting multiple crypto.</summary>

While BEEFY currently works with `secp256k1` signatures, we intend in the future to support multiple
signature schemes.
This means that multiple kinds of `SignedCommitment`s might exist and only together they form a full
`BEEFY Justification` (see more in [naming details](2-details)).

</details>

# The BEEFY Protocol

## Mental Model

BEEFY should be considered as an extra voting round done by GRANDPA validators for the current best
finalized block. Similarily to how GRANDPA is lagging behind best produced (non-finalized) block,
BEEFY is going to lag behind best GRANDPA (finalized) block.

```
                       ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
                       │      │ │      │ │      │ │      │ │      │
                       │  B1  │ │  B2  │ │  B3  │ │  B4  │ │  B5  │
                       │      │ │      │ │      │ │      │ │      │
                       └──────┘ └───▲──┘ └──────┘ └───▲──┘ └───▲──┘
                                    │                 │        │
     Best BEEFY block───────────────┘                 │        │
                                                      │        │
     Best GRANDPA block───────────────────────────────┘        │
                                                               │
     Best produced block───────────────────────────────────────┘

```

A pseudo-algorithm of behavior for a fully-synced BEEFY validator is:

```
loop {
  let (best_beefy, best_grandpa) = wait_for_best_blocks();

  let block_to_vote_on = choose_next_beefy_block(
    best_beefy,
    best_grandpa
  );

  let payload_to_vote_on = retrieve_payload(block_to_vote_on);

  let commitment = (block_to_vote_on, payload_to_vote_on);

  let signature = sign_with_current_session_key(commitment);

  broadcast_vote(commitment, signature);
}
```

Read more about the details in [Implementation](#1-implementation) section.

## Details

Before we jump into describing how BEEFY works in details, let's agree on the terms we are going to
use and actors in the system. All nodes in the network need to participate in the BEEFY networking
protocol, but we can identify two distinct actors though: **regular nodes** and **BEEFY validators**.
Validators are expected to actively participate in the protocol, by producing and broadcasting
**votes**. Votes are simply their signatures over a **Commitment**. A Commitment consists of a
**payload** (an opaque blob of bytes extracted from a block or state at that block, expected to be some
form of crypto accumulator (like Merkle Tree Hash or Merkle Mountain Range Root Hash)) and **block
number** from which this payload originates. Additionally Commitment contains BEEFY **validator
set id** at that particular block. Note the block is finalized, so there is no ambiguity despite
using block number instead of a hash. A collection of **votes**, or rather
a Commitment and a collection of signatures is going to be called **Signed Commitment**. A valid
(see later for the rules) Signed Commitment is also called a **BEEFY Justification** or
**BEEFY Finality Proof**. For more details on the actual data structures please see
[Data Formats](#2-data-formats).

A **round** is an attempt by BEEFY validators to produce BEEFY Justification. **Round number** is
simply defined as a block number the validators are voting for, or to be more precise, the
Commitment for that block number. Round ends when the next round is started, which may happen when
one of the events occur:
1. Either the node collects `2/3rd + 1` valid votes for that round.
2. Or the node receives a BEEFY Justification for a block greater than the current best BEEFY block.

In both cases the node proceeds to determining the new round number using
[Round Selection](#3-round-selection) procedure.

Regular nodes are expected to:
1. Receive & validate votes for the current round and broadcast them to their peers.
1. Receive & validate BEEFY Justifications and broadcast them to their peers.
1. Return BEEFY Justifications for [**Mandatory Blocks**](#3-round-selection) on demand.
1. Optionally return BEEFY Justifications for non-mandatory blocks on demand.

Validators are expected to additionally:
1. Produce & broadcast vote for the current round.

Both kinds of actors are expected to fully participate in the protocol ONLY IF they believe they
are up-to-date with the rest of the network, i.e. they are fully synced. Before this happens,
the node should stay completely passive with regards to the BEEFY protocol.

See [Initial Sync](#3-initial-sync) section for details on how to sync BEEFY.

### Round Selection

Every node (both regular nodes and validators) need to determine locally what they believe
current round number is. The choice is based on their knowledge of:

1. Best GRANDPA finalized block number (`best_grandpa`).
1. Best BEEFY finalized block number (`best_beefy`).
1. Starting block of current session (`session_start`).

**Session** means a period of time (or rather number of blocks) where validator set (keys) do not change.
See `pallet_session` for implementation details in `FRAME` context. Since we piggy-back on GRANDPA,
session boundaries for BEEFY are exactly the same as the ones for GRANDPA.

We define two kinds of blocks from the perspective of BEEFY protocol:
1. **Mandatory Blocks**
2. **Non-mandatory Blocks**

Mandatory blocks are the ones that MUST have BEEFY justification. That means that the validators
will always start and conclude a round at mandatory blocks. For non-mandatory blocks, there may
or may not be a justification and validators may never choose these blocks to start a round.

Every **first block in** each **session** is considered a **mandatory block**. All other blocks in
the session are non-mandatory, however validators are encouraged to finalize as many blocks as
possible to enable lower latency for light clients and hence end users. Since GRANDPA is considering
session boundary blocks as mandatory as well, `session_start` block will always have both GRANDPA
and BEEFY Justification.

<details>
<summary>Extra considerations regarding mandatory blocks choice.</summary>

(TODO [ToDr] Check with Al/Alfonso if there is any benefit of having
mandatory GRANDPA & BEEFY justification on the same block).

</details>

Therefore, to determine current round number nodes use a formula:

```
round_number =
      (1 - M) * session_start
   +        M * (best_beefy + NEXT_POWER_OF_TWO((best_grandpa - best_beefy + 1) / 2))
```

where:

- `M` is `1` if mandatory block in current session is already finalized and `0` otherwise.
- `NEXT_POWER_OF_TWO(x)` returns the smallest number greater or equal to `x` that is a power of two.

In other words, the round number should be the last mandatory block if it does not have
justification yet or the highest GRANDPA-finalized block, whose block number difference with
`best_beefy` block is a power of two. The mental model for round selection is to first finalize
the mandatory block and then to attempt to pick a block taking into account how fast BEEFY catches
up with GRANDPA. In case GRANDPA makes progress, but BEEFY seems to be lagging behind, validators
are changing rounds less often to increase the chance of concluding them.

As mentioned earlier, every time the node picks a new `round_number` (and validator casts a vote) it
ends the previous one, no matter if finality was reached (i.e. the round concluded) or not. Votes
for an inactive round should not be propagated.

Note that since BEEFY only votes for GRANDPA-finalized blocks, `session_start` here actually means:
"the latest session for which the start of is GRANDPA-finalized", i.e. block production might have
already progressed, but BEEFY needs to first finalize the mandatory block of the older session. See
[Catch up](#3-catch-up) for more details and also consult [Lean BEEFY](#2-lean-beefy).

In good networking conditions BEEFY may end up finalizing each and every block (if GRANDPA does
the same). Practically, with short block times, it's going to be rare and might be excessive, so
it's suggested for implementations to introduce a `min_delta` parameter which will limit the
frequency with which new rounds are started. The affected component of the formula would be:
`best_beefy + MAX(min_delta, NEXT_POWER_OF_TWO(...))`, so we start a new round only if the
power-of-two component is greater than the min delta. Note that if `round_number > best_grandpa`
the validators are not expected to start any round.

<details>
<summary>Future: New round selection.</summary>

Better heuristic from Al: in the next beefy round, consider the finalized blocks that
come after the last beefy-signed block, and select the one whose block number that can be divided
by 2 the most times.
(i.e it's forgiving if validators don't agree on neither best beefy block nor best grandpa block)

</details>

### Catch up

Every session is guaranteed to have at least one BEEFY-finalized block. However it also means
that the round at mandatory block must be concluded even though, a new session has already started
(i.e. the on-chain component has selected a new validator set and GRANDPA might have already
finalized the transition). In such case BEEFY must "catch up" the previous sessions and make sure to
conclude rounds for mandatory blocks. Note that older sessions must obviously be finalized by the
validator set at that point in time, not the latest/current one.

### Initial Sync

It's all rainbows and unicorns when the node is fully synced with the network. However during cold
startup it will have hard time determining the current round number. Because of that nodes that
are not fully synced should not participate in BEEFY protocol at all.

During the sync we should make sure to also fetch BEEFY justifications for all mandatory blocks.
This can happen asynchronously, but validators, before starting to vote, need to be certain
about the last session that contains a concluded round on mandatory block in order to initiate the
catch up procedure.

### Gossip

Nodes participating in BEEFY protocol are expected to gossip messages around. The protocol defines
following messages:

1. Votes for the current round,
2. BEEFY Justifications for recently concluded rounds,
3. BEEFY Justification for the latest mandatory block,

Each message is additionally associated with a **topic**, which can be either:
1. the round number (i.e. topic associated with a particular round),
2. or the global topic (independent from the rounds).

Round-specific topic should only be used to gossip the votes, other messages are gossiped
periodically on the global topic. Let's now dive into description of the messages.

- **Votes**
  - Votes are sent on the round-specific topic.
  - Vote is considered valid when:
    - The commitment matches local commitment.
    - The validator is part of the current validator set.
    - The signature is correct.

- **BEEFY Justification**
  - Justifications are sent on the global topic.
  - Justification is considered worthwhile to gossip when:
    - It is for a recent (implementation specific) round or the latest mandatory round.
    - All signatures are valid and there is at least `2/3rd + 1` of them.
    - Signatorees are part of the current validator set.
  - Mandatory justifications should be announced periodically (see also [Lean BEEFY](#2-lean-beefy)).

## Misbehavior

Similarily to other PoS protocols, BEEFY considers casting two different votes in the same round a
misbehavior. I.e. for a particular `round_number`, the validator produces signatures for 2 different
`Commitment`s and broadcasts them. This is called **equivocation**.

On top of this, voting on an incorrect **payload** is considered a misbehavior as well, and since
we piggy-back on GRANDPA there is no ambiguity in terms of the fork validators should be voting for.

Misbehavior should be penalized. If more validators misbehave in the exact same `round` the penalty
should be more severe, up to the entire bonded stake in case we reach `1/3rd + 1` validators
misbehaving.

<details>
<summary>Open questions</summary>

// TODO [ToDr] Penalization formula.
// TODO [ToDr] If there are misbehaving validators in GRANDPA should we penalize them twice?
IMHO yes, cause they should be relying on GRANDPA finality proof, not just the votes.

</details>

# Implementation

TODO

- What if grandpa < beefy?
- Missing mandatory blocks
- Extra assumptions and consequences (session start block selection) [Lean BEEFY]

///
When we start the worker, we should:
1. Ask the Client for the Best BEEFY block
   (TODO: what if we don't have that API?)
2. Traverse up to "Session Length" number of blocks forward to find a session transition.
3. Figure out if we need to vote on the mandatory block of the next session
   (i.e. we are doing BEEFY Catch-up)
4. or if we can follow the power-of-two rule.

With LEAN BEEFY:
1. Start with Best GRANDPA block
2. Traverse up to "Session Length" number of blocks backwards to find the session start block.
3. ASSUME this block is BEEFY-finalized, even though we haven't seen BEEFY Justification.


//TODO [ToDr] How to figure out the `session_start` block?
 - with additional 2/3rds assumption we just take the best session we see
 - in the future we need to take the first session that comes right after `best_beefy` block.

## On-Chain Pallet

TODO

## BEEFY Worker

TODO

## Lean BEEFY

To simplify the initial implementation of BEEFY, we are taking a bunch of shortcuts. These
shotcuts are described in this section and are collectively called "Lean BEEFY".

Lean BEEFY is based on additional assumption of validators always being able to complete the
mandatory round, i.e. there is at least `2/3rd + 1` validators on-line during the session long
enough to conclude at least one round. Based on this assumption, we further relax some requirements
to simplify the implementation.

### Justifications Sync

The nodes are not required to sync BEEFY justifications in order, nor they are expected to
have all justifications for mandatory blocks. Since the assumption is that these mandatory rounds
have been concluded the nodes assume that justifications are available somewhere, and since BEEFY
piggy-backs on GRANDPA, it's further assumed that since GRANDPA finalized blocks, BEEFY had to
correctly finalize the same blocks as well.

Implementation wise, this means that it's not required to plug into the block import pipeline or
alter any `sync` logic. The node can only import justifications that it sees being gossiped when it's
on-line. The complete sync of justifications for mandatory blocks can be implemented later.

### Round Selection

Since the node does not necessarily have justifications for mandatory blocks, when it sees GRANDPA
moving to a new session, it can assume BEEFY has moved to that new session as well. Hence the
`best_beefy` block number can immediatelly be set to the mandatory block number of the previous
session, despite the fact the node does not have the justification for that block yet.

### Mandatory Justification Gossip

Due to the lack of `sync` alterations, that would ensure all justifications for mandatory blocks
are fetched and imported to the local database, the node, after going on-line will miss recent
justifications. To help out with justification propagation mandatory justifications are gossiped
on the global topic as described in the [Gossip](#3-Gossip) section. While they are not strictly
necessary for neither BEEFY nor Lean BEEFY, practically this should make the consensus more robust.

# BEEFY & MMR (Polkadot implementation)

TODO

## Data Formats

TODO

# Light Client Design

TODO

## Substrate Runtime

TODO

## Solidity Smart Contract

TODO

