# RFC-045: Real Transport Adapter (BLE / Wi-Fi Direct / QR)

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M11 (sync sprint) |
| **Priority** | P1 |
| **Review finding** | Functional §8; Non-functional §5 |

## Problem

`taktakk-sync` has a working manifest exchange protocol, chunk model, and
inventory diff. The only real transport is `LocalFsTransport`, which uses
the filesystem as a "wire" — suitable for CI but not for field use.

Missing:
- BLE (Bluetooth Low Energy) GATT-based discovery and object transfer
- Wi-Fi Direct (Android P2P API)
- QR code bootstrap (bootstrap inventory exchange over 1D/2D barcode)
- Noise protocol handshake (encrypted channel for BLE/WiFi)
- Android permission integration for Nearby Devices, Camera
- Radio session timeout and automatic shutdown after transfer

## Design

### Transport trait (current — keep)

```rust
pub trait SyncTransportAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn exchange_inventory(&self, local_json: &str) -> CoreResult<String>;
    fn fetch_object(&self, object_hash: &str) -> CoreResult<Vec<u8>>;
    fn push_object(&self, object_hash: &str, data: &[u8]) -> CoreResult<()>;
}
```

All new transports implement this trait. The sync protocol layer is
transport-agnostic.

### Phase 1 — QR bootstrap (minimum viable real transport)

QR is the simplest to implement on Android without a network stack:

1. Device A renders a QR code encoding a JSON payload:
   ```json
   {"inv": "<inventory-json>", "addr": "<local-ip:port>"}
   ```
2. Device B scans the QR, extracts `inv` for the diff, and connects
   to `addr` over local Wi-Fi to transfer objects via a minimal HTTP/1.1
   server (no TLS required on a local link, but use a nonce challenge).

Implementation:
- `QrTransport` on Android uses `CameraX` for scanning.
- Object transfer uses `HttpLocalTransport` (loopback / LAN).
- `HttpLocalTransport` implements `SyncTransportAdapter`.

### Phase 2 — BLE GATT

- Discovery: taktakk devices advertise a GATT service UUID derived from a
  truncated HMAC of the current day's date (rolling identifier — not
  linkable across days).
- Inventory exchange: GATT characteristic write/notify.
- Large objects: transferred over L2CAP CoC (BLE 5.0) or chunked GATT
  writes with the chunk model (RFC-023).
- Android permissions: `BLUETOOTH_SCAN`, `BLUETOOTH_CONNECT` requested
  when user opens Share menu (RFC-024).
- Session timeout: auto-disconnect after 10 minutes of inactivity.

### Phase 3 — Wi-Fi Direct

- Android `WifiP2pManager` API.
- Device A acts as group owner; Device B connects.
- Objects transferred over TCP socket with the existing chunk protocol.
- Timeout: WifiP2p group dissolved after transfer completes or 15 min.

### Platform adapter pattern

```
taktakk-sync/src/transport/
  mod.rs          SyncTransportAdapter trait
  local.rs        LocalFsTransport (CI / demo)
  http_local.rs   HttpLocalTransport (LAN / QR)
  ble.rs          BleTransport trait (platform-specific impl injected)
  qr.rs           QrBootstrap (encode/decode JSON + launch HttpLocal)
```

Android-specific BLE/WifiP2p code lives in `apps/taktakk-android-wrapper`
and is injected via a Rust `extern "C"` FFI or a Kotlin bridge.

## Acceptance criteria

1. `HttpLocalTransport` performs a complete inventory exchange and object
   transfer between two processes on the same machine.
2. `cargo test -p taktakk-sync -- http_local_transport` passes.
3. `QrBootstrap::encode(inventory_json, addr)` produces a valid QR payload
   decodable by `QrBootstrap::decode()`.
4. After the QR bootstrap, `HttpLocalTransport::exchange_inventory()` and
   `fetch_object()` complete the sync.
5. `LocalFsTransport` documentation is updated to label it
   "CI/demo only — not for field use".
6. At least one of QR+HTTP, BLE, or Wi-Fi Direct completes an end-to-end
   two-device package transfer test on Android hardware.
