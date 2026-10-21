//! Nonnegative integer marshalling/expansion, separate from gameplay randomness.
use std::sync::Arc;
use sha2::{Digest, Sha256};
use crate::{EnvError, InputReason};
use crate::error::{input, reserved, setup};

pub const SEED_ADAPTER_ID: &str = "spire-lab-env-seed-sha256-v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegerSeed(Arc<[u8]>);

impl IntegerSeed {
    pub fn from_be_bytes(bytes: &[u8]) -> Result<Self, EnvError> {
        if bytes.is_empty() { return Err(input(InputReason::Seed, "integer seed bytes must not be empty")); }
        let first = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len() - 1);
        let bytes = &bytes[first..];
        u64::try_from(bytes.len()).map_err(|_| setup("seed byte length exceeds u64 framing"))?;
        let mut owned = reserved(bytes.len())?;
        owned.extend_from_slice(bytes);
        Ok(Self(owned.into()))
    }

    pub fn as_be_bytes(&self) -> &[u8] { &self.0 }

    pub fn root(&self) -> [u8;32] {
        let mut hash = Sha256::new();
        hash.update(b"spire-lab/env-seed/v1\0");
        // from_be_bytes establishes the framing bound for every instance.
        hash.update(u64::try_from(self.0.len()).expect("validated seed length").to_be_bytes());
        hash.update(&*self.0);
        hash.finalize().into()
    }

    pub fn add_slot(&self, slot: usize) -> Result<Self, EnvError> {
        let capacity = self.0.len().max(size_of::<usize>()).checked_add(1)
            .ok_or_else(|| setup("expanded seed byte length overflow"))?;
        let mut little = reserved(capacity)?;
        let mut carry = slot;
        for byte in self.0.iter().rev() {
            let sum = usize::from(*byte) + (carry & 255);
            little.push((sum & 255) as u8);
            carry = (carry >> 8) + (sum >> 8);
        }
        while carry != 0 {
            little.push((carry & 255) as u8);
            carry >>= 8;
        }
        little.reverse();
        Self::from_be_bytes(&little)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeedBatch {
    Continue,
    Scalar(IntegerSeed),
    PerSlot(Vec<Option<IntegerSeed>>),
}

pub fn expand_seeds(batch: SeedBatch, slots: usize) -> Result<Vec<Option<IntegerSeed>>, EnvError> {
    if let SeedBatch::PerSlot(seeds) = batch {
        if seeds.len() != slots {
            return Err(input(InputReason::Shape, "per-slot seed count does not match batch size"));
        }
        return Ok(seeds);
    }
    let mut seeds = reserved(slots)?;
    for i in 0..slots {
        seeds.push(match &batch {
            SeedBatch::Continue => None,
            SeedBatch::Scalar(seed) => Some(seed.add_slot(i)?),
            SeedBatch::PerSlot(_) => unreachable!("handled before allocation"),
        });
    }
    Ok(seeds)
}
