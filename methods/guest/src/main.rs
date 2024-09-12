use std::vec;
use std::str::FromStr;

use risc0_zkvm::guest::env;
use rustreexo::accumulator::node_hash::NodeHash;
use rustreexo::accumulator::proof::Proof;
use rustreexo::accumulator::stump::Stump;
use sha2::{Digest, Sha512_256};

use bitcoin::hashes::Hash;
use bitcoin::XOnlyPublicKey;
use bitcoin::consensus::encode::serialize;
use bitcoin::secp256k1::{Secp256k1, Message, Scalar};
use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::TapSighash;
use bitcoin::{TapTweakHash, ScriptBuf, TxOut, Amount};

fn main() {
    let secp = Secp256k1::verification_only();

    // read the input
    let s: Stump = env::read();
    let proof: Proof = env::read();
    let internal_key: XOnlyPublicKey = env::read();
    let blinding_bytes: [u8; 32] = env::read();
    let signature: Signature = env::read();

    // We'll check that the given public key corresponds to an output in the utxo set.
    let mut hasher = Sha512_256::new();
    let script_pubkey = ScriptBuf::new_p2tr(&secp, internal_key, None);
    let utxo = TxOut {
        value: Amount::ZERO,
        script_pubkey,
    };

    let serialized_txout = serialize(&utxo);
    hasher.update(&serialized_txout);
    let result = hasher.finalize();
    let myhash = NodeHash::from_str(hex::encode(result).as_str()).unwrap();

    // Assert it is in the set.
    assert_eq!(s.verify(&proof, &[myhash]), Ok(true));

    // Blind the public key before commiting it to the public inputs.
    let blinding_scalar = Scalar::from_be_bytes(blinding_bytes).unwrap();
    let blinded_pubkey = internal_key.add_tweak(&secp, &blinding_scalar).unwrap().0;

    // write public output to the journal
    env::commit(&s);
    env::commit(&signature);
    env::commit(&blinded_pubkey);
}
