use std::vec;
use std::str::FromStr;

use risc0_zkvm::guest::env;
use rustreexo::accumulator::node_hash::NodeHash;
use rustreexo::accumulator::proof::Proof;
use rustreexo::accumulator::stump::Stump;

use bitcoin::hashes::Hash;
use bitcoin::XOnlyPublicKey;
//use bitcoin::key::{Keypair, TapTweak, TweakedKeypair, UntweakedPublicKey};
//use bitcoin::locktime::absolute;
//use bitcoin::secp256k1::{rand, Message, Secp256k1, SecretKey, Signing, Verification};
use bitcoin::secp256k1::{Secp256k1, Message};
use bitcoin::TapSighash;
//use bitcoin::sighash::{TapSighashType};
//use bitcoin::sighash::{Prevouts, SighashCache, TapSighashType};
//use bitcoin::{
//    transaction, Address, Amount, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
//    Txid, Witness,
//};


fn main() {
    // TODO: Implement your guest code here
 //   let secp = Secp256k1::verification_only();

    // TODO: take utxo set root hash, proof of inlusion,
    // verify the proof
    // write utxp root hash as public data. Proves that one has a proof for some utxo in that set.

    // read the input
    let s: Stump = env::read();
    let utxo: NodeHash = env::read();
    let proof: Proof = env::read();
    //let signature: bitcoin::taproot::Signature = env::read();
    //let sighash: TapSighash = env::read();
    //let msg = Message::from_digest(sighash.to_byte_array());
    //let pubkey: XOnlyPublicKey = env::read();

//    let is_valid = secp
//        .verify_schnorr(&signature.signature, &msg, &pubkey)
//        .is_ok();

//    if is_valid {
//        println!("Signature is valid!");
//    } else {
//        println!("Signature is invalid!");
//    }


    // TODO: create an output and sign for that output, them prove it is in the utxo set.

    // These are the utxos that we want to add to the Stump, in Bitcoin, these would be the
    // UTXOs created in a block.
    // If we assume this is the very first block, then the Stump is empty, and we can just add
    // the utxos to it. Assuming a coinbase with two outputs, we would have the following utxos:
//    let utxos = vec![
//        NodeHash::from_str("b151a956139bb821d4effa34ea95c17560e0135d1e4661fc23cedc3af49dac42")
//            .unwrap(),
//        NodeHash::from_str("d3bd63d53c5a70050a28612a2f4b2019f40951a653ae70736d93745efb1124fa")
//            .unwrap(),
//    ];
    // Create a new Stump, and add the utxos to it. Notice how we don't use the full return here,
    // but only the Stump. To understand what is the second return value, see the documentation
    // for `Stump::modify`, or the proof-update example.
    //let s = Stump::new()
    //    .modify(&utxos, &[], &Proof::default())
    //    .unwrap()
    //    .0;
    // Create a proof that the first utxo is in the Stump.
    //let proof = Proof::new(vec![0], vec![utxos[1]]);
    assert_eq!(s.verify(&proof, &[utxo]), Ok(true));

    // Now we want to update the Stump, by removing the first utxo, and adding a new one.
    // This would be in case we received a new block with a transaction spending the first utxo,
    // and creating a new one.
//    let new_utxo =
//        NodeHash::from_str("d3bd63d53c5a70050a28612a2f4b2019f40951a653ae70736d93745efb1124fa")
//            .unwrap();
//    let s = s.modify(&[new_utxo], &[utxos[0]], &proof).unwrap().0;
//    // Now we can verify that the new utxo is in the Stump, and the old one is not.
//    let new_proof = Proof::new(vec![2], vec![new_utxo]);
//    assert_eq!(s.verify(&new_proof, &[new_utxo]), Ok(true));
//    assert_eq!(s.verify(&proof, &[utxos[0]]), Ok(false));

    // TODO: do something with the input

    // write public output to the journal
    env::commit(&s);
}
