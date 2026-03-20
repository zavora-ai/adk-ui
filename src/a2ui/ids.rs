/// Stable, deterministic IDs for A2UI components.
///
/// Uses a simple FNV-1a 64-bit hash to avoid introducing extra dependencies.
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn fnv1a_64(input: &str) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Create a stable ID from a seed string.
pub fn stable_id(seed: &str) -> String {
    format!("id_{:016x}", fnv1a_64(seed))
}

/// Create a stable ID from a parent + child key.
pub fn stable_child_id(parent_id: &str, child_key: &str) -> String {
    stable_id(&format!("{}/{}", parent_id, child_key))
}

/// Create a stable ID for indexed children (arrays).
pub fn stable_indexed_id(parent_id: &str, kind: &str, index: usize) -> String {
    stable_id(&format!("{}/{}[{}]", parent_id, kind, index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_id_is_deterministic() {
        let a = stable_id("seed");
        let b = stable_id("seed");
        let c = stable_id("other");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn stable_child_id_matches_format() {
        let parent = "parent";
        let child = "child";
        assert_eq!(stable_child_id(parent, child), stable_id("parent/child"));
    }

    #[test]
    fn stable_indexed_id_matches_format() {
        let parent = "parent";
        assert_eq!(
            stable_indexed_id(parent, "item", 3),
            stable_id("parent/item[3]")
        );
    }
}
