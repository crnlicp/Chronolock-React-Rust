# Encryption and Decryption Architecture in Chronolock

## Table of Contents

1. [Overview](#overview)
2. [Architecture Components](#architecture-components)
3. [Encryption Flow](#encryption-flow)
4. [Decryption Flow](#decryption-flow)
5. [Cryptographic Algorithms](#cryptographic-algorithms)
6. [Security Model](#security-model)
7. [Key Management](#key-management)
8. [Implementation Details](#implementation-details)
9. [Code Examples](#code-examples)
10. [Security Considerations](#security-considerations)

---

## Overview

Chronolock implements a sophisticated multi-layered encryption system that combines:

- **AES-GCM** for data encryption (symmetric encryption)
- **IBE (Identity-Based Encryption)** for access control
- **VetKD (Verifiable Encrypted Threshold Key Derivation)** for time-locked decryption
- **Internet Computer Protocol** for decentralized key management

This system ensures that encrypted content (metadata and media files) can only be decrypted by authorized users after a specific unlock time, without relying on centralized key storage or trusted third parties.

---

## Architecture Components

### 1. Frontend Components

- **Web Crypto API**: Browser-native cryptographic operations (AES-GCM encryption/decryption)
- **@dfinity/vetkeys**: IBE and VetKD operations for identity-based encryption
- React components for encryption/decryption UI

### 2. Backend Components (IC Canister)

- **VetKD System**: Time-based key derivation on Internet Computer
- **Management Canister**: IC's built-in management functions for VetKD
- Stable storage for encrypted data

### 3. Cryptographic Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    User Content Layer                        │
│         (Original metadata + media files)                    │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────┐
│              Layer 1: AES-GCM Encryption                     │
│  • Symmetric encryption of content                           │
│  • Random AES-256 key generated per Chronolock              │
│  • Random IV (12 bytes) per encryption                       │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────┐
│         Layer 2: IBE (Identity-Based Encryption)             │
│  • Encrypts the AES key using IBE                           │
│  • Identity format: "user_principal:unlock_time"            │
│  • Or just "unlock_time" for public chronolocks             │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────┐
│           Layer 3: VetKD Time-Lock System                    │
│  • Time-based key derivation on IC                          │
│  • Decryption key only available after unlock_time          │
│  • Transport encryption for key delivery                     │
└─────────────────────────────────────────────────────────────┘
```

---

## Encryption Flow

### Step 1: AES Key Generation

```typescript
// Location: src/frontend/src/pages/Create.tsx
const generateKey = async (): Promise<CryptoKey> => {
  return await window.crypto.subtle.generateKey(
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt'],
  );
};
```

**Purpose**: Generate a unique 256-bit AES key for each Chronolock.

### Step 2: Metadata Encryption

```typescript
// Location: src/frontend/src/components/create/ReviewAndCreate.tsx

// 1. Prepare metadata object
const secureMetaData = {
  name, // Chronolock name
  description, // Description
  fileType, // MIME type
  mediaId, // Media file identifier
  mediaSize, // Size in bytes
  attributes, // Custom key-value pairs
};

// 2. Generate random IV (Initialization Vector)
const iv = window.crypto.getRandomValues(new Uint8Array(12));

// 3. Encrypt metadata with AES-GCM
const encodedMetaData = new TextEncoder().encode(
  JSON.stringify(secureMetaData),
);

const encryptedBuffer = await window.crypto.subtle.encrypt(
  { name: 'AES-GCM', iv },
  cryptoKey,
  encodedMetaData,
);

// 4. Concatenate IV + encrypted data
const concatenatedArray = new Uint8Array(
  iv.length + encryptedBuffer.byteLength,
);
concatenatedArray.set(iv, 0);
concatenatedArray.set(new Uint8Array(encryptedBuffer), iv.length);

// 5. Encode to Base64 for storage
const encryptedBase64 = btoa(
  String.fromCharCode(...new Uint8Array(concatenatedArray.buffer)),
);
```

**Data Structure**:

```
[12 bytes IV][Variable length encrypted data]
```

### Step 3: Media File Encryption

```typescript
// Location: src/frontend/src/components/create/UploadFile.tsx

// 1. Read file as ArrayBuffer
const arrayBuffer = await files[0].file.arrayBuffer();

// 2. Generate new IV for media
const iv = window.crypto.getRandomValues(new Uint8Array(12));

// 3. Encrypt media with same AES key
const encryptedBuffer = await window.crypto.subtle.encrypt(
  { name: 'AES-GCM', iv },
  cryptoKey,
  arrayBuffer,
);

// 4. Concatenate IV + encrypted media
const concatenatedArray = new Uint8Array(
  iv.length + encryptedBuffer.byteLength,
);
concatenatedArray.set(iv, 0);
concatenatedArray.set(new Uint8Array(encryptedBuffer), iv.length);

// 5. Upload to canister in chunks (1.95 MB per chunk)
const result = await upload(concatenatedArray.buffer);
```

### Step 4: AES Key Encryption with IBE

```typescript
// Location: src/frontend/src/components/create/ReviewAndCreate.tsx

// 1. Export raw AES key
const rawKey = await window.crypto.subtle.exportKey('raw', cryptoKey);
const rawKeyUint8 = new Uint8Array(rawKey);

// 2. Get VetKD public key from canister
const vetkdPublicKeyObject = await getVetkdPublicKey();
const vetkdPublicKey = DerivedPublicKey.deserialize(
  new Uint8Array(vetkdPublicKeyBuffer),
);

// 3a. For specific recipients (private chronolock)
recipients.map((recipient) => {
  const encryptedKey = IbeCiphertext.encrypt(
    vetkdPublicKey,
    IbeIdentity.fromString(`${recipient}:${lockTime}`), // Identity format
    rawKeyUint8,
    IbeSeed.random(),
  );

  const encryptedKeyBase64 = btoa(
    String.fromCharCode(...new Uint8Array(encryptedKey.serialize())),
  );

  userKeys.push({
    user: recipient,
    key: encryptedKeyBase64,
  });
});

// 3b. For public chronolock (no specific recipients)
const encryptedKey = IbeCiphertext.encrypt(
  vetkdPublicKey,
  IbeIdentity.fromString(lockTime.toString()), // Time only
  rawKeyUint8,
  IbeSeed.random(),
);

userKeys.push({
  user: 'public',
  key: encryptedKeyBase64,
});
```

**IBE Identity Formats**:

- **Public Chronolock**: `"1734567890"` (just the unlock time as decimal)
- **Private Chronolock**: `"user_principal_id:1734567890"` (user + unlock time)

### Step 5: Storage Structure

The final metadata stored in the canister:

```json
{
  "title": "Displayed before unlock",
  "owner": "principal_id",
  "lockTime": 1734567890,
  "createdAt": 1734000000,
  "encryptedMetaData": "base64_encrypted_data",
  "userKeys": [
    {
      "user": "recipient_principal_or_public",
      "key": "base64_ibe_encrypted_aes_key"
    }
  ]
}
```

---

## Decryption Flow

### Step 1: Verify Access and Time Lock

```rust
// Location: src/backend/chronolock_canister/src/lib.rs

#[update]
async fn get_time_decryption_key(
    unlock_time_hex: String,
    transport_public_key: Vec<u8>,
) -> Result<VetKDDeriveKeyReply, ChronoError> {
    // 1. Validate time lock
    let unlock_time = u64::from_str_radix(&unlock_time_hex, 16)?;
    let unlock_time_ns = unlock_time * 1_000_000_000;
    let current_time_ns = time();

    if current_time_ns < unlock_time_ns {
        return Err(ChronoError::TimeLocked);
    }

    // 2. Derive VetKD key using time as input
    let input = unlock_time.to_string().into_bytes();
    let context = b"chronolock-encryption".to_vec();

    call_vetkd_derive_key(input, context, transport_public_key).await
}

#[update]
async fn get_user_time_decryption_key(
    unlock_time_hex: String,
    user_id: String,
    transport_public_key: Vec<u8>,
) -> Result<VetKDDeriveKeyReply, ChronoError> {
    // 1. Authenticate caller
    let authenticated_caller = validate_caller_authentication()?;
    let authorized_principal = Principal::from_text(&user_id)?;

    if authenticated_caller != authorized_principal {
        return Err(ChronoError::Unauthorized);
    }

    // 2. Validate time lock
    let unlock_time = u64::from_str_radix(&unlock_time_hex, 16)?;
    let unlock_time_ns = unlock_time * 1_000_000_000;
    let current_time_ns = time();

    if current_time_ns < unlock_time_ns {
        return Err(ChronoError::TimeLocked);
    }

    // 3. Derive VetKD key using user+time as input
    let combined_id = format!("{}:{}", user_id, unlock_time);
    let input = combined_id.into_bytes();
    let context = b"chronolock-encryption".to_vec();

    call_vetkd_derive_key(input, context, transport_public_key).await
}
```

### Step 2: VetKD Key Derivation

```rust
// Location: src/backend/chronolock_canister/src/lib.rs

async fn call_vetkd_derive_key(
    input: Vec<u8>,
    context: Vec<u8>,
    transport_public_key: Vec<u8>,
) -> Result<VetKDDeriveKeyReply, ChronoError> {
    let args = VetKDDeriveKeyArgs {
        key_id: VetKDKeyId {
            name: get_vetkd_key_name(), // "dfx_test_key" or "test_key_1"
            curve: VetKDCurve::Bls12_381_G2,
        },
        input,           // Derivation input (time or user:time)
        context,         // "chronolock-encryption"
        transport_public_key, // Ephemeral public key from client
    };

    // Call IC management canister with cycles
    let cycles = 30_000_000_000u64; // 30 billion cycles
    let (result,): (VetKDDeriveKeyReply,) =
        call_with_payment(
            Principal::management_canister(),
            "vetkd_derive_key",
            (args,),
            cycles
        ).await?;

    Ok(result)
}
```

### Step 3: Frontend Decryption Process

```typescript
// Location: src/frontend/src/components/collection/DecryptModal.tsx

const decrypt = async () => {
  // 1. Generate transport key pair
  const transportSecretKey = TransportSecretKey.generate();
  const transportPublicKey = Array.from(transportSecretKey.publicKey());

  // 2. Get VetKD public key
  const vetkdPublicKeyResult = await getVetkdPublicKey();
  const derivedPublicKey = DerivedPublicKey.deserialize(
    new Uint8Array(vetkdPublicKeyResult.Ok.public_key),
  );

  // 3. Request decryption key from canister
  let decryptionKeyResult;
  if (isPublic) {
    // Public chronolock
    const unlockTimeHex = metadata.lockTime.toString(16).padStart(16, '0');
    decryptionKeyResult = await getTimeDecryptionKey(
      unlockTimeHex,
      transportPublicKey,
    );
  } else {
    // Private chronolock
    const unlockTimeHex = metadata.lockTime.toString(16).padStart(16, '0');
    decryptionKeyResult = await getUserTimeDecryptionKey(
      unlockTimeHex,
      userIdentity,
      transportPublicKey,
    );
  }

  // 4. Decrypt VetKey with transport secret key
  const encryptedVetKeyBytes = new Uint8Array(
    decryptionKeyResult.Ok.encrypted_key,
  );
  const encryptedVetKey = new EncryptedVetKey(encryptedVetKeyBytes);

  // Prepare derivation input matching backend
  let derivationInput;
  if (isPublic) {
    derivationInput = new TextEncoder().encode(metadata.lockTime.toString());
  } else {
    derivationInput = new TextEncoder().encode(
      `${userIdentity}:${metadata.lockTime}`,
    );
  }

  // Decrypt and verify VetKey
  const vetKey = encryptedVetKey.decryptAndVerify(
    transportSecretKey,
    derivedPublicKey,
    derivationInput,
  );

  // 5. Decrypt AES key using IBE and VetKey
  const encryptedUserKeyBytes = atob(userKey);
  const encryptedUserKeyUint8 = new Uint8Array(
    Array.from(encryptedUserKeyBytes).map((char) => char.charCodeAt(0)),
  );
  const ibeCiphertext = IbeCiphertext.deserialize(encryptedUserKeyUint8);

  const aesKeyBytes = ibeCiphertext.decrypt(vetKey);

  // 6. Import AES key for Web Crypto API
  const cryptoKey = await window.crypto.subtle.importKey(
    'raw',
    new Uint8Array(aesKeyBytes),
    { name: 'AES-GCM' },
    false,
    ['decrypt'],
  );

  // 7. Decrypt metadata
  const encryptedMetadataBytes = atob(metadata.encryptedMetaData);
  const encryptedBuffer = Uint8Array.from(encryptedMetadataBytes, (c) =>
    c.charCodeAt(0),
  );

  // Extract IV (first 12 bytes) and ciphertext
  const iv = encryptedBuffer.slice(0, 12);
  const ciphertext = encryptedBuffer.slice(12);

  const decryptedBuffer = await window.crypto.subtle.decrypt(
    { name: 'AES-GCM', iv },
    cryptoKey,
    ciphertext,
  );

  // Parse decrypted JSON
  const decryptedText = new TextDecoder().decode(decryptedBuffer);
  const decryptedJson = JSON.parse(decryptedText);

  // 8. Decrypt media file if present
  if (decryptedJson.mediaId && decryptedJson.mediaSize) {
    const encryptedMediaData = await getMediaChunked(
      decryptedJson.mediaId,
      estimatedEncryptedSize,
    );

    // Extract IV and encrypted content
    const mediaIv = encryptedMediaData.slice(0, 12);
    const encryptedMediaContent = encryptedMediaData.slice(12);

    // Decrypt media
    const decryptedMediaBuffer = await window.crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: mediaIv },
      cryptoKey,
      encryptedMediaContent,
    );

    // Create blob URL for display
    const blob = new Blob([decryptedMediaBuffer], {
      type: decryptedJson.fileType,
    });
    const mediaUrl = URL.createObjectURL(blob);

    setDecryptedData({
      ...decryptedJson,
      mediaUrl,
    });
  }
};
```

---

## Cryptographic Algorithms

### 1. AES-GCM (Advanced Encryption Standard - Galois/Counter Mode)

**Purpose**: Symmetric encryption of content (metadata and media files)

**Specifications**:

- Key size: 256 bits
- IV size: 12 bytes (96 bits)
- Authentication tag: 16 bytes (128 bits) - automatically included by AES-GCM
- Mode: Authenticated encryption with associated data (AEAD)

**Why AES-GCM?**

- Fast and efficient for large files
- Built-in authentication (integrity checking)
- Native browser support via Web Crypto API
- Industry standard for data encryption

**Security Properties**:

- Confidentiality: Data cannot be read without the key
- Integrity: Tampering is detected
- Authenticity: Ensures data comes from legitimate source

### 2. IBE (Identity-Based Encryption)

**Purpose**: Encrypt the AES key using user identity and unlock time

**Specifications**:

- Based on BLS12-381 elliptic curve pairing
- Identity format determines who can decrypt
- No need for traditional PKI certificates

**Identity Structure**:

```
Public: "1734567890"                    (unlock time only)
Private: "principal_id:1734567890"      (user + unlock time)
```

**Why IBE?**

- Direct encryption to identity (no key exchange needed)
- Access control built into cryptographic identity
- Supports time-based access patterns
- Compatible with VetKD system

### 3. VetKD (Verifiable Encrypted Threshold Key Derivation)

**Purpose**: Time-locked and distributed key derivation

**Specifications**:

- Curve: BLS12-381 G2
- Key name: "dfx_test_key" (local) or "test_key_1" (mainnet)
- Context: "chronolock-encryption"
- Threshold signature scheme

**How VetKD Works**:

1. **Key Derivation Input**: Time or User+Time identity
2. **Threshold Consensus**: Multiple IC subnet nodes must agree
3. **Time Check**: Only derive key after unlock time
4. **Transport Encryption**: Key encrypted for specific recipient
5. **Verification**: Client can verify key authenticity

**Security Properties**:

- **Threshold security**: No single node can derive key alone
- **Time-lock**: Key mathematically unavailable before unlock time
- **Verifiable**: Recipients can verify key authenticity
- **Non-interactive**: No coordination needed between users

---

## Security Model

### Threat Model

**Protected Against**:

1. ✅ **Unauthorized Access Before Time**: Cannot decrypt before unlock time
2. ✅ **Data Tampering**: AES-GCM detects modifications
3. ✅ **Key Theft**: AES keys never stored, only encrypted form
4. ✅ **Impersonation**: IBE identity verification
5. ✅ **Replay Attacks**: Random IVs for each encryption
6. ✅ **Centralized Failure**: Distributed VetKD across IC subnet

**Not Protected Against** (by design):

1. ❌ **Post-Unlock Access**: Anyone with proper identity can decrypt after time
2. ❌ **Content Redistribution**: Users can copy decrypted content
3. ❌ **Client-Side Malware**: Compromised browser can capture decrypted data
4. ❌ **Quantum Computers**: Classical cryptography vulnerable to future quantum attacks

### Trust Assumptions

**Trusted Components**:

- Internet Computer consensus
- Web browser's Web Crypto API implementation
- VetKD threshold subnet nodes
- User's device security

**Zero Trust**:

- No trusted third party needed
- No central key authority
- Canister code is open and verifiable

---

## Key Management

### Key Lifecycle

```
1. Generation
   └─> AES-256 key generated in browser
       - Cryptographically secure random
       - Never leaves client unencrypted

2. Encryption Phase
   └─> AES key encrypted with IBE
       - Multiple recipients possible
       - Identity-based access control

3. Storage
   └─> Only encrypted keys stored
       - On IC canister (decentralized)
       - No plaintext key anywhere

4. Derivation Request
   └─> User requests VetKD derivation
       - After unlock time only
       - Authenticated request

5. Decryption
   └─> VetKey decrypts IBE ciphertext
       - Recovers original AES key
       - Used to decrypt content

6. Ephemeral Use
   └─> AES key exists in memory only
       - Used for decryption
       - Discarded after use
```

### Key Rotation

**Not Implemented** (by design):

- Each Chronolock has a unique, single-use AES key
- Keys are never rotated or updated
- Immutable once created (blockchain property)

### Key Recovery

**Not Possible**:

- No key escrow or backup system
- If unlock time hasn't passed: **mathematically impossible to decrypt**
- If user loses access: **cannot recover without proper identity**
- This is a feature, not a bug (trustless system)

---

## Implementation Details

### Frontend Stack

**Libraries**:

- `@dfinity/vetkeys`: IBE and VetKD operations

  - `DerivedPublicKey`: VetKD public key handling
  - `IbeCiphertext`: IBE encryption/decryption
  - `IbeIdentity`: Identity string formatting
  - `IbeSeed`: Random seed generation
  - `TransportSecretKey`: Ephemeral key for secure transport
  - `EncryptedVetKey`: VetKey decryption

- Web Crypto API: Native browser cryptography
  - `crypto.subtle.generateKey()`: AES key generation
  - `crypto.subtle.encrypt()`: AES-GCM encryption
  - `crypto.subtle.decrypt()`: AES-GCM decryption
  - `crypto.getRandomValues()`: IV generation

**Key Files**:

- `src/frontend/src/pages/Create.tsx`: Chronolock creation flow
- `src/frontend/src/components/create/ReviewAndCreate.tsx`: Encryption logic
- `src/frontend/src/components/create/UploadFile.tsx`: Media encryption
- `src/frontend/src/components/collection/DecryptModal.tsx`: Decryption logic
- `src/frontend/src/hooks/useChronolock.ts`: Canister interaction

### Backend Stack (Rust)

**Libraries**:

- `ic-cdk`: Internet Computer development kit
- `candid`: Interface definition and serialization
- `serde`: JSON serialization/deserialization

**Key Files**:

- `src/backend/chronolock_canister/src/lib.rs`: Main canister logic
- Functions:
  - `ibe_encryption_key()`: Get VetKD public key
  - `get_time_decryption_key()`: Public chronolock decryption
  - `get_user_time_decryption_key()`: Private chronolock decryption
  - `call_vetkd_derive_key()`: VetKD interaction
  - `create_chronolock()`: Store encrypted chronolock
  - `get_media_chunk()`: Retrieve encrypted media

### Data Storage

**Stable Structures** (IC):

```rust
CHRONOLOCKS: StableBTreeMap<String, Chronolock, Memory>
MEDIA_FILES: StableBTreeMap<String, Vec<u8>, Memory>
OWNER_TO_TOKENS: StableBTreeMap<Principal, TokenList, Memory>
```

**Chronolock Structure**:

```rust
struct Chronolock {
    id: String,
    owner: Principal,
    metadata: String, // hex-encoded MetaData
}

struct MetaData {
    title: Option<String>,
    owner: String,
    unlock_time: u64,
    created_at: Option<u64>,
    user_keys: Option<serde_json::Value>, // Array of {user, key}
    encrypted_metadata: String,            // Base64 encrypted data
}
```

---

## Code Examples

### Creating a Chronolock

```typescript
// 1. Generate AES key
const cryptoKey = await window.crypto.subtle.generateKey(
  { name: 'AES-GCM', length: 256 },
  true,
  ['encrypt', 'decrypt'],
);

// 2. Encrypt metadata
const iv = window.crypto.getRandomValues(new Uint8Array(12));
const metadata = { name: 'My Secret', description: 'Secret message' };
const encryptedMetadata = await window.crypto.subtle.encrypt(
  { name: 'AES-GCM', iv },
  cryptoKey,
  new TextEncoder().encode(JSON.stringify(metadata)),
);

// 3. Get VetKD public key
const vetkdPubKey = await chronolockCanister.ibe_encryption_key();

// 4. Encrypt AES key with IBE
const rawKey = await window.crypto.subtle.exportKey('raw', cryptoKey);
const encryptedAesKey = IbeCiphertext.encrypt(
  DerivedPublicKey.deserialize(vetkdPubKey.Ok.public_key),
  IbeIdentity.fromString('recipient_id:1734567890'),
  new Uint8Array(rawKey),
  IbeSeed.random(),
);

// 5. Store chronolock
await chronolockCanister.create_chronolock(
  btoa(
    JSON.stringify({
      title: 'My Chronolock',
      owner: principalId,
      lockTime: 1734567890,
      encryptedMetaData: btoa(
        String.fromCharCode(...new Uint8Array(encryptedMetadata)),
      ),
      userKeys: [
        {
          user: 'recipient_id',
          key: btoa(String.fromCharCode(...encryptedAesKey.serialize())),
        },
      ],
    }),
  ),
);
```

### Decrypting a Chronolock

```typescript
// 1. Generate transport key pair
const transportSecretKey = TransportSecretKey.generate();

// 2. Request VetKD decryption key
const decryptionKey = await chronolockCanister.get_user_time_decryption_key(
  lockTime.toString(16).padStart(16, '0'),
  userId,
  Array.from(transportSecretKey.publicKey()),
);

// 3. Decrypt VetKey
const vetKey = new EncryptedVetKey(
  new Uint8Array(decryptionKey.Ok.encrypted_key),
).decryptAndVerify(
  transportSecretKey,
  derivedPublicKey,
  new TextEncoder().encode(`${userId}:${lockTime}`),
);

// 4. Decrypt AES key
const aesKeyBytes = IbeCiphertext.deserialize(
  new Uint8Array(
    atob(userKey)
      .split('')
      .map((c) => c.charCodeAt(0)),
  ),
).decrypt(vetKey);

// 5. Import and decrypt content
const cryptoKey = await window.crypto.subtle.importKey(
  'raw',
  new Uint8Array(aesKeyBytes),
  { name: 'AES-GCM' },
  false,
  ['decrypt'],
);

const encryptedData = Uint8Array.from(atob(encryptedMetaData), (c) =>
  c.charCodeAt(0),
);

const decryptedData = await window.crypto.subtle.decrypt(
  { name: 'AES-GCM', iv: encryptedData.slice(0, 12) },
  cryptoKey,
  encryptedData.slice(12),
);

const metadata = JSON.parse(new TextDecoder().decode(decryptedData));
```

---

## Security Considerations

### Best Practices Implemented

1. **Random IV Generation**: New IV for every encryption operation
2. **Authenticated Encryption**: AES-GCM provides both confidentiality and integrity
3. **Key Separation**: Different encrypted keys for different recipients
4. **No Key Reuse**: Unique AES key per Chronolock
5. **Secure Random**: Using cryptographically secure random number generators
6. **Time Validation**: Backend enforces unlock time strictly
7. **Authentication**: User identity verified before private key derivation

### Potential Vulnerabilities & Mitigations

| Vulnerability              | Mitigation                                                  |
| -------------------------- | ----------------------------------------------------------- |
| **Timing attacks**         | VetKD runs on IC, isolated from attacker timing analysis    |
| **Side-channel attacks**   | Web Crypto API uses constant-time operations                |
| **Key exposure in memory** | Keys exist transiently, not persisted                       |
| **Man-in-the-middle**      | IC's HTTPS certification and consensus                      |
| **Replay attacks**         | Random IVs and unique transport keys                        |
| **Quantum computing**      | Future: Migrate to post-quantum cryptography when available |

### Operational Security

**For Users**:

- 🔐 Keep Internet Identity secure (seed phrase backup)
- 🖥️ Use trusted devices and browsers
- 🌐 Verify canister IDs before interacting
- ⏰ Understand that unlock time is immutable
- 💾 Decrypted content should be handled securely

**For Developers**:

- 🔍 Audit smart contract code regularly
- 🔄 Keep dependencies updated (@dfinity/vetkeys)
- 📊 Monitor canister cycles for VetKD operations
- 🧪 Comprehensive testing of encryption/decryption flows
- 📝 Document security assumptions clearly

### Limitations

1. **Post-Decryption Protection**: Once decrypted, content is no longer protected
2. **Browser Security**: Relies on browser's Web Crypto API security
3. **IC Liveness**: Requires IC to be operational for decryption
4. **Irrevocable Access**: Cannot revoke access after unlock time
5. **No Forward Secrecy**: Same key structure for entire chronolock lifetime

---

## Future Enhancements

### Potential Improvements

1. **Post-Quantum Cryptography**

   - Migrate to quantum-resistant algorithms
   - Hybrid classical-quantum schemes

2. **Zero-Knowledge Proofs**

   - Prove access rights without revealing identity
   - Privacy-preserving access patterns

3. **Multi-Factor Time Locks**

   - Require multiple conditions (time + event)
   - Conditional decryption logic

4. **Progressive Disclosure**

   - Decrypt content in stages
   - Partial reveals over time

5. **Access Revocation**

   - Mechanism to revoke access before unlock (if supported by use case)
   - Re-encryption capabilities

6. **Key Splitting**
   - Shamir's Secret Sharing for enhanced security
   - M-of-N decryption schemes

---

## Conclusion

Chronolock's encryption and decryption system represents a sophisticated implementation of modern cryptographic techniques combined with blockchain technology. The multi-layered approach provides:

✅ **Strong Security**: AES-GCM + IBE + VetKD  
✅ **Time-Lock Guarantees**: Cryptographically enforced unlock times  
✅ **Decentralization**: No trusted third parties  
✅ **User Control**: Identity-based access management  
✅ **Transparency**: Open-source and verifiable

This architecture makes Chronolock suitable for various use cases requiring trustless, time-locked content storage, from personal time capsules to legal documents and scheduled releases.

---

## References

### Technical Documentation

- [Internet Computer VetKD Documentation](https://internetcomputer.org/docs/current/references/vetkeys-overview)
- [Web Crypto API Specification](https://www.w3.org/TR/WebCryptoAPI/)
- [AES-GCM NIST Documentation](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf)
- [Identity-Based Encryption (Boneh-Franklin)](https://crypto.stanford.edu/~dabo/papers/bfibe.pdf)
- [BLS12-381 Elliptic Curve](https://hackmd.io/@benjaminion/bls12-381)

### Chronolock Codebase

- Backend: `/src/backend/chronolock_canister/src/lib.rs`
- Frontend Encryption: `/src/frontend/src/components/create/ReviewAndCreate.tsx`
- Frontend Decryption: `/src/frontend/src/components/collection/DecryptModal.tsx`
- Hooks: `/src/frontend/src/hooks/useChronolock.ts`

### Dependencies

- `@dfinity/agent`: ^2.4.1
- `@dfinity/auth-client`: ^2.4.1
- `@dfinity/vetkeys`: ^0.2.0
- `ic-cdk`: Latest (Rust)
- `candid`: Latest (Rust)

---

**Document Version**: 1.0  
**Last Updated**: October 5, 2025  
**Maintained By**: Chronolock Development Team
