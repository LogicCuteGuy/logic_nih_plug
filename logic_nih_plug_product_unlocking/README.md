# `logic_nih_plug_product_unlocking`

Product-unlocking (server-side keyfile generator + client-side state
machine) ported from
[JUCE `juce_product_unlocking`](https://docs.juce.com/master/group__juce__product__unlocking.html)
for [nih-plug](https://github.com/robbert-vdh/nih-plug).

## What's in here

- **`key_generation`** — `KeyGeneration`-style helpers that mint the
  human-readable keyfile text your server sends to a user to unlock a
  product.
  - `generate_key_file(app_name, user_email, user_name, machine_numbers, &private_key) -> Result<String, KeyFileError>`
  - `generate_expiring_key_file(app_name, user_email, user_name, machine_numbers, expiry_time_ms, &private_key) -> Result<String, KeyFileError>`
  - `decrypt_key_file(keyfile_text, &public_key) -> Result<KeyFileData, KeyFileError>`
  - `LicenseResult` — the JUCE-compatible error-message constants
- **`online_unlock_status`** — `OnlineUnlockStatus`-style state machine
  that runs on the client. Generic over a user-supplied `UnlockStore`
  trait that supplies the per-app bits (product ID, public key,
  persistence, machine IDs).
- **`machine_id`** — `MachineIDUtilities` helpers (platform prefix,
  encoded machine IDs).

## Keyfile format

A keyfile is a block of plain text:

```text
Keyfile for <app_name>
User: <user_name>
Email: <user_email>
Machine numbers: <comma- or semicolon-separated>
Created: <YYYY-MM-DD HH:MM:SS UTC>

#<70-char hex chunk>
<70-char hex chunk>
...
```

The hex after the last `#` is the raw-RSA ciphertext of a single-line
`<key …/>` XML element holding the same metadata as the comment header.
The encryption is textbook RSA (`m^d mod n` / `c^e mod n`), matching
`juce::KeyGeneration` line-for-line.

## Usage

### Server side: minting a keyfile

```rust,no_run
use logic_nih_plug_product_unlocking::key_generation::generate_key_file;
use logic_nih_plug_crypto::rsa_key::RSAKey;

let private_key = RSAKey::generate(2048).expect("key generation");
let keyfile_text = generate_key_file(
    "MyApp",                 // app name
    "joe@foo.bar",           // user email
    "Joe Bloggs",            // user name
    "MACHINEID123",          // machine ID
    &private_key,
).expect("keyfile generation");
// Send `keyfile_text` to the user via email / your marketplace.
```

### Client side: state machine

```rust,no_run
use logic_nih_plug_data::ValueTree;
use logic_nih_plug_product_unlocking::online_unlock_status::{
    OnlineUnlockStatus, UnlockStore,
};
use logic_nih_plug_crypto::rsa_key::RSAKey;

struct MyStore { public_key: RSAKey, product_id: String, persisted: String }
impl UnlockStore for MyStore {
    fn get_product_id(&self) -> &str { &self.product_id }
    fn does_product_id_match(&self, id: &str) -> bool { id == self.product_id }
    fn get_public_key(&self) -> &RSAKey { &self.public_key }
    fn save_state(&mut self, state: &str) { self.persisted = state.to_string(); }
    fn get_state(&self) -> String { self.persisted.clone() }
    fn get_website_name(&self) -> &str { "My Store" }
    fn get_server_authentication_url(&self) -> &str { "https://example.com/auth" }
    fn read_reply_from_webserver(&self, _email: &str, _password: &str) -> String { String::new() }
}

let mut status = OnlineUnlockStatus::new(MyStore {
    public_key: /* your embedded public key */ todo!(),
    product_id: "MY-PRODUCT-1".to_owned(),
    persisted: String::new(),
});
status.load();
if !status.is_unlocked() {
    // Show an unlock UI, let the user paste / drop in a keyfile, etc.
}
```

## Feature flags

| Feature | Default | What it adds |
|---|---|---|
| `key_generation` | ✅ | `generate_key_file`, `generate_expiring_key_file`, `decrypt_key_file`, `KeyFileData` |
| `online_unlock_status` | ✅ | `OnlineUnlockStatus`, `UnlockStore` trait, `UnlockResult`, `UnlockState` |
| `full` | — | All of the above |

## Security notes

- **Use at least 2048-bit RSA keys.** Smaller moduli can be brute-forced
  by attackers who grab a single keyfile.
- **The textbook RSA used here is not semantically secure on its own.**
  An attacker who captures two keyfiles for the same product+user can
  use the `gcd` of `(m1^e - m2^e)` and `n` to factor `n`. Mitigate by
  including a per-keyfile nonce inside the XML payload (`<date>` is
  already per-keyfile to within `SystemTime::now()`'s precision; add
  random padding if you need stronger guarantees).
- The same `RUSTSEC-2023-0071` Marvin Attack caveat applies as in
  `logic_nih_plug_crypto` — the underlying `rsa` crate's modular
  exponentiation is not constant-time. Don't run key verification on
  a network-attacker-reachable host.

## License

ISC, same as the rest of the workspace.