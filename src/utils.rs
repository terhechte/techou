pub fn hash_string(input: &str, length: usize) -> String {
    use sha2::Sha256;
    use sha2::Digest;
    let hash = format!("{:x}", Sha256::digest(input.as_bytes()));
    // 10 seems to be a good prefix for distinctness
    let (short_hash, _) = hash.split_at(length);
    short_hash.to_string()
}