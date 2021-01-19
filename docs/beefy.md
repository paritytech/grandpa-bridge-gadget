# BEEFY

Related issues:
- [Merkle Mountain Range for Efficient Bridges](https://github.com/paritytech/parity-bridges-common/issues/263)
- [Grandpa Super Light Client](https://github.com/paritytech/parity-bridges-common/issues/323)

### TL;DR / Rationale

The idea is that we have an extra round of signatures after GRANDPA for which we'll use the Ethereum
standard secp256k1 ECDSA, which I understand that is already supported by Substrate.  The advantages
of doing the signatures outside GRANDPA are that aside from not touching existing code, we can have
all validators sign the same thing, rather than potentially different blocks and having to deal with
equivocations in GRANDPA.  Aside from that, it also gives the advantage that although all honest
validators sign it, so we get 2/3 of validators signatures, we only need to check that over 1/3 of
validators sign it, since honest validators would only sig if they see it is already final.

This means that we could verify finality interactively like this:
- We give the light client the thing that was signed, a bit field of validators who signed it, a
  Merkle root of a tree of signatures on it and a few arbitrary signatures the light client then asks
  for a few random signatures of those validators who signed it.
- We give these signatures and their Merkle proofs.
- The light client would previously have a Merkle root of all public keys and we'll need to give
Merkle proofs of the public keys along with the signatures.
- For the Ethereum on-chain light client, for 2, we would generate the random challenges using a
pseudo-random finstion seeded by the block hash of the block exactly 100 blocks later or the like.
  - I think we can query the last 255 block hashes using solidity so this should be fine.

### General Notes
Every time we say “Merkle root/merkelize” we mean `keccak256` ordered merkle trie
(`keccak_256_ordered_root` function in Substrate).

### BEEFY Substrate Pallet
- Tracks a list of secp256k1 public keys on-chain (BEEFY frame runtime).
- The pallet hooks up into the session lifecycle.
- Whenever the set is signalled we add a digest item to the header which contains a list of all
  Public Keys
- Stores the checkpoint (starting block), where we actually start this secondary protocol.
- Migration for session keys (session pallet) is required to support this extra key.

### BeefyToMmr Converter Pallet
- Take the list of validators from the (BEEFY Pallet) and merkelize them once per epoch (probably
  right after session change is triggered)
- Insert the merkle root hash of the secp256k1 public keys into the MMR

### BEEFY Service (Client)
- On the client side Grandpa produces a stream of finalized events and our new component (BEEFY)
  should:
  - Fetch the finalized block header
  - Retrieve the digest and update the Authority Set (Grandpa guarantees finalizing blocks that
    signal change).
  - Or assume there was no change and keep the previous one
- “The things we vote on” is pluggable:
  - We can either use the Digest which contains MMR root hash 
  - Or we can retrieve this from Offchain DB (MMR root hash pushed via Indexing API)
- Produce & gossip a vote on “the thing to be voted on” and start collecting signatures of others
  (we have to support multiple “things to be voted on” - the rounds happens concurrently)
  - We need to vote for every epoch change block (the ones that contain BEEFY Digest item)
  - If there is no such (pending) block, we vote roughly for
    ``` 
       // obviously block_to_sign_on has to be <= last_finalized_block.
       block_to_sign_on = last_block_with_signed_mmr_root
        + Max(
            2,
            NextPowerOfTwo((last_finalized_block - last_block_with_signed_mmr_root) / 2),
        )
    ```
      - Every time we are voting on a block, we call it a round
  - The round progresses concurrently - we might have multiple rounds happening at the same time.
  - The epoch change round HAS to be run to completion.
  - If we change the epoch we SHOULD cancel all previous rounds.
- Proofs for BEEFY should contain the signatures to make sure during sync we can easily verify that
  the transitions were done correctly.
  - The sync needs to be extended to fetch BEEFY justifications if we find them missing. (optional
    for MVP)
  - At the start we query the runtime to learn about the starting block and the initial set.
- Migration for existing blocks in the database to support multiple justifications: `Vec<(ConsensusEngineId, Blob)>`
- The BEEFY-justifications for epoch blocks are part of the blockchain database - there probably
  should be an RPC to retrieve them.
- Other non-epoch BEEFY-justifications can be stored only in memory (no need to persist them at all).
- RPC side:
  - Secondary-finalized-block-stream, something like grandpa_justifications RPC (Jon’s PR)
  - On-demand retrieval of BEEFY-justifications from epoch-blocks stored in the DB.
    (getBlock is enough - cause it returns all justifications)

#### Brain dump for the ETH contract:
1. Imports:
  - `MerkleMountRangeRootHash`
  - `CurrentBlockHash`
  - MerkleRoot of PublicKeys of the next Authority Set
  - Bit-vec of validators that signed (only accept if there is enough signatures)
  - Merkle root of all signatures
2. Starts interactive verification process
  - Pick ⅓ validators that signed at random
  - Request their signatures
3. Required signatures are then submitted:
  - We get ⅓ signatures + merkle proof that all of them were part of the initial set
  - We get ⅓ public keys (or their hashes) + merkle proof that they are part of the current
    Authority Set (stored in the contract)
  - We `ecrecover` the signatures and make sure the public keys match the ones we got and the merkle
    proof is valid.
