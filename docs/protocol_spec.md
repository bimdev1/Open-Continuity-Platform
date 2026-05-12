# Open Continuity API (OCA) - Protocol Specification V1

This document defines the binary wire format for the Open Continuity API (OCA) Platform.

## 1. Transport Layer
- **Protocol:** TCP
- **Port:** 5005 (assigned via mDNS)
- **mDNS Service Type:** `_oca._tcp.local`

## 2. Binary Wire Format
All numeric values are Little Endian (LE).

| Offset | Field | Size (Bytes) | Description |
| :--- | :--- | :--- | :--- |
| 0 | Version | 1 | Protocol version (0x01) |
| 1 | Payload Type | 1 | 0x01: Handshake, 0x02: Clipboard Text, 0x03: Ack |
| 2 | Payload Length | 4 | Length of the encrypted data section (N) |
| 6 | AEAD Nonce | 12 | Nonce used for ChaCha20-Poly1305 |
| 18 | Encrypted Data | N | The AEAD-encrypted payload |
| 18 + N | Auth Tag | 16 | The 128-bit authentication tag from AEAD |

## 3. Handshake Sequence
1. **Peer A -> Peer B:** `HandshakeInit` (Includes Ed25519 Public Key)
2. **Peer B -> Peer A:** `HandshakeAck` (Includes Ed25519 Public Key)
3. **Session Establishment:** Peers derive a shared secret (ideally via X25519, though Ed25519 is used for identity in this MVP).

## 4. Clipboard Payload
The decrypted payload for `Payload Type: 0x02` is a JSON object:

```json
{
  "text": "String contents of clipboard",
  "timestamp": 1625091234000,
  "source_device": "Android-Pixel-6"
}
```

## 5. Security Notes
- **Identity:** Public keys should be pinned or verified via a first-time-use (TOFU) model.
- **Perfect Forward Secrecy:** Future versions should implement Ephemeral DH (X25519) to ensure past captures cannot be decrypted if keys are stolen.
