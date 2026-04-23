# ckb-ultraplonk-verifier

An on-chain UltraPlonk zk-SNARK verifier for CKB-VM. Verify [Noir](https://noir-lang.org/) / Barretenberg proofs trustlessly on the [Nervos CKB](https://www.nervos.org/) blockchain.

> **Disclaimer:** As of Barretenberg v0.87.0+, UltraPlonk has been officially deprecated in favor of [UltraHonk](https://barretenberg.aztec.network/docs/). Newer versions of Noir and `bb` default to UltraHonk and will **not** produce proofs compatible with this verifier. This project is intended for **experimentation, learning, and tinkering** with on-chain ZK verification on CKB using Noir <= v1.0.0-beta.1. It is not suitable for production use with current Noir tooling. For production deployments, an UltraHonk-compatible verifier would be needed.

## Why

CKB currently has no production-ready SNARK verifier that runs inside CKB-VM. This blocks any CKB application that needs on-chain ZK verification -private identity proofs, confidential transactions, ZK bridges, and more.

This project fills that gap by packaging [zkVerify's UltraPlonk verifier](https://github.com/zkVerify/ultraplonk_verifier) (pure Rust, `no_std`) as a deployable CKB script.

## Key numbers

| Metric                | Value                         |
| --------------------- | ----------------------------- |
| Binary size           | 147 KB                        |
| Verification cost     | ~103M cycles                  |
| CKB block cycle limit | 3.5B cycles                   |
| Noir versions tested  | v0.30.0 through v1.0.0-beta.1 |

## How it works

1. You write a [Noir](https://noir-lang.org/docs/getting_started/quick_start) circuit and generate a proof off-chain using `nargo` + `bb`
2. You convert the proof/vk to zkVerify format using `noir-cli` (included in the [ultraplonk_verifier](https://github.com/zkVerify/ultraplonk_verifier) repo)
3. You submit a CKB transaction with the proof data in the witness
4. CKB-VM runs this verifier script -valid proof = transaction accepted, invalid proof = rejected

## Witness layout

The script reads its data from witness index 0 of the input group:

| Field      | Size                     | Description                        |
| ---------- | ------------------------ | ---------------------------------- |
| `num_pubs` | 4 bytes (u32 big-endian) | Number of public inputs            |
| `vk`       | 1,632 bytes              | Verification key (zkVerify format) |
| `proof`    | 2,144 bytes              | Proof (zkVerify format)            |
| `pubs`     | `num_pubs * 32` bytes    | Public inputs, 32 bytes each       |

## Build

Requires Rust with the `riscv64imac-unknown-none-elf` target:

```bash
rustup target add riscv64imac-unknown-none-elf
```

Build the contract:

```bash
make build
```

The compiled binary will be at `contracts/ckb-ultraplonk-verifier`.

## Test

Integration tests verify both valid and invalid proofs against a mock CKB environment:

```bash
make test
```

## Generating test vectors

Install `noir-cli` from the [ultraplonk_verifier](https://github.com/zkVerify/ultraplonk_verifier) repo:

```bash
cargo install --features bins --path /path/to/ultraplonk_verifier
```

Convert Barretenberg outputs to zkVerify format:

```bash
# Convert proof (also extracts public inputs)
noir-cli proof-data -n <num_public_inputs> \
  --input-proof <bb_proof> \
  --output-proof <zkv_proof> \
  --output-pubs <zkv_pubs>

# Convert verification key
noir-cli key --input <bb_vk> --output <zkv_vk>

# Verify locally
noir-cli verify --proof <zkv_proof> --pubs <zkv_pubs> --key <zkv_vk>
```

## Architecture notes

- The verifier is a pure Rust port of Noir's UltraPlonk Solidity verifier, maintained by [Horizen Labs / zkVerify](https://github.com/zkVerify/ultraplonk_verifier)
- It uses `ark-bn254-ext` with the default `()` implementation, which delegates to standard [arkworks](https://github.com/arkworks-rs) BN254 arithmetic -no host functions or SIMD required
- The `CurveHooks` trait is generic, allowing future optimization (e.g., CKB syscall-accelerated pairing) without changing the verifier logic
- Recursive proofs are not currently supported

## Noir / Barretenberg version compatibility

Aztec has been migrating Barretenberg toward UltraHonk internally. This verifier targets the **UltraPlonk** proving path, which `bb` still supports. Pin your Noir/bb version and use the UltraPlonk backend to ensure compatibility.

Tested versions: v0.30.0, v0.31.0, v0.32.0, v0.33.0, v0.34.0, v0.35.0, v0.36.0, v0.37.0, v0.38.0, v0.39.0, v1.0.0-beta.0, v1.0.0-beta.1.

## License

MIT OR Apache-2.0

## Acknowledgements

- [zkVerify / Horizen Labs](https://github.com/zkVerify) for the `ultraplonk_verifier` crate
- [Nervos](https://www.nervos.org/) for CKB-VM and `ckb-std`
- [Aztec / Noir](https://noir-lang.org/) for the proving system
