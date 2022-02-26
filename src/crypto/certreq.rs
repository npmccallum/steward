use anyhow::{anyhow, Result};
use der::{asn1::BitString, Encodable};
use pkcs8::PrivateKeyInfo;
use x509::request::{CertReq, CertReqInfo};

use super::{PrivateKeyInfoExt, SubjectPublicKeyInfoExt};

pub trait CertReqExt<'a> {
    /// Verifies that a certification request is sane.
    ///
    /// This verifies that the certification request is properly self signed
    /// and well formatted. The rest is up to you.
    fn verify(self) -> Result<CertReqInfo<'a>>;
}

impl<'a> CertReqExt<'a> for CertReq<'a> {
    fn verify(self) -> Result<CertReqInfo<'a>> {
        if self.info.version != x509::request::Version::V1 {
            return Err(anyhow!("invalid version"));
        }

        let sign = self
            .signature
            .as_bytes()
            .ok_or(anyhow!("invalid signature"))?;
        let body = self.info.to_vec()?;
        self.info.public_key.verify(&body, self.algorithm, sign)?;

        Ok(self.info)
    }
}

pub trait CertReqInfoExt<'a> {
    /// Signs the `CertReqInfo` with the specified `PrivateKeyInfo`
    fn sign(self, pki: &PrivateKeyInfo) -> Result<Vec<u8>>;
}

impl<'a> CertReqInfoExt<'a> for CertReqInfo<'a> {
    fn sign(self, pki: &PrivateKeyInfo) -> Result<Vec<u8>> {
        let algo = pki.signs_with()?;
        let body = self.to_vec()?;
        let sign = pki.sign(&body, algo)?;

        let rval = CertReq {
            info: self,
            algorithm: algo,
            signature: BitString::from_bytes(&sign)?,
        };

        Ok(rval.to_vec()?)
    }
}
