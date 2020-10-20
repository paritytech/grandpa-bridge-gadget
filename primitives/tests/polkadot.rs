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
	// ParachainY -HRMP->ParachainX
	// ParachainX -> SmartContractX
	//
	//
	//
	// RelayChain -> BridgeSmartContract
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
