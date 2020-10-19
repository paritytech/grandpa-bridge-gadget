struct MerkleMountainRangeLeaf(
    (
        /// This is populated only for epoch blocks and contains a merkle root of the NEXT
        /// validator set.
        Option<MerkleRootOfPublicKeys>,
        ///
        MerkleRootOfParaHeads,
        ///
        ParentBlockHash,
    ),
)

pub struct Commitment<H> {
    /// The payload being signed.
    ///
    /// This should be some form of cummulative representation of the chain (think MMR root hash).
    /// For transition blocks it also MUST contain details of the next validator set.
    pub payload: H,

    /// BEEFY round id this commitment is for.
    ///
    /// Round id starts from `0` and is reset for each `validator_set_id`.
    /// Commitment in the next round is guaranteed to be generated from
    /// a block that is more far in the future - i.e. the chain made some progress.
    pub round_id: u64,

    /// BEEFY valitor set supposed to sign this comittment.
    pub validator_set_id: u64,

    /// Indicator of the last block of the epoch.
    ///
    /// The payload will contain some form of the NEW validator set public keys
    /// information, yet the block is signed by the current validator set.
    /// When this committment is imported, the client MUST increment the `validator_set_id`.
    pub is_set_transition_block: bool,
}

pub struct SignedCommitment<H, Sig> {
    /// comittment
    pub comittment: Commitment<H>,
    /// signatures
    pub signatures: Vec<Sig>,
}

pub struct PartialSignedCommitment<H> {
    pub comittment: ...,

    pub bit_vec_of_signatures: Vec<bool>,

    pub merkle_root_of_signatures: ...,
}


#[cfg(test)]
mod tests {

    #[test]
    fn light_client_andres_case() {
        // we submit block 10 (1st phase)
        
        // an app submitting a mmr proof against this hash from block10


        // we are at block 10 doing second phase for it. (validatorset=1)
        ..

        // we got block 20 (validatorset=1)

        ..
    }

    #[test]
    fn solidity_light_client_makes_progress() {
        let lc = SolidityContractOnEthereum;
        //
        lc.submit(PartialSignedCommitment {});

        // For epoch blocks
        lc.submit(PartialSignedCommitment {}, MMMProofOfMerkleProofOfPublicKeys);

        //TODO: 2nd phase verification
        lc.submit_signatures(
            Vec<(idx, PublicKey)>,
            Vec<(idx, Signature)>,
            MerkleProofOfPublicKeys,
            MerkleProofOfSignatures,
        );
    }

    #[test]
    fn light_client_makes_progress() {
        let lc = ...;

        lc.submit(SignedCommitment, None);

        // if the validator_set_id_changes we require an extra proof.
        lc.submit(SignedCommitment, Some(Vec<PublicKey> + MMRProofForTheValidatorMerkleRoot));
    }

    #[test]
    fn can_process_bridge_messages() {
        ParachainY -HRMP->ParachainX
        ParachainX -> SmartContractX



        RelayChain -> BridgeSmartContract
        //
        //
        //     x       
        // ........................... Relay Chian Blocks
        // c c c c c c | c c c c c c c Generated comittment (by BEEFY)
        // ^   ^   ^   ^               Commitment seen by the light client
        //             *               2nd-phase proven commitments.
        //     l   l
        //
        //     x
        // ....a......................
        // c c c c c c | c c c c c c c
        // ^           ^
        //             *
                    


        let heavy_proof = (
            ParachainSpecificProof, // for instance storage proof on a parachain
            ParachainHead,
            MerkleProofOfParachainHeadAtTheRelayChainBlockXWhenParachainHeadGotIncluded,
            //MmrProofOfRelayChainBlock,
            Block10
        );

        let lighter_proof = (
            MmrProofOfRelayChainBlock,

        );

    }
}
