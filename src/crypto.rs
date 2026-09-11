use aes::cipher::block_padding::NoPadding;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use aes::Aes256;

use crate::error::AppError;

type Aes256CbcDec = cbc::Decryptor<Aes256>;

#[derive(Clone)]
pub struct Cipher {
    key: [u8; 32],
    iv: [u8; 16],
}

impl Cipher {
    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self, AppError> {
        let key: [u8; 32] = key
            .try_into()
            .map_err(|_| AppError::Config("KEY must be 32 bytes".into()))?;
        let iv: [u8; 16] = iv
            .try_into()
            .map_err(|_| AppError::Config("IV must be 16 bytes".into()))?;
        Ok(Self { key, iv })
    }

    /// Decrypts a hex encoded AES-256-CBC payload and strips the trailing
    /// PKCS7-style padding byte, mirroring the original Go implementation.
    pub fn decrypt_hex(&self, payload: &str) -> Result<String, AppError> {
        let mut buffer = hex::decode(payload)
            .map_err(|_| AppError::Crypto("couldn't decode image hex".into()))?;

        if buffer.is_empty() || buffer.len() % 16 != 0 {
            return Err(AppError::Crypto("couldn't decode image hex".into()));
        }

        let key = GenericArray::from_slice(&self.key);
        let iv = GenericArray::from_slice(&self.iv);
        let decryptor = Aes256CbcDec::new(key, iv);

        let decrypted = decryptor
            .decrypt_padded_mut::<NoPadding>(&mut buffer)
            .map_err(|_| AppError::Crypto("couldn't decrypt image".into()))?;

        let pad = *decrypted
            .last()
            .ok_or_else(|| AppError::Crypto("couldn't decrypt image".into()))?
            as usize;

        if pad == 0 || pad > decrypted.len() {
            return Err(AppError::Crypto("couldn't decrypt image".into()));
        }

        let unpadded = &decrypted[..decrypted.len() - pad];
        String::from_utf8(unpadded.to_vec())
            .map_err(|_| AppError::Crypto("decrypted url is not valid utf-8".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes::cipher::{BlockEncryptMut, KeyIvInit};

    type Aes256CbcEnc = cbc::Encryptor<Aes256>;

    fn encrypt_hex(key: &[u8; 32], iv: &[u8; 16], plaintext: &str) -> String {
        let mut data = plaintext.as_bytes().to_vec();
        let pad = 16 - (data.len() % 16);
        data.extend(std::iter::repeat_n(pad as u8, pad));
        let msg_len = data.len();

        let key = GenericArray::from_slice(key);
        let iv = GenericArray::from_slice(iv);
        let encryptor = Aes256CbcEnc::new(key, iv);
        let encrypted = encryptor
            .encrypt_padded_mut::<NoPadding>(&mut data, msg_len)
            .unwrap();
        hex::encode(encrypted)
    }

    #[test]
    fn round_trip_decrypt() {
        let key = *b"07e983ed1676bbfa9d01f3ea7e23179c";
        let iv = *b"40f38a55cb4fec35";
        let cipher = Cipher::new(&key, &iv).unwrap();

        let url = "https://example.com/image.png";
        let payload = encrypt_hex(&key, &iv, url);
        assert_eq!(cipher.decrypt_hex(&payload).unwrap(), url);
    }

    #[test]
    fn round_trip_decrypt_exact_block() {
        let key = *b"07e983ed1676bbfa9d01f3ea7e23179c";
        let iv = *b"40f38a55cb4fec35";
        let cipher = Cipher::new(&key, &iv).unwrap();

        let url = "https://ex.com/ab";
        let payload = encrypt_hex(&key, &iv, url);
        assert_eq!(cipher.decrypt_hex(&payload).unwrap(), url);
    }

    #[test]
    fn rejects_invalid_hex() {
        let cipher = Cipher::new(b"07e983ed1676bbfa9d01f3ea7e23179c", b"40f38a55cb4fec35").unwrap();
        assert!(cipher.decrypt_hex("not-hex").is_err());
    }

    #[test]
    fn rejects_wrong_key_length() {
        assert!(Cipher::new(b"short", b"40f38a55cb4fec35").is_err());
    }
}
