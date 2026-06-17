# `logic_nih_plug_crypto`

Hashing, big-integer arithmetic and RSA keys ported from JUCE for nih-plug.

This crate provides pure-Rust implementations of JUCE's `juce_crypto` module:

- **`Sha256` / `Sha1` / `Md5`** — streaming hash contexts (with one-shot
  helpers and hex-encoded convenience functions).
- **`BigInteger`** — arbitrary-precision unsigned integer arithmetic:
  parsing, formatting, bit access, modular exponentiation, GCD.
- **`RSAKey`** — RSA key generation, signing and verification using
  SHA-256 + PKCS#1 v1.5 padding.

## Feature flags

| Feature | Default | What it adds |
|---|---|---|
| `sha2` | ✅ | SHA-256 streaming hash + one-shot helpers |
| `sha1` | — | SHA-1 streaming hash + one-shot helpers |
| `md5` | — | MD5 streaming hash + one-shot helpers |
| `bignum` | — | `BigInteger` |
| `rsa` | — | `RSAKey` (also enables `sha2`) |
| `full` | — | All of the above |

Enable exactly what you need to keep your build small:

```toml
[dependencies]
logic_nih_plug_crypto = { version = "0", default-features = false, features = ["sha2"] }
```

## Examples

### Hashing

```rust
use logic_nih_plug_crypto::sha256::{Sha256, sha256_hex};

// One-shot
assert_eq!(
    sha256_hex(b"abc"),
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
);

// Streaming
let mut h = Sha256::new();
h.update(b"hello, ");
h.update(b"world");
assert_eq!(h.finalize_hex(), sha256_hex(b"hello, world"));
```

### Big integers

```rust
use logic_nih_plug_crypto::big_integer::BigInteger;

let n = BigInteger::parse_decimal("123456789012345678901234567890").unwrap();
assert!(n.bit_length() > 64);

// Modular exponentiation: 2^10 mod 1000 = 24
let base = BigInteger::from(2u32);
let exp  = BigInteger::from(10u32);
let modu = BigInteger::from(1000u32);
assert_eq!(base.mod_pow(&exp, &modu).unwrap(), BigInteger::from(24u32));
```

### RSA

```rust
use logic_nih_plug_crypto::rsa_key::RSAKey;

let key = RSAKey::generate(2048).unwrap();
assert!(key.has_private());

let msg  = b"verify this message";
let sig  = key.sign(msg).unwrap();
assert!(key.verify(msg, &sig));

// Hand out a verifier without leaking signing capability:
let verifier = key.into_public_only();
assert!(!verifier.has_private());
assert!(verifier.verify(msg, &sig));
```

## Threading

All functions and methods in this crate are stateless (apart from the
`SHAxxx` streaming contexts, which are not `Sync` by design — `Clone` them
explicitly if you need to share). Safe to call from any thread.

## Security notes

- SHA-1 and MD5 are broken for cryptographic use. Only use them for
  non-security applications (cache keys, content fingerprinting on
  trusted inputs, etc.).
- The minimum supported RSA key size is **2048 bits**. Smaller keys are
  rejected by `RSAKey::generate`.
- The bundled `rsa` crate is **not constant-time** and is vulnerable to
  [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html).
  Don't sign attacker-controlled data on a network-reachable host.
- For cryptographic applications that need stronger guarantees, prefer
  [`ring`](https://crates.io/crates/ring) or
  [`RustCrypto`](https://github.com/RustCrypto) directly.

## License

ISC — same as the parent `nih-plug` project.
