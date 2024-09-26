use std::str::FromStr;

use risc0_zkvm::guest::env;
use rustreexo::accumulator::node_hash::NodeHash;
use rustreexo::accumulator::proof::Proof;
use rustreexo::accumulator::stump::Stump;
use sha2::{Digest, Sha512_256};

use bitcoin::XOnlyPublicKey;
use bitcoin::key::Keypair;
use bitcoin::consensus::encode::serialize;
use bitcoin::secp256k1::{Secp256k1, Scalar, SecretKey};
use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::{ScriptBuf, TxOut, Amount};
use k256::schnorr;
use k256::schnorr::signature::Verifier;

fn main() {
    let secp = Secp256k1::new();

    // read the input
    let msg_bytes: Vec<u8> = env::read();
    let priv_key: SecretKey = env::read();
    let s: Stump = env::read();
    let proof: Proof = env::read();
    let signature: Signature = env::read();

    let keypair = Keypair::from_secret_key(&secp, &priv_key);
    let (internal_key, _) = keypair.x_only_public_key();

    // We'll check that the given public key corresponds to an output in the utxo set.
    let script_pubkey = ScriptBuf::new_p2tr(&secp, internal_key, None);
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
    hasher.update(&priv_key.secret_bytes());
    let sk_hash = hex::encode(hasher.finalize());

    let pub_bytes = internal_key.serialize();
    let verifying_key = schnorr::VerifyingKey::from_bytes(&pub_bytes).unwrap();

    let sig_bytes = signature.serialize();
    let schnorr_sig = schnorr::Signature::try_from(sig_bytes.as_slice()).unwrap();

    verifying_key
        .verify(msg_bytes.as_slice(), &schnorr_sig)
        .expect("schnorr verification failed");

    // write public output to the journal
    env::commit(&s);
    env::commit(&sk_hash);
}
