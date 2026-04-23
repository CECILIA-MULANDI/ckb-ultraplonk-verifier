#![no_std]
#![no_main]

use ckb_std::default_alloc;
use ckb_std::ckb_constants::Source;
use ckb_std::high_level::load_witness;
use ultraplonk_no_std::{verify, PROOF_SIZE, PUBS_SIZE, VK_SIZE};

ckb_std::entry!(main);
default_alloc!();

/// Error codes returned by the script.
/// Negative values indicate verification failure.
const ERROR_WITNESS_LOAD: i8 = -1;
const ERROR_WITNESS_TOO_SHORT: i8 = -2;
const ERROR_WITNESS_LENGTH_MISMATCH: i8 = -3;
const ERROR_VERIFICATION_FAILED: i8 = -4;

/// Witness data layout:
///
/// | Field       | Size                      | Description                         |
/// |-------------|---------------------------|-------------------------------------|
/// | num_pubs    | 4 bytes (u32 big-endian)  | Number of public inputs             |
/// | vk          | 1632 bytes                | Verification key (zkVerify format)  |
/// | proof       | 2144 bytes                | Proof data (zkVerify format)        |
/// | pubs        | num_pubs * 32 bytes       | Public inputs (32 bytes each)       |
///
fn main() -> i8 {
    match run() {
        Ok(()) => 0,
        Err(code) => code,
    }
}

fn run() -> Result<(), i8> {
    let witness = load_witness(0, Source::GroupInput).map_err(|_| ERROR_WITNESS_LOAD)?;

    if witness.len() < 4 {
        return Err(ERROR_WITNESS_TOO_SHORT);
    }

    let num_pubs =
        u32::from_be_bytes([witness[0], witness[1], witness[2], witness[3]]) as usize;
    let expected_len = 4 + VK_SIZE + PROOF_SIZE + (num_pubs * PUBS_SIZE);

    if witness.len() != expected_len {
        return Err(ERROR_WITNESS_LENGTH_MISMATCH);
    }

    let vk_start = 4;
    let proof_start = vk_start + VK_SIZE;
    let pubs_start = proof_start + PROOF_SIZE;

    let vk = &witness[vk_start..proof_start];
    let proof = &witness[proof_start..pubs_start];

    let pubs: alloc::vec::Vec<[u8; PUBS_SIZE]> = (0..num_pubs)
        .map(|i| {
            let mut buf = [0u8; PUBS_SIZE];
            buf.copy_from_slice(&witness[pubs_start + i * PUBS_SIZE..pubs_start + (i + 1) * PUBS_SIZE]);
            buf
        })
        .collect();

    verify::<()>(vk, proof, &pubs).map_err(|_| ERROR_VERIFICATION_FAILED)
}
