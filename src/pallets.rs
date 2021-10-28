use parity_scale_codec::Encode;

use crate::client::{Api, ClientError, Result, Signer};
use crate::rpc::RpcClient;
use crate::{public_into_account, Era, GenericExtra, SignedPayload, UncheckedExtrinsic};

pub mod balances;
pub mod staking;
pub mod storage;

pub(crate) type CallIndex = [u8; 2];

impl<S: Signer, Client: RpcClient> Api<'_, S, Client> {
    /// Creates and signs an extrinsic that can be submitted to a node
    pub fn create_xt<C: Encode + Clone>(&self, call: C) -> UncheckedExtrinsic<C> {
        let gen_hash = self.genesis_hash;
        let runtime_version = self.runtime_version;
        let extra = GenericExtra::new(Era::Immortal, self.nonce().expect("account nonce"));
        let s_extra = (
            runtime_version.spec_version,
            runtime_version.transaction_version,
            gen_hash,
            gen_hash,
            (),
            (),
            (),
        );
        let raw_payload = SignedPayload::new(call.clone(), extra, s_extra);

        let signature = if let Some(signer) = &self.signer {
            let from = signer.public().into();
            let sig = raw_payload.encoded(|payload| signer.sign(payload));
            Some((from, sig, extra))
        } else {
            None
        };

        UncheckedExtrinsic {
            signature,
            function: call,
        }
    }

    pub fn nonce(&self) -> Result<u32> {
        match &self.signer {
            Some(signer) => {
                let my_pub = public_into_account(signer.public());
                let info = self.account_info(my_pub)?;
                Ok(info.nonce)
            }
            None => Err(ClientError::NoSigner),
        }
    }
}
