use std::str::{from_utf8, FromStr};

use risc0_zkvm::guest::env;
use rustreexo::accumulator::node_hash::NodeHash;
use rustreexo::accumulator::proof::Proof;
use rustreexo::accumulator::stump::Stump;
use sha2::{Digest, Sha512_256};

use bitcoin::XOnlyPublicKey;
use bitcoin::consensus::encode::serialize;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{ScriptBuf, TxOut, Amount};
use k256::schnorr;
use k256::schnorr::signature::Verifier;

fn main() {
    let secp = Secp256k1::verification_only();

    // read the input
    let msg_bytes: Vec<u8> = env::read();
    let priv_key: schnorr::SigningKey = env::read();
    let s: Stump = env::read();
    let proof: Proof = env::read();
    let sig_bytes: Vec<u8> = env::read();

    let internal_key = priv_key.verifying_key();

    // We'll check that the given public key corresponds to an output in the utxo set.
    let pubx = XOnlyPublicKey::from_slice(internal_key.to_bytes().as_slice()).unwrap();
    let script_pubkey = ScriptBuf::new_p2tr(&secp, pubx, None);
    let utxo = TxOut {
        value: Amount::ZERO,
        script_pubkey,
    };

    let serialized_txout = serialize(&utxo);

    let mut hasher = Sha512_256::new();
    hasher.update(&serialized_txout);
    let result = hasher.finalize();
    let myhash = NodeHash::from_str(hex::encode(result).as_str()).unwrap();

    // Assert it is in the set.
    assert_eq!(s.verify(&proof, &[myhash]), Ok(true));

    let mut hasher = Sha512_256::new();
    hasher.update(&priv_key.to_bytes());
    let sk_hash = hex::encode(hasher.finalize());
    let msg = from_utf8(msg_bytes.as_slice()).unwrap();

    let schnorr_sig = schnorr::Signature::try_from(sig_bytes.as_slice()).unwrap();

    internal_key
        .verify(msg_bytes.as_slice(), &schnorr_sig)
        .expect("schnorr verification failed");

    // write public output to the journal
    env::commit(&s);
    env::commit(&sk_hash);
    env::commit(&msg);
}
