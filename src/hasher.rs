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

impl FromStr for Algorithm {
    type Err = String;

    /// Validates whether the provided string slice corresponds to a supported hash algorithm.
    ///
    /// This function checks if the given input matches one of the variants of the [`Algorithm`] enum.
    /// The validation is case-insensitive, allowing for flexible input formats. If the input is a valid
    /// algorithm name, the function returns the corresponding `Algorithm` variant; otherwise, it returns
    /// an error with a descriptive message.
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice containing the name of the algorithm to validate.
    ///
    /// # Returns
    ///
    /// * `Ok(Algorithm)` - If the input matches a valid algorithm.
    /// * `Err(String)` - If the input is invalid, with an error message describing the issue.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(validate_algorithm("sha2-256"), Ok(Algorithm::SHA2_256));
    /// assert_eq!(validate_algorithm("invalid"), Err(String::from("Invalid algorithm: invalid - Possible values: [SHA2-224, SHA2-256, SHA2-384, SHA2-512, SHA3-224, SHA3-256, SHA3-384, SHA3-512]")));
    /// ```
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_uppercase().as_str() {
            "SHA2-224" => Ok(Algorithm::SHA2_224),
            "SHA2-256" => Ok(Algorithm::SHA2_256),
            "SHA2-384" => Ok(Algorithm::SHA2_384),
            "SHA2-512" => Ok(Algorithm::SHA2_512),
            "SHA3-224" => Ok(Algorithm::SHA3_224),
            "SHA3-256" => Ok(Algorithm::SHA3_256),
            "SHA3-384" => Ok(Algorithm::SHA3_384),
            "SHA3-512" => Ok(Algorithm::SHA3_512),
            _ => Err(format!("Invalid algorithm: {} - Possible values: [SHA2-224, SHA2-256, SHA2-384, SHA2-512, SHA3-224, SHA3-256, SHA3-384, SHA3-512]", input)),
        }
    }
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
    origin_hash_sum.to_lowercase() == calculated_hash_sum.to_lowercase()
}

#[allow(dead_code)]
/// Checks if the given hash is a valid Lower-Hex digit
pub fn is_lower_hex(hash: &str) -> bool {
    !hash.trim().is_empty() && hash.chars().all(|c| matches!(c, 'a'..='f' | '0'..='9'))
}

/// Verifies that every character in the string is a valid hexadecimal digit.
/// Valid hexadecimal (hex) digits are characters that represent numbers in base-16 (hexadecimal) notation.
/// In base-16, digits range from 0 to 15, and these are represented as follows:<br>
/// Decimal 0-9: Represented directly as 0, 1, 2, 3, 4, 5, 6, 7, 8, 9.<br>
/// Decimal 10-15: Represented as letters A, B, C, D, E, F (uppercase) or a, b, c, d, e, f (lowercase).
pub fn is_valid_hex_digit(hash: &str) -> bool {
    !hash.trim().is_empty() && hash.chars().all(|c| c.is_ascii_hexdigit())
}
