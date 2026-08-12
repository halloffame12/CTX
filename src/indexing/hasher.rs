//! Content hashing (SHA-256) used for incremental indexing.

use sha2::{Digest, Sha256};

use crate::errors::CtxResult;

pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in digest {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}

pub fn hash_file(path: &std::path::Path) -> CtxResult<String> {
    let data = std::fs::read(path)?;
    Ok(hash_bytes(&data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_hash() {
        assert_eq!(hash_bytes(b"ctx"), hash_bytes(b"ctx"));
        assert_ne!(hash_bytes(b"ctx"), hash_bytes(b"ctx!"));
        assert_eq!(hash_bytes(b"").len(), 64);
    }
}
