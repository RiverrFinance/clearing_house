use candid::Principal;
use ic_ledger_types::Subaccount as ICSubaccount;
use sha2::{Digest, Sha256};

/// Creates a subaccount from a Principal
///
/// # Arguments
/// * `principal` - The Principal to convert to a subaccount
///
/// # Returns
/// * `ICSubaccount` - A 32-byte subaccount derived from the principal
///
/// # Notes
/// - Uses SHA-256 hash of the principal bytes to generate the subaccount
/// - Ensures deterministic subaccount generation for the same principal
fn _to_subaccount(principal: &Principal) -> ICSubaccount {
    let mut hasher = Sha256::new();
    hasher.update(principal.as_slice());
    hasher.update(&0u64.to_be_bytes());
    let hash = hasher.finalize();
    let mut subaccount = [0u8; 32];
    subaccount.copy_from_slice(&hash[..32]);
    ICSubaccount(subaccount)
}
