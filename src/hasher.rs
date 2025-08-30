use clap::ValueEnum;
use std::str::FromStr;

use sha2::Digest;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
/// Supported hash algorithm for calculating the hash sum
pub enum Algorithm {
    SHA2_224,
    #[default]
    SHA2_256,
    SHA2_384,
    SHA2_512,
    SHA3_224,
    SHA3_256,
    SHA3_384,
    SHA3_512,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Algorithm::SHA2_224 => write!(f, "SHA2-224"),
            Algorithm::SHA2_256 => write!(f, "SHA2-256"),
            Algorithm::SHA2_384 => write!(f, "SHA2-384"),
            Algorithm::SHA2_512 => write!(f, "SHA2-512"),
            Algorithm::SHA3_224 => write!(f, "SHA3-224"),
            Algorithm::SHA3_256 => write!(f, "SHA3-256"),
            Algorithm::SHA3_384 => write!(f, "SHA3-384"),
            Algorithm::SHA3_512 => write!(f, "SHA3-512"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseAlgorithmError;

impl FromStr for Algorithm {
    type Err = ParseAlgorithmError;

    /// Validates whether the provided string slice corresponds to a supported hash algorithm.
    ///
    /// This function checks if the given input matches one of the variants of the [`Algorithm`] enum.
    /// The validation is case-insensitive, allowing for flexible input formats. If the input is a valid
    /// algorithm name, the function returns the corresponding `Algorithm` variant; otherwise, it returns an error.
    ///
    /// # Arguments
    ///
    /// * `v` - A string slice containing the name of the algorithm to validate.
    ///
    /// # Returns
    ///
    /// * Ok([`Algorithm`]) - If the input matches a valid algorithm.
    /// * Err([`ParseAlgorithmError`]) - If the input is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(validate_algorithm("sha2-256"), Ok(Algorithm::SHA2_256));
    /// assert_eq!(validate_algorithm("sha2_256"), Ok(Algorithm::SHA2_256));
    /// assert_eq!(validate_algorithm("sha256"), Ok(Algorithm::SHA2_256));
    /// assert_eq!(validate_algorithm("invalid"), Err(ParseAlgorithmError);
    /// ```
    fn from_str(v: &str) -> Result<Self, Self::Err> {
        match v.trim().to_ascii_lowercase().as_str() {
            "sha224" | "sha2-224" | "sha2_224" => Ok(Algorithm::SHA2_224),
            "sha256" | "sha2-256" | "sha2_256" => Ok(Algorithm::SHA2_256),
            "sha384" | "sha2-384" | "sha2_384" => Ok(Algorithm::SHA2_384),
            "sha512" | "sha2-512" | "sha2_512" => Ok(Algorithm::SHA2_512),
            "sha3-224" | "sha3_224" => Ok(Algorithm::SHA3_224),
            "sha3-256" | "sha3_256" => Ok(Algorithm::SHA3_256),
            "sha3-384" | "sha3_384" => Ok(Algorithm::SHA3_384),
            "sha3-512" | "sha3_512" => Ok(Algorithm::SHA3_512),
            _ => Err(ParseAlgorithmError),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum HashValidationError {
    InvalidFormat,
    UnknownPrefix,
    EmptyHash,
}

impl std::fmt::Display for HashValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashValidationError::InvalidFormat => write!(
                f,
                "The specified hash sum contains at least one invalid hexadecimal digit."
            ),
            HashValidationError::UnknownPrefix => write!(
                f,
                "Unknown prefix for the hash algorithm. For example, use 'sha256' to use the SHA2-256 algorithm."
            ),
            HashValidationError::EmptyHash => write!(f, "An empty hash is not allowed"),
        }
    }
}

impl std::error::Error for HashValidationError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HashProperty {
    pub hash: String,
    pub algorithm: Option<Algorithm>,
}

#[derive(Debug, Clone)]
pub enum Hasher {
    // --- SHA‑2 -------------------------------------------------------------
    SHA2_224(sha2::Sha224),
    SHA2_256(sha2::Sha256),
    SHA2_384(sha2::Sha384),
    SHA2_512(sha2::Sha512),
    // --- SHA‑3 -------------------------------------------------------------
    SHA3_224(sha3::Sha3_224),
    SHA3_256(sha3::Sha3_256),
    SHA3_384(sha3::Sha3_384),
    SHA3_512(sha3::Sha3_512),
}

impl Hasher {
    /// Creates a new hasher instance based on the specified algorithm.
    ///
    /// # Arguments
    ///
    /// * `algorithm` - The hash algorithm to use for creating the hasher.
    ///
    /// # Returns
    ///
    /// A new `Hasher` instance configured with the specified algorithm.
    pub fn new(algorithm: Algorithm) -> Self {
        match algorithm {
            Algorithm::SHA2_224 => Hasher::SHA2_224(sha2::Sha224::new()),
            Algorithm::SHA2_256 => Hasher::SHA2_256(sha2::Sha256::new()),
            Algorithm::SHA2_384 => Hasher::SHA2_384(sha2::Sha384::new()),
            Algorithm::SHA2_512 => Hasher::SHA2_512(sha2::Sha512::new()),
            Algorithm::SHA3_224 => Hasher::SHA3_224(sha3::Sha3_224::new()),
            Algorithm::SHA3_256 => Hasher::SHA3_256(sha3::Sha3_256::new()),
            Algorithm::SHA3_384 => Hasher::SHA3_384(sha3::Sha3_384::new()),
            Algorithm::SHA3_512 => Hasher::SHA3_512(sha3::Sha3_512::new()),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        match self {
            Hasher::SHA2_224(hasher) => hasher.update(data),
            Hasher::SHA2_256(hasher) => hasher.update(data),
            Hasher::SHA2_384(hasher) => hasher.update(data),
            Hasher::SHA2_512(hasher) => hasher.update(data),
            Hasher::SHA3_224(hasher) => hasher.update(data),
            Hasher::SHA3_256(hasher) => hasher.update(data),
            Hasher::SHA3_384(hasher) => hasher.update(data),
            Hasher::SHA3_512(hasher) => hasher.update(data),
        }
    }

    pub fn finalize(self) -> Vec<u8> {
        match self {
            Hasher::SHA2_224(hasher) => hasher.finalize().to_vec(),
            Hasher::SHA2_256(hasher) => hasher.finalize().to_vec(),
            Hasher::SHA2_384(hasher) => hasher.finalize().to_vec(),
            Hasher::SHA2_512(hasher) => hasher.finalize().to_vec(),
            Hasher::SHA3_224(hasher) => hasher.finalize().to_vec(),
            Hasher::SHA3_256(hasher) => hasher.finalize().to_vec(),
            Hasher::SHA3_384(hasher) => hasher.finalize().to_vec(),
            Hasher::SHA3_512(hasher) => hasher.finalize().to_vec(),
        }
    }

    /// Calculates the hash sum of the provided data and returns it as a hexadecimal string.
    pub fn digest_hex_lower(&self, data: &[u8]) -> String {
        match self {
            Hasher::SHA2_224(_) => format!("{:x}", sha2::Sha224::digest(data)),
            Hasher::SHA2_256(_) => format!("{:x}", sha2::Sha256::digest(data)),
            Hasher::SHA2_384(_) => format!("{:x}", sha2::Sha384::digest(data)),
            Hasher::SHA2_512(_) => format!("{:x}", sha2::Sha512::digest(data)),
            Hasher::SHA3_224(_) => format!("{:x}", sha3::Sha3_224::digest(data)),
            Hasher::SHA3_256(_) => format!("{:x}", sha3::Sha3_256::digest(data)),
            Hasher::SHA3_384(_) => format!("{:x}", sha3::Sha3_384::digest(data)),
            Hasher::SHA3_512(_) => format!("{:x}", sha3::Sha3_512::digest(data)),
        }
    }
}

/// Compares two hash sums for equality, accounting for potential case differences
/// in the original hash sum.
///
/// # Returns
/// - `true` if the two hash sums are equal (ignoring case).
/// - `false` if the hash sums differ
pub fn is_hash_equal(origin_hash_sum: &str, calculated_hash_sum: &str) -> bool {
    origin_hash_sum.eq_ignore_ascii_case(calculated_hash_sum)
}

#[allow(dead_code)]
/// Checks if the given hash is a valid Lower-Hex digit
pub fn is_lower_hex(hash: &str) -> bool {
    !hash.trim().is_empty() && hash.chars().all(|c| matches!(c, 'a'..='f' | '0'..='9'))
}

pub fn parse_hash(input: &str) -> Result<HashProperty, HashValidationError> {
    let (prefix, hash) = match input.split_once(':') {
        Some((p, h)) => (Some(p), h),
        None => (None, input),
    };

    if input.trim().is_empty() {
        return Err(HashValidationError::EmptyHash);
    }

    // check if the specified hash sum contains valid hex digits
    if !is_valid_hex_digit(hash) {
        return Err(HashValidationError::InvalidFormat);
    }

    if let Some(prefix) = prefix {
        // try to convert prefix into hash algorithm
        match <Algorithm as FromStr>::from_str(prefix) {
            Ok(algorithm) => Ok(HashProperty {
                hash: hash.to_string(),
                algorithm: Some(algorithm),
            }),
            Err(_) => Err(HashValidationError::UnknownPrefix),
        }
    } else {
        Ok(HashProperty {
            hash: hash.to_string(),
            algorithm: None,
        })
    }
}

/// Verifies that every character in the string is a valid hexadecimal digit.
/// Valid hexadecimal (hex) digits are characters that represent numbers in base-16 (hexadecimal) notation.
/// In base-16, digits range from 0 to 15, and these are represented as follows:<br>
/// Decimal 0-9: Represented directly as 0, 1, 2, 3, 4, 5, 6, 7, 8, 9.<br>
/// Decimal 10-15: Represented as letters A, B, C, D, E, F (uppercase) or a, b, c, d, e, f (lowercase).
pub fn is_valid_hex_digit(hash: &str) -> bool {
    !hash.trim().is_empty() && hash.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA_SHA2_224: &str = "ea10400a4713d8464e24f40fe20d76fc0d755914ce8e76b1979f27f5";
    const DATA_SHA2_256: &str = "9e2a73027d72a28e5cb05cf9e87e71d5f5850d047a8b163f92f2189e5e8f42ac";
    const DATA_SHA2_384: &str = "0c3a0b5e4476796cfb51f878d5b3c947a30258d3903480b3c0789d2f4d3c3cf308be7a38dbd28c166a4630d52e81173e";
    const DATA_SHA2_512: &str = "f95a2a01c2c67c67ac8c9c2459cbe7c5b046ec2a5cb1d9c13e9495ee46b5a40f3fc7c6b4c9b67f5cb7390c5aebfe08ec9fba97fc56f6e9246a70ffb62a27d3a2";
    const DATA_SHA3_224: &str = "bbdf787ab64cbaefcdbdfb64e32b62c70a63b59a73c74845c8a07c53";
    const DATA_SHA3_256: &str = "1f7b5dd56752d1f5a800c5c84ac2e8c7c3f637e6a9e3f7a532d51fb6593a2a62";
    const DATA_SHA3_384: &str = "3f5dd4d9024d4f87f9c123e635aee30f60a92c956632f3a14dbecbd68073e53230a51d6906b59c275a8e9e15e3edc700";
    const DATA_SHA3_512: &str = "3dbd5d9a2f0b768d3df6e86a0f38f2e27834c0fdaa9c2c49d7f4c8d9a730f4fbd812f91b534b9a20e064dc6c5c238ffb5c618f6916a8e65d909b5f34f6b1aef0";

    // -------------------------
    // ✅ Positive Tests
    // -------------------------

    #[test]
    fn parse_sha2_224() {
        let prefixes = ["sha224", "sha2-224", "sha2_224"];
        for prefix in prefixes {
            let input = format!("{prefix}:{DATA_SHA2_224}");
            assert_eq!(
                parse_hash(&input),
                Ok(HashProperty {
                    hash: DATA_SHA2_224.to_string(),
                    algorithm: Some(Algorithm::SHA2_224)
                })
            );
        }
    }

    #[test]
    fn parse_sha2_256() {
        let prefixes = ["sha256", "sha2-256", "sha2_256"];
        for prefix in prefixes {
            let input = format!("{prefix}:{DATA_SHA2_256}");
            assert_eq!(
                parse_hash(&input),
                Ok(HashProperty {
                    hash: DATA_SHA2_256.to_string(),
                    algorithm: Some(Algorithm::SHA2_256)
                })
            );
        }
    }

    #[test]
    fn parse_sha2_384() {
        let prefixes = ["sha384", "sha2-384", "sha2_384"];
        for prefix in prefixes {
            let input = format!("{prefix}:{DATA_SHA2_384}");
            assert_eq!(
                parse_hash(&input),
                Ok(HashProperty {
                    hash: DATA_SHA2_384.to_string(),
                    algorithm: Some(Algorithm::SHA2_384)
                })
            );
        }
    }

    #[test]
    fn parse_sha2_512() {
        let prefixes = ["sha512", "sha2-512", "sha2_512"];
        for prefix in prefixes {
            let input = format!("{prefix}:{DATA_SHA2_512}");
            assert_eq!(
                parse_hash(&input),
                Ok(HashProperty {
                    hash: DATA_SHA2_512.to_string(),
                    algorithm: Some(Algorithm::SHA2_512)
                })
            );
        }
    }

    #[test]
    fn parse_sha3_224() {
        let prefixes = ["sha3_224", "sha3-224"];
        for prefix in prefixes {
            let prefix = prefix.to_ascii_uppercase();
            let input = format!("{prefix}:{DATA_SHA3_224}");
            assert_eq!(
                parse_hash(&input),
                Ok(HashProperty {
                    hash: DATA_SHA3_224.to_string(),
                    algorithm: Some(Algorithm::SHA3_224)
                })
            );
        }
    }

    #[test]
    fn parse_sha3_256() {
        let prefixes = ["sha3_256", "sha3-256"];
        for prefix in prefixes {
            let input = format!("{prefix}:{DATA_SHA3_256}");
            assert_eq!(
                parse_hash(&input),
                Ok(HashProperty {
                    hash: DATA_SHA3_256.to_string(),
                    algorithm: Some(Algorithm::SHA3_256)
                })
            );
        }
    }

    #[test]
    fn parse_sha3_384() {
        let prefixes = ["sha3_384", "sha3-384"];
        for prefix in prefixes {
            let input = format!("{prefix}:{DATA_SHA3_384}");
            assert_eq!(
                parse_hash(&input),
                Ok(HashProperty {
                    hash: DATA_SHA3_384.to_string(),
                    algorithm: Some(Algorithm::SHA3_384)
                })
            );
        }
    }

    #[test]
    fn parse_sha3_512() {
        let prefixes = ["sha3_512", "sha3-512", "SHA3-512"];
        for prefix in prefixes {
            let input = format!("{prefix}:{DATA_SHA3_512}");
            assert_eq!(
                parse_hash(&input),
                Ok(HashProperty {
                    hash: DATA_SHA3_512.to_string(),
                    algorithm: Some(Algorithm::SHA3_512)
                })
            );
        }
    }

    #[test]
    fn parse_without_prefix() {
        let input = DATA_SHA2_256;
        assert_eq!(
            parse_hash(input),
            Ok(HashProperty {
                hash: input.to_string(),
                algorithm: None
            })
        );
    }

    #[test]
    fn hash_equal() {
        assert!(is_hash_equal(DATA_SHA2_224, DATA_SHA2_224))
    }

    #[test]
    fn lower_hex() {
        assert!(is_lower_hex(DATA_SHA2_512))
    }

    // -------------------------
    // ❌ Negative Tests
    // -------------------------

    #[test]
    fn invalid_hash() {
        let invalid_hashes = [
            "xyz",
            "ea10400a?4713d8464e24f40fe20d76fc0d755914ce8e76b1979f27f5",
        ];
        for hash in invalid_hashes {
            assert_eq!(parse_hash(hash), Err(HashValidationError::InvalidFormat))
        }
    }

    #[test]
    fn empty_hash() {
        let empty_hashes = ["", "\n    \t"];
        for hash in empty_hashes {
            assert_eq!(parse_hash(hash), Err(HashValidationError::EmptyHash))
        }
    }

    #[test]
    fn hash_not_equal() {
        assert!(!is_hash_equal(DATA_SHA2_224, DATA_SHA2_256))
    }

    #[test]
    fn hash_is_not_lower_hex() {
        assert!(!is_lower_hex(&DATA_SHA3_512.to_ascii_uppercase()))
    }

    #[test]
    fn unknown_prefixes() {
        let unknown_prefixes = ["md5", "sha1", "test", "\n    \t", ""];

        for prefix in unknown_prefixes {
            let input = format!("{prefix}:{DATA_SHA2_256}");
            assert_eq!(parse_hash(&input), Err(HashValidationError::UnknownPrefix))
        }
    }
}
