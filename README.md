# utxozkp
`utxozkp` is a proof of concept tool for proving Bitcoin UTXO set inclusion in
zero knowledge.

## Applications 
Since unspent transaction outputs is a scare resource, having a way of
cryptographically prove you own one without revealing anything about the output
is useful for all sorts of anti-DOS applications.

Examples are:
- Lightning channel announcements: prove the channel exist without revealing
  it.
- Proof-of-reserves: prove you control a certain amount of coins without
  reealing which ones.
- etc

## Architecture 
The tool works with the UTXO set dump from Bitcoin Core. It uses this dump to
create a [Utreexo](https://dci.mit.edu/utreexo) representation of the UTXO set
and a proof for inclusion of the given UTXO in this set.

The prover then signs a message using the private key for the output with
public key `P`, proving that he controls the coins. In order to not have this
signature reveal which output it signs for, we have the prover tweak the key
with a random value before signing.

The prover then creates a ZK-STARK proof using the [Risc0 ZKVM](https://github.com/risc0/risc0) 
that proves the following:

- The prover has a public key `P` and a blinding value `b` that together forms
  the tweaked key `P' = P + b*G`. `P'` is shown to the verifier.
- The prover has a proof showing that the public key P is found in the Utreexo
  set. The Utreexo root is shown to the verifier.

In addition the verifier is given a signature S that signs a message M for the
key `P'`. Together with the STARK proof this is convincing the verifier that
the prover has the private key to the output in the UTXO set.

Note that in theory we could also have done the signature check itself in the
ZKVM, but signature checks are expensive in that environment, and needs more
work to be practical.

## Quick start

### Requirements 
Install the `risc0` toolchain: https://github.com/risc0/risc0?tab=readme-ov-file#getting-started

### Proof creation
Create an address to send some testnet3 coins to:
```bash
$ cargo run --release -- --priv-key "new"
priv: 4b55c185428041cff1d3cf9044ad18c14fa79a927fa242d2b6ee7582f59b9581
pub: 1fa2ab3dfcdeaba8d8253c6e7ef49135f36cb4c4c8515c5579f3010e2999e3b5
address: tb1p7677cydywmtk67vfvxrwlh2g7g4cz7ha4z6x6v4u4kj8xyyu3n0srfaj4g
```

You can now fund the given address with some tBTC, then wait for the
transaction to confirm and Bitcoin Core to sync to the block (feel free to use
the above private key for testing, but please don't spend the coins).

Now get a dump of the UTXO set from the Bitcoin Core: 

```bash
$ bitcoin-cli -testnet dumptxoutset testnet_utxoset.dat
```

This will take few minutes, as the UTXO is rather large. But now we got what we
need to run `utxozkp`, and presumably our output is contained in it (replace
folder with Bitcoin Core directory):

```bash
$$  cargo run --release -- --utxoset-file "<path_to_bitcoin>/testnet3/testnet_utxoset.dat" --priv-key "4b55c185428041cff1d3cf9044ad18c14fa79a927fa242d2b6ee7582f59b9581" --msg "messsage to sign" --receipt-file receipt.bin --utreexo-file utreexo_stump.bin --prove
```

This command will read the UTXO set, and create a ZK proof as detailed in the
Architecture section. The `receipt.bin` file contains this proof, while the
`utreexo_stump.bin` file contains the Utreexo roots (these can be independently
created by the verifier).


### Verification
The proof can be verified using

```bash
$ cargo run --release -- --msg "messsage to sign" --receipt-file receipt.bin --utreexo-file utreexo_stump.bin 
```

## Benchmarks, Apple M1 Max
- Proving time is about 5:50 minutes (not counting loading the UTXO set into
  memory).
- Verification time is ~250 ms.
- Proof size is 1.4 MB.

## Limitations
This is a rough first draft of how a tool like this could look like. It has
plenty of known limitations and should absolutely not be used with private keys
controlling real (mainnet) coins.

A non-exhaustive list (some of these could be relatively easy to fix):

- Only supports taproot keyspend outputs.
- Only supports testnet3.
- Only proving existence, selectively revealing more about the output is not
  supported.
- Proving time is not optimized.
- Proof size is not attempted optimized.
- Private key must be hot.
- ... and many more.

