/*
 * Copyright (C) 2020 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::mem::MaybeUninit;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Unexpected error returned from {0}")]
    Unexpected(&'static str),
}

use authfs_crypto_bindgen::{SHA256_Final, SHA256_Init, SHA256_Update, SHA256_CTX};

pub struct Sha256Hasher {
    ctx: SHA256_CTX,
}

impl Sha256Hasher {
    pub const HASH_SIZE: usize = 32;

    pub fn new() -> Result<Sha256Hasher, CryptoError> {
        // Safe assuming the crypto FFI should initialize the uninitialized `ctx`, which is
        // currently a pure data struct.
        unsafe {
            let mut ctx = MaybeUninit::uninit();
            if SHA256_Init(ctx.as_mut_ptr()) == 0 {
                Err(CryptoError::Unexpected("SHA256_Init"))
            } else {
                Ok(Sha256Hasher { ctx: ctx.assume_init() })
            }
        }
    }

    pub fn update(&mut self, data: &[u8]) -> Result<&mut Self, CryptoError> {
        // Safe assuming the crypto FFI will not touch beyond `ctx` as pure data.
        let retval = unsafe {
            SHA256_Update(&mut self.ctx, data.as_ptr() as *mut std::ffi::c_void, data.len())
        };
        if retval == 0 {
            Err(CryptoError::Unexpected("SHA256_Update"))
        } else {
            Ok(self)
        }
    }

    pub fn finalize(&mut self) -> Result<[u8; Self::HASH_SIZE], CryptoError> {
        let mut md = [0u8; Self::HASH_SIZE];
        // Safe assuming the crypto FFI will not touch beyond `ctx` as pure data.
        let retval = unsafe { SHA256_Final(md.as_mut_ptr(), &mut self.ctx) };
        if retval == 0 {
            Err(CryptoError::Unexpected("SHA256_Final"))
        } else {
            Ok(md)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_hex_string(data: &[u8]) -> String {
        data.iter().map(|&b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn verify_hash_values() -> Result<(), CryptoError> {
        let hash = Sha256Hasher::new()?.update(&[0; 0])?.finalize()?;
        let s: String = to_hex_string(&hash);
        assert_eq!(s, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");

        let hash = Sha256Hasher::new()?
            .update(&[1u8; 1])?
            .update(&[2u8; 1])?
            .update(&[3u8; 1])?
            .finalize()?;
        let s: String = to_hex_string(&hash);
        assert_eq!(s, "039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81");
        Ok(())
    }
}
