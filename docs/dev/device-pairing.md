# Device Pairing Architecture

**Status:** Implemented
**Phase:** 08-production-readiness
**Related:** `docs/concepts/device-security.md`, `docs/guides/device-pairing-setup.md`

## Overview

Device pairing provides mTLS (mutual TLS) authentication for AOF daemon connections. Instead of shared secrets or API keys, devices present client certificates issued by a private Certificate Authority. The daemon validates certificates and checks device approval status before accepting connections.

## Architecture Components

### 1. Private Certificate Authority (CA)

**Location:** `crates/aof-runtime/src/device/ca.rs`

The Private CA is responsible for issuing and managing certificates:

- **Root Certificate:** Self-signed, 10-year validity, stored at `~/.local/share/aof/ca/ca.crt`
- **Private Key:** 0600 permissions, stored at `~/.local/share/aof/ca/ca.key`
- **Client Certificates:** Signed by CA, 1-year validity (configurable), embedded device metadata

**Certificate Format:**
```
Root CA:
  Subject: CN=AOF Private CA, O=AOF
  Validity: 10 years
  Key Usage: DigitalSignature, KeyCertSign, CrlSign

Client Certificate:
  Subject: CN=<device-name>, O=AOF Device
  Validity: 1 year (default)
  Key Usage: DigitalSignature, KeyEncipherment
  Extended Key Usage: ClientAuth
  SAN: device-{uuid}, type-{DeviceType}
```

**Device Metadata in SANs:**
Device ID and type are embedded as DNS Subject Alternative Names:
- `device-{uuid}` - Unique device identifier
- `type-{DeviceType}` - Device type (cli, web_ui, slack_bot, etc.)

This allows the server to extract device information during TLS handshake without additional lookups.

### 2. Device Registry

**Location:** `crates/aof-runtime/src/device/registry.rs`

Persistent JSON storage for device approval workflow:

**Storage Format (`~/.local/share/aof/devices/registry.json`):**
```json
{
  "dev-123": {
    "device_id": "dev-123",
    "name": "mission-control-laptop",
    "device_type": "cli",
    "status": "approved",
    "certificate_fingerprint": "aa:bb:cc:...",
    "registered_at": "2026-02-14T10:00:00Z",
    "approved_at": "2026-02-14T10:05:00Z",
    "approved_by": "admin",
    "last_seen": "2026-02-14T14:30:00Z",
    "last_ip": "192.168.1.10",
    "metadata": {}
  }
}
```

**Approval Workflow:**
1. **Pending:** Device registered, certificate issued, awaiting approval
2. **Approved:** Operator approved, device can connect
3. **Revoked:** Previously approved, now blocked
4. **Expired:** Certificate expired

**Concurrency Model:**
- parking_lot::RwLock for thread-safe access
- Automatic persistence on every mutation (register, approve, revoke)
- Atomic write pattern (temp file + rename) for crash safety

### 3. mTLS Server Configuration

**Location:** `crates/aof-runtime/src/device/mtls.rs`

Integrates rustls for mutual TLS authentication:

**TLS Stack:**
```
Client Certificate → WebPkiClientVerifier → RootCertStore (CA cert)
                                          ↓
                              Device ID Extraction (from SAN)
                                          ↓
                              DeviceRegistry Approval Check
                                          ↓
                          Accept (Approved) | Reject (403 Unauthorized)
```

**Server Configuration:**
```rust
let config = ServerConfig::builder()
    .with_client_cert_verifier(client_verifier)  // Require client cert
    .with_single_cert(server_certs, server_key)  // Server identity
    .build()?;

config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
```

**Device Approval Check Flow:**
1. TLS handshake validates client certificate signature
2. Extract device_id from certificate SAN
3. Query DeviceRegistry for approval status
4. Reject if status != Approved
5. Record connection (last_seen, last_ip)

### 4. CLI Commands

**Location:** `crates/aofctl/src/commands/device.rs`

kubectl-style device management:

**Command Structure:**
```
aofctl init ca                           # Initialize CA
aofctl device register --name <name>     # Register device
aofctl device list                       # List all devices
aofctl device approve <device-id>        # Approve pending device
aofctl device revoke <device-id>         # Revoke approved device
aofctl device inspect <device-id>        # Show device details
```

**Certificate Storage:**
Certificates saved at `~/.local/share/aof/devices/{device-id}/`:
- `client.crt` - Client certificate
- `client.key` - Private key (0600 permissions)
- `ca.crt` - CA certificate (for validating server cert)

## Security Considerations

### Certificate Key Protection

**CA Private Key:**
- File permissions: 0600 (owner read/write only)
- Location: `~/.local/share/aof/ca/ca.key`
- Backup recommended (offline storage)

**Client Private Keys:**
- File permissions: 0600 per device directory
- Never transmitted over network
- Stored only on registered device

### Certificate Revocation

**Current Implementation:**
- Status-based revocation (Approved → Revoked in registry)
- No Certificate Revocation List (CRL) in v1
- Revoked devices rejected at approval check

**Future Enhancement:**
- OCSP responder for real-time revocation checks
- CRL distribution for offline validation

### Attack Vectors & Mitigations

**1. CA Key Compromise:**
- **Risk:** Attacker can issue valid certificates
- **Mitigation:** Strict file permissions (0600), offline backup, key rotation
- **Detection:** Monitor registry for unexpected registrations

**2. Stolen Client Certificate:**
- **Risk:** Attacker uses stolen cert to impersonate device
- **Mitigation:** Revoke compromised device via `aofctl device revoke`
- **Detection:** Monitor connection IPs, unusual activity patterns

**3. Self-Signed Certificate Attack:**
- **Risk:** Attacker presents self-signed cert
- **Mitigation:** Client verifier validates CA signature
- **Detection:** TLS handshake fails, logged at daemon

