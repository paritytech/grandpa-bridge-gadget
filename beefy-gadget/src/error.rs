#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// Check signature error
	#[error("Message signature {0:?} by {1:?} is invalid.")]
	InvalidSignature(Vec<u8>, Vec<u8>),
	/// Sign commitment error
	#[error("Failed to sign comitment using key: {0:?}. Reason: {1}")]
	CannotSign(Vec<u8>, String),
}
