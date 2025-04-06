use anchor_lang::prelude::*;

#[error_code]
pub enum StakeErrorCode {
    #[msg("the caller is not signer")]
    NotSigner,
    #[msg("Invalid round")]
    InvalidRound,
    #[msg("Already been initialized")]
    AlreadyInited,
    #[msg("Signature verification failed.")]
    SigVerificationFailed,
    #[msg("ZeroAddressError")]
    ZeroAddressError,

}
#[error_code]
pub enum ClaimErrorCode {
    #[msg("Invalid Amount")]
    InvalidAmount,
    #[msg("Not Approved")]
    NotApproved,
    #[msg("Signature verification failed")]
    SigVerificationFailed,
    #[msg("Insufficient Balance")]
    InsufficientBalance,
    #[msg("Invalid Timestamp")]
    InvalidTimestamp,
    #[msg("Invalid Nonce")]
    InvalidNonce,
}
