use ckb_testtool::builtin::ALWAYS_SUCCESS;
use ckb_testtool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use ckb_testtool::context::Context;

const VERIFIER_BIN: &str =
    "../target/riscv64imac-unknown-none-elf/release/ckb-ultraplonk-verifier";

// 500 billion cycles — BN254 pairing is expensive on RISC-V.
// We start high to find the actual cost, then tighten.
const MAX_CYCLES: u64 = 500_000_000_000;

/// Build the witness payload: [num_pubs (4 bytes BE) | vk | proof | pubs...]
fn build_witness(vk: &[u8], proof: &[u8], pubs: &[&[u8]]) -> Bytes {
    let num_pubs = pubs.len() as u32;
    let mut data = Vec::new();
    data.extend_from_slice(&num_pubs.to_be_bytes());
    data.extend_from_slice(vk);
    data.extend_from_slice(proof);
    for p in pubs {
        data.extend_from_slice(p);
    }
    Bytes::from(data)
}

fn setup() -> (Context, OutPoint, Script) {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let always_success_lock = context
        .build_script(&always_success_out_point, Bytes::new())
        .expect("always_success lock script");

    let verifier_bin = std::fs::read(VERIFIER_BIN)
        .expect("verifier binary not found — run `make build` first");
    let verifier_out_point = context.deploy_cell(Bytes::from(verifier_bin));

    (context, verifier_out_point, always_success_lock)
}

/// Valid proof should pass verification.
#[test]
fn test_valid_proof_passes() {
    let (mut context, verifier_out_point, always_success_lock) = setup();

    let verifier_script = context
        .build_script(&verifier_out_point, Bytes::new())
        .expect("verifier script");

    // Load test vectors (zkVerify format, generated via noir-cli)
    let vk = include_bytes!("../../test-vectors/vk.bin");
    let proof = include_bytes!("../../test-vectors/proof.bin");
    let pubs = include_bytes!("../../test-vectors/pubs.bin");

    assert_eq!(vk.len(), 1632, "unexpected vk size");
    assert_eq!(proof.len(), 2144, "unexpected proof size");
    assert_eq!(pubs.len(), 32, "unexpected pubs size");

    let witness = build_witness(vk, proof, &[pubs.as_slice()]);

    // Create an input cell locked by the verifier script
    let input_cell = context.create_cell(
        CellOutput::default()
            .as_builder()
            .capacity(1000u64)
            .lock(verifier_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_cell)
        .build();

    // Output cell with always_success lock (just needs somewhere to go)
    let output = CellOutput::default()
        .as_builder()
        .capacity(900u64)
        .lock(always_success_lock)
        .build();

    let tx = TransactionBuilder::default()
        .input(input)
        .output(output)
        .output_data(Bytes::new().pack())
        .witness(witness.pack())
        .build();

    let tx = context.complete_tx(tx);

    let result = context.verify_tx(&tx, MAX_CYCLES);
    match &result {
        Ok(cycles) => println!("Verification passed! Cycles consumed: {}", cycles),
        Err(e) => {
            // Dump failed tx for debugging
            let mock_tx = context.dump_tx(&tx).expect("dump tx");
            let json = serde_json::to_string_pretty(&mock_tx).expect("json");
            std::fs::create_dir_all("failed_txs").ok();
            let path = format!("failed_txs/0x{:x}.json", tx.hash());
            std::fs::write(&path, json).expect("write failed tx");
            println!("Failed tx written to {path}");
            panic!("Verification failed: {e}");
        }
    }
    result.unwrap();
}

/// Tampered proof should fail verification.
#[test]
fn test_invalid_proof_fails() {
    let (mut context, verifier_out_point, always_success_lock) = setup();

    let verifier_script = context
        .build_script(&verifier_out_point, Bytes::new())
        .expect("verifier script");

    let vk = include_bytes!("../../test-vectors/vk.bin");
    let mut bad_proof = include_bytes!("../../test-vectors/proof.bin").to_vec();
    let pubs = include_bytes!("../../test-vectors/pubs.bin");

    // Tamper with the proof
    bad_proof[0] ^= 0xff;
    bad_proof[1] ^= 0xff;

    let witness = build_witness(vk, &bad_proof, &[pubs.as_slice()]);

    let input_cell = context.create_cell(
        CellOutput::default()
            .as_builder()
            .capacity(1000u64)
            .lock(verifier_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_cell)
        .build();

    let output = CellOutput::default()
        .as_builder()
        .capacity(900u64)
        .lock(always_success_lock)
        .build();

    let tx = TransactionBuilder::default()
        .input(input)
        .output(output)
        .output_data(Bytes::new().pack())
        .witness(witness.pack())
        .build();

    let tx = context.complete_tx(tx);

    let result = context.verify_tx(&tx, MAX_CYCLES);
    assert!(result.is_err(), "Tampered proof should have been rejected");
    println!("Tampered proof correctly rejected");
}