**4. Expired Certificate:**
- **Risk:** Device continues operating with expired cert
- **Mitigation:** Certificate expiry enforced at TLS layer
- **Detection:** Auto-marked as Expired in registry

## Implementation Details

### Device ID Extraction

Using x509-parser to extract device_id from certificate SAN:

```rust
use x509_parser::prelude::FromDer;

let cert = x509_parser::parse_x509_certificate(cert_der)?.1;
let san_ext = cert.tbs_certificate.get_extension_unique(
    &x509_parser::oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME
)?;

let san = x509_parser::extensions::SubjectAlternativeName::from_der(san_ext.value)?.1;

for name in &san.general_names {
    if let GeneralName::DNSName(dns_name) = name {
        if let Some(device_id) = dns_name.strip_prefix("device-") {
            return Ok(device_id.to_string());
        }
    }
}
```

### Certificate Fingerprint Calculation

SHA256 hash of certificate PEM content:

```rust
use sha2::{Sha256, Digest};

let mut hasher = Sha256::new();
hasher.update(cert_pem.as_bytes());
let result = hasher.finalize();

// Format: aa:bb:cc:...
let fingerprint = result.iter()
    .map(|byte| format!("{:02x}", byte))
    .collect::<Vec<_>>()
    .join(":");
```

### Atomic File Writes

Registry persistence uses atomic write pattern:

```rust
// Write to temp file
let temp_path = registry_path.with_extension("tmp");
tokio::fs::write(&temp_path, json).await?;

// Atomic rename
tokio::fs::rename(temp_path, registry_path).await?;
```

Ensures no partial writes on crash or power loss.

## Performance Considerations

### TLS Handshake Overhead

**Benchmarks (approximate):**
- Initial handshake: 50-100ms (includes cert validation)
- Session resumption: 5-10ms (reuses TLS session)
- Certificate parsing: <5ms per connection

**Optimization:**
- Enable TLS session tickets for resumption
- Cache device approval status (future: in-memory LRU cache)
- Use HTTP/2 for connection pooling

### Registry Scalability

**Current Design:**
- Full registry loaded into memory
- Suitable for <1000 devices
- Single file I/O on mutations

**Future Scaling (if needed):**
- Migrate to SQLite for >1000 devices
- Add index on certificate_fingerprint
- Batch registry writes (flush every 5s)

## Testing

### Unit Tests

**CA Tests (5 tests):**
- CA initialization creates valid root cert
- CA loading preserves certificate data
- Client cert issuance produces valid certs
- CA key has 0600 permissions
- Certificate contains device metadata in SAN

**Registry Tests (7 tests):**
- Device registration starts as Pending
- Approval transitions to Approved with timestamp
- Revocation transitions to Revoked
- is_approved check works correctly
- find_by_fingerprint lookup works
- Persistence survives save/load cycle
- Status filtering returns correct subsets

**mTLS Tests (3 tests):**
- MtlsConfig creation sets correct defaults
- build_tls_config successfully loads CA and server certs
- Device approval check integrates with registry

### Integration Tests (Task 6)

**E2E Scenarios:**
1. Full pairing workflow: init CA → register → approve → connect
2. Unapproved device rejection
3. Certificate expiry handling
4. Revoked device rejection

## Operational Procedures

### CA Initialization

```bash
# Initialize CA (once per deployment)
aofctl init ca

# Backup CA directory
tar czf aof-ca-backup.tar.gz ~/.local/share/aof/ca/
# Store backup offline
```

### Device Registration

```bash
# On server: Register device
aofctl device register --name laptop-001 --type cli

# Copy certificates to device
scp -r ~/.local/share/aof/devices/<device-id> user@device:/tmp/

# On server: Approve device
aofctl device approve <device-id>

# On device: Connect using mTLS
aofctl --cert /tmp/<device-id>/client.crt \
       --key /tmp/<device-id>/client.key \
       --ca-cert /tmp/<device-id>/ca.crt \
       status
```

### Certificate Rotation

**Manual Rotation (current):**
1. Generate new certificate: `aofctl device register`
2. Approve new device
3. Update client configuration
4. Revoke old device: `aofctl device revoke <old-device-id>`

**Automated Rotation (future):**
- Daemon monitors certificate expiry
- Auto-generates new cert 30 days before expiry
- Sends notification to device
- Grace period allows seamless transition

### Disaster Recovery

**CA Key Compromise:**
1. Revoke all devices: `for id in $(aofctl device list -o json | jq -r '.[].device_id'); do aofctl device revoke $id; done`
2. Re-initialize CA: `rm -rf ~/.local/share/aof/ca && aofctl init ca`
3. Re-register all devices with new certificates

**Registry Corruption:**
1. Restore from backup: `cp registry.json.backup ~/.local/share/aof/devices/registry.json`
2. Verify integrity: `aofctl device list`
3. Re-approve devices if needed

## References

- **RFC 5280:** X.509 Public Key Infrastructure Certificate
- **RFC 5246:** TLS 1.2 Protocol
- **RFC 8446:** TLS 1.3 Protocol
- **rustls:** Modern TLS library in Rust (https://github.com/rustls/rustls)
- **rcgen:** Certificate generation library (https://github.com/rustls/rcgen)

## Future Enhancements

1. **OCSP Responder:** Real-time certificate revocation checks
2. **Hardware Security Module (HSM):** Store CA key in HSM
3. **Certificate Transparency Logs:** Audit trail of issued certificates
4. **Automated Certificate Rotation:** 30-day expiry warning and auto-renewal
5. **Multi-CA Support:** Different CAs for different device types
6. **CRL Distribution:** Support offline revocation validation
