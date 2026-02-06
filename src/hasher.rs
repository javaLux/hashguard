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
    UnknownPrefix,
    EmptyHash,
    HexError(String),
}

impl std::fmt::Display for HashValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashValidationError::UnknownPrefix => write!(
                f,
                "Unknown prefix for the hash algorithm. For example, use 'sha256' to use the SHA2-256 algorithm."
            ),
            HashValidationError::EmptyHash => write!(f, "An empty hash is not allowed"),
            HashValidationError::HexError(err) => write!(f, "{err}"),
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

pub fn parse_hash(input: &str) -> Result<HashProperty, HashValidationError> {
    if input.trim().is_empty() {
        return Err(HashValidationError::EmptyHash);
    }

    let (hash, algorithm) = match input.split_once(':') {
        Some((prefix, hash)) => {
            // check prefix
            match <Algorithm as FromStr>::from_str(prefix) {
                Ok(alg) => (hash, Some(alg)),
                Err(_) => return Err(HashValidationError::UnknownPrefix),
            }
        }
        None => (input, None),
    };

    // try to decode the given hash, if it contains invalid hex digits or the length of the hex string
    // is odd -> an error is returned
    if let Err(err) = hex::decode(hash) {
        return Err(HashValidationError::HexError(err.to_string()));
    }

    Ok(HashProperty {
        hash: hash.to_string(),
        algorithm,
    })
}

// Verifies that every character in the string is a valid hexadecimal digit.
// Valid hexadecimal (hex) digits are characters that represent numbers in base-16 (hexadecimal) notation.
// In base-16, digits range from 0 to 15, and these are represented as follows:<br>
// Decimal 0-9: Represented directly as 0, 1, 2, 3, 4, 5, 6, 7, 8, 9.<br>
// Decimal 10-15: Represented as letters A, B, C, D, E, F (uppercase) or a, b, c, d, e, f (lowercase).
fn _is_valid_hex_digit(hash: &str) -> bool {
    !hash.trim().is_empty() && hash.chars().all(|c| c.is_ascii_hexdigit())
}

// Checks if the given hash is a valid Lower-Hex digit
fn _is_lower_hex(hash: &str) -> bool {
    !hash.trim().is_empty() && hash.chars().all(|c| matches!(c, 'a'..='f' | '0'..='9'))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test hashes for the string 'Hello World'
    // SHA-2
    const DATA_SHA2_224: &str = "c4890faffdb0105d991a461e668e276685401b02eab1ef4372795047";
    const DATA_SHA2_256: &str = "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e";
    const DATA_SHA2_384: &str = "99514329186b2f6ae4a1329e7ee6c610a729636335174ac6b740f9028396fcc803d0e93863a7c3d90f86beee782f4f3f";
    const DATA_SHA2_512: &str = "2c74fd17edafd80e8447b0d46741ee243b7eb74dd2149a0ab1b9246fb30382f27e853d8585719e0e67cbda0daa8f51671064615d645ae27acb15bfb1447f459b";

    // SHA-3
    const DATA_SHA3_224: &str = "8e800079a0b311788bf29353f400eff969b650a3597c91efd9aa5b38";
    const DATA_SHA3_256: &str = "e167f68d6563d75bb25f3aa49c29ef612d41352dc00606de7cbd630bb2665f51";
    const DATA_SHA3_384: &str = "a78ec2851e991638ce505d4a44efa606dd4056d3ab274ec6fdbac00cde16478263ef7213bad5a7db7044f58d637afdeb";
    const DATA_SHA3_512: &str = "3d58a719c6866b0214f96b0a67b37e51a91e233ce0be126a08f35fdf4c043c6126f40139bfbc338d44eb2a03de9f7bb8eff0ac260b3629811e389a5fbee8a894";

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
        assert!(_is_lower_hex(DATA_SHA2_512))
    }

    // -------------------------
    // ❌ Negative Tests
    // -------------------------

    #[test]
    fn invalid_hash_1() {
        assert!(parse_hash("xyz123").is_err());
    }

    #[test]
    fn invalid_hash_2() {
        assert!(parse_hash("abcdeF12345U").is_err());
    }

    #[test]
    fn odd_digit() {
        assert_eq!(
            parse_hash("ea10400a?4713d8464e24f40fe20d76fc0d755914ce8e76b1979f27f5"),
            Err(HashValidationError::HexError(
                "Odd number of digits".to_string()
            ))
        )
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
        assert!(!_is_lower_hex(&DATA_SHA3_512.to_ascii_uppercase()))
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
