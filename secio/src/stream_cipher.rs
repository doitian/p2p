/// Most of the code for this module comes from `rust-libp2p`
///
/// But upgrade library dependencies and follow the latest library requirements
use crate::codec::StreamCipher;
use aes_ctr::stream_cipher::generic_array::GenericArray;
use aes_ctr::stream_cipher::NewStreamCipher;
use aes_ctr::{Aes128Ctr, Aes256Ctr};
use ctr::Ctr128;
use twofish::Twofish;

/// Possible encryption ciphers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cipher {
    /// Aes128
    Aes128,
    /// Aes256
    Aes256,
    /// Two fish
    TwofishCtr,
}

impl Cipher {
    /// Returns the size of in bytes of the key expected by the cipher.
    pub fn key_size(self) -> usize {
        match self {
            Cipher::Aes128 => 16,
            Cipher::Aes256 => 32,
            Cipher::TwofishCtr => 32,
        }
    }

    /// Returns the size of in bytes of the IV expected by the cipher.
    #[inline]
    pub const fn iv_size(self) -> usize {
        16
    }
}

/// Returns your stream cipher depending on `Cipher`.
#[inline]
pub fn ctr_init(key_size: Cipher, key: &[u8], iv: &[u8]) -> StreamCipher {
    match key_size {
        Cipher::Aes128 => Box::new(Aes128Ctr::new(
            GenericArray::from_slice(key),
            GenericArray::from_slice(iv),
        )),
        Cipher::Aes256 => Box::new(Aes256Ctr::new(
            GenericArray::from_slice(key),
            GenericArray::from_slice(iv),
        )),
        Cipher::TwofishCtr => Box::new(Ctr128::<Twofish>::new(
            GenericArray::from_slice(key),
            GenericArray::from_slice(iv),
        )),
    }
}
