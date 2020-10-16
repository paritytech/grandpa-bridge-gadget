pub struct Comittment<H> {
    /// most likely MMR ROOT
    pub payload: H,
    /// .....
    pub validator_handoff: Option<???>,
}

pub struct SignedComittment<H, Sig> {
    /// comittment
    pub comittment: Comittment<H>,
    /// signatures
    pub signatures: Vec<Sig>,
}


#[cfg(test)]
mod tests {
    #[test]
    fn can_process_bridge_messages() {
        let heavy_proof = (
            ParachainSpecificProof, // for instance storage proof on a parachain
            ParachainHead,
            MerkleProofOfParachainHeadAtRelayChain,
            MmrProofOfRelayChainBlock,
        );
    

        let lighter_proof = (
            MmrProofOfRelayChainBlock,

        );

    }
}
