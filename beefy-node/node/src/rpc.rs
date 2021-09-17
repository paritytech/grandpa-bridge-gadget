//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use jsonrpsee::RpcModule;
use sc_rpc::SubscriptionTaskExecutor;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::traits::Block as BlockT;

pub use sc_rpc_api::DenyUnsafe;

use beefy_gadget::notification::BeefySignedCommitmentStream;
use beefy_node_runtime::{opaque::Block, AccountId, Balance, Index};

/// Extra dependencies for BEEFY
pub struct BeefyDeps<B: BlockT> {
	/// Receives notifications about signed commitments from BEEFY.
	pub signed_commitment_stream: BeefySignedCommitmentStream<B>,
	/// Executor to drive the subscription manager in the BEEFY RPC handler.
	pub subscription_executor: SubscriptionTaskExecutor,
}

/// Full client dependencies.
pub struct FullDeps<B: BlockT, C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// BEEFY specific dependencies.
	pub beefy: BeefyDeps<B>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<B, C, P>(deps: FullDeps<B, C, P>) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	B: BlockT,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
{
	use beefy_gadget_rpc::{BeefyApiServer, BeefyRpcHandler};
	use pallet_transaction_payment_rpc::{TransactionPaymentApiServer, TransactionPaymentRpc};
	use substrate_frame_rpc_system::{SystemApiServer, SystemRpc, SystemRpcBackendFull};

	let mut io = RpcModule::new(());
	let FullDeps {
		client,
		pool,
		deny_unsafe,
		beefy,
	} = deps;

	let BeefyDeps {
		signed_commitment_stream,
		subscription_executor,
	} = beefy;

	let system_backend = SystemRpcBackendFull::new(client.clone(), pool, deny_unsafe);
	io.merge(SystemRpc::new(Box::new(system_backend)).into_rpc())?;
	io.merge(TransactionPaymentRpc::new(client.clone()).into_rpc())?;

	// Extend this RPC with a custom API by using the following syntax.
	// `YourRpcStruct` should have a reference to a client, which is needed
	// to call into the runtime.
	// `io.extend_with(YourRpcTrait::to_delegate(YourRpcStruct::new(ReferenceToClient, ...)));`

	io.merge(BeefyRpcHandler::new(signed_commitment_stream, subscription_executor).into_rpc())?;

	Ok(io)
}
