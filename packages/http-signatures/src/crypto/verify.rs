use base64::{Engine, prelude::BASE64_STANDARD};
use miette::Diagnostic;
use quick_error::quick_error;
use ring::signature::UnparsedPublicKey;

quick_error! {
    /// Verification error
    #[derive(Debug, Diagnostic)]
    pub enum VerifyError {
        /// Failed to decode the Base64 payload
        Base64(err: base64::DecodeError) {
            from()
        }

        /// Verification failed
        Verification {}
    }
}

/// Verify that the message corresponds with the signature using the provided verifying key
#[inline]
pub fn verify<B>(
    msg: &[u8],
    encoded_signature: &str,
    key: &UnparsedPublicKey<B>,
) -> Result<(), VerifyError>
where
    B: AsRef<[u8]>,
{
    let signature = BASE64_STANDARD.decode(encoded_signature)?;
    key.verify(msg, &signature)
        .map_err(|_| VerifyError::Verification)
}
