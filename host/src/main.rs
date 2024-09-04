// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};

use std::str::FromStr;
use std::vec;

use rustreexo::accumulator::node_hash::NodeHash;
use rustreexo::accumulator::proof::Proof;
use rustreexo::accumulator::stump::Stump;

use bitcoin::hashes::Hash;
use bitcoin::key::{Keypair, TapTweak, TweakedKeypair, UntweakedPublicKey};
use bitcoin::locktime::absolute;
use bitcoin::secp256k1::{rand, Message, Secp256k1, SecretKey, Signing, Verification};
use bitcoin::sighash::{Prevouts, SighashCache, TapSighashType};
use bitcoin::{
    transaction, Address, Amount, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
    Txid, Witness,
};
const DUMMY_UTXO_AMOUNT: Amount = Amount::from_sat(20_000_000);
const SPEND_AMOUNT: Amount = Amount::from_sat(5_000_000);
const CHANGE_AMOUNT: Amount = Amount::from_sat(14_999_000); // 1000 sat fee.

fn senders_keys<C: Signing>(secp: &Secp256k1<C>) -> Keypair {
    let sk = SecretKey::new(&mut rand::thread_rng());
    Keypair::from_secret_key(secp, &sk)
}

fn receivers_address() -> Address {
    "bc1p0dq0tzg2r780hldthn5mrznmpxsxc0jux5f20fwj0z3wqxxk6fpqm7q0va"
        .parse::<Address<_>>()
        .expect("a valid address")
        .require_network(Network::Bitcoin)
        .expect("valid address for mainnet")
}

fn dummy_unspent_transaction_output<C: Verification>(
    secp: &Secp256k1<C>,
    internal_key: UntweakedPublicKey,
) -> (OutPoint, TxOut) {
    let script_pubkey = ScriptBuf::new_p2tr(secp, internal_key, None);

    let out_point = OutPoint {
        txid: Txid::all_zeros(), // Obviously invalid.
        vout: 0,
    };

    let utxo = TxOut {
        value: DUMMY_UTXO_AMOUNT,
        script_pubkey,
    };

    (out_point, utxo)
}

fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let secp = Secp256k1::new();

    // Get a keypair we control. In a real application these would come from a stored secret.
    let keypair = senders_keys(&secp);
    let (internal_key, _parity) = keypair.x_only_public_key();

    // Get an unspent output that is locked to the key above that we control.
    // In a real application these would come from the chain.
    let (dummy_out_point, dummy_utxo) = dummy_unspent_transaction_output(&secp, internal_key);

    // Get an address to send to.
    let address = receivers_address();

    // The input for the transaction we are constructing.
    let input = TxIn {
        previous_output: dummy_out_point, // The dummy output we are spending.
        script_sig: ScriptBuf::default(), // For a p2tr script_sig is empty.
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::default(), // Filled in after signing.
    };

    // The spend output is locked to a key controlled by the receiver.
    let spend = TxOut {
        value: SPEND_AMOUNT,
        script_pubkey: address.script_pubkey(),
    };

    // The change output is locked to a key controlled by us.
    let change = TxOut {
        value: CHANGE_AMOUNT,
        script_pubkey: ScriptBuf::new_p2tr(&secp, internal_key, None), // Change comes back to us.
    };

    // The transaction we want to sign and broadcast.
    let mut unsigned_tx = Transaction {
        version: transaction::Version::TWO,  // Post BIP-68.
        lock_time: absolute::LockTime::ZERO, // Ignore the locktime.
        input: vec![input],                  // Input goes into index 0.
        output: vec![spend, change],         // Outputs, order does not matter.
    };
    let input_index = 0;

    // Get the sighash to sign.

    let sighash_type = TapSighashType::Default;
    let prevouts = vec![dummy_utxo];
    let prevouts = Prevouts::All(&prevouts);

    let mut sighasher = SighashCache::new(&mut unsigned_tx);
    let sighash = sighasher
        .taproot_key_spend_signature_hash(input_index, &prevouts, sighash_type)
        .expect("failed to construct sighash");

    // Sign the sighash using the secp256k1 library (exported by rust-bitcoin).
    let tweaked: TweakedKeypair = keypair.tap_tweak(&secp, None);
    let msg = Message::from_digest(sighash.to_byte_array());
    let signature = secp.sign_schnorr(&msg, &tweaked.to_inner());

    // Update the witness stack.
    let signature = bitcoin::taproot::Signature {
        signature,
        sighash_type,
    };
    sighasher
        .witness_mut(input_index)
        .unwrap()
        .push(&signature.to_vec());

    // Get the signed transaction.
    let tx = sighasher.into_transaction();

    // Verify the signature
    let pubkey = tweaked.to_inner().x_only_public_key().0;
    let is_valid = secp
        .verify_schnorr(&signature.signature, &msg, &pubkey)
        .is_ok();

    if is_valid {
        println!("Signature is valid!");
    } else {
        println!("Signature is invalid!");
    }

    // BOOM! Transaction signed and ready to broadcast.
    println!("{:#?}", tx);

    //-------------------------------------------
    // These are the utxos that we want to add to the Stump, in Bitcoin, these would be the
    // UTXOs created in a block.
    // If we assume this is the very first block, then the Stump is empty, and we can just add
    // the utxos to it. Assuming a coinbase with two outputs, we would have the following utxos:
    let utxos = vec![
        NodeHash::from_str("b151a956139bb821d4effa34ea95c17560e0135d1e4661fc23cedc3af49dac42")
            .unwrap(),
        NodeHash::from_str("d3bd63d53c5a70050a28612a2f4b2019f40951a653ae70736d93745efb1124fa")
            .unwrap(),
    ];
    // Create a new Stump, and add the utxos to it. Notice how we don't use the full return here,
    // but only the Stump. To understand what is the second return value, see the documentation
    // for `Stump::modify`, or the proof-update example.
    let s0 = Stump::new()
        .modify(&utxos, &[], &Proof::default())
        .unwrap()
        .0;
    // Create a proof that the first utxo is in the Stump.
    let proof = Proof::new(vec![0], vec![utxos[1]]);
    assert_eq!(s0.verify(&proof, &[utxos[0]]), Ok(true));

    // Now we want to update the Stump, by removing the first utxo, and adding a new one.
    // This would be in case we received a new block with a transaction spending the first utxo,
    // and creating a new one.
    //    let new_utxo =
    //        NodeHash::from_str("d3bd63d53c5a70050a28612a2f4b2019f40951a653ae70736d93745efb1124fa")
    //            .unwrap();
    //    let s = s0.modify(&[new_utxo], &[utxos[0]], &proof).unwrap().0;
    //    // Now we can verify that the new utxo is in the Stump, and the old one is not.
    //    let new_proof = Proof::new(vec![2], vec![new_utxo]);
    //    assert_eq!(s.verify(&new_proof, &[new_utxo]), Ok(true));
    //    assert_eq!(s.verify(&proof, &[utxos[0]]), Ok(false));
    //-------------------------------------------

    // An executor environment describes the configurations for the zkVM
    // including program inputs.
    // An default ExecutorEnv can be created like so:
    // `let env = ExecutorEnv::builder().build().unwrap();`
    // However, this `env` does not have any inputs.
    //
    // To add guest input to the executor environment, use
    // ExecutorEnvBuilder::write().
    // To access this method, you'll need to use ExecutorEnv::builder(), which
    // creates an ExecutorEnvBuilder. When you're done adding input, call
    // ExecutorEnvBuilder::build().

    // For example:
    let input: u32 = 15 * u32::pow(2, 27) + 1;
    let env = ExecutorEnv::builder()
        .write(&s0).unwrap()
        .write(&utxos[0]).unwrap()
        .write(&proof).unwrap()
        //.write(&signature).unwrap()
        //.write(&sighash).unwrap()
        //.write(&pubkey).unwrap()
        .build().unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Proof information by proving the specified ELF binary.
    // This struct contains the receipt along with statistics about execution of the guest
    let prove_info = prover.prove(env, METHOD_ELF).unwrap();

    // extract the receipt.
    let receipt = prove_info.receipt;

    // TODO: Implement code for retrieving receipt journal here.

    // For example:
    let _output: Stump = receipt.journal.decode().unwrap();
    println!("journal: {:?}", _output);

    let receipt_bytes = bincode::serialize(&receipt).unwrap();
    println!("receipt ({}): {}", receipt_bytes.len(), hex::encode(receipt_bytes));


    // The receipt was verified at the end of proving, but the below code is an
    // example of how someone else could verify this receipt.
    receipt.verify(METHOD_ID).unwrap();
}