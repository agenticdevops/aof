# Device Security with mTLS

**Audience:** All Users
**Related:** `docs/guides/device-pairing-setup.md`, `docs/dev/device-pairing.md`

## Why Device Authentication Matters

In production environments, the AOF daemon accepts commands from multiple client types:
- CLI tools (`aofctl`)
- Web dashboards (Mission Control)
- Messaging bots (Slack, Discord)
- API clients (custom integrations)

Without authentication, any client with network access can send commands to agents operating on production infrastructure. This creates serious security risks:

- Unauthorized command execution
- Data exfiltration via agent queries
- Resource abuse (running expensive LLM calls)
- Lateral movement (compromised device spreads to other systems)

**Device pairing with mTLS solves this** by requiring cryptographic proof of identity and a human-in-the-loop approval workflow.

## What is mTLS?

**Mutual TLS (mTLS)** means both client and server prove their identity using certificates:

**Standard TLS (HTTPS):**
```
Client → "Are you really example.com?" → Server
       ← Server Certificate (signed by public CA) ←
       → "OK, I trust you" →
```

**Mutual TLS (mTLS):**
```
Client → "Are you really aof-daemon?" → Server
       ← Server Certificate (signed by private CA) ←
       → Client Certificate (signed by private CA) →
       ← "OK, I trust you too" ←
```

Both sides verify each other's identity before establishing a connection.

## Why mTLS Instead of API Keys?

| Approach | Security | Rotation | Revocation | Audit Trail |
|----------|----------|----------|------------|-------------|
| **API Keys** | Shared secret (can be stolen) | Manual | Manual | Limited |
| **OAuth/JWT** | Time-limited tokens | Automatic | Revoke tokens | Good |
| **mTLS** | Cryptographic proof | Certificate expiry | Instant | Excellent |

**mTLS advantages:**
- **No shared secrets:** Private key never leaves the device
- **Automatic expiry:** Certificates have built-in validity periods
- **Instant revocation:** Block compromised devices immediately
- **Tamper-proof audit:** Certificate fingerprints in logs

**Why not public CAs?**

As of May 2026, public Certificate Authorities (Let's Encrypt, DigiCert, etc.) no longer issue client authentication certificates. They focus solely on server authentication (HTTPS websites).

AOF uses a **private CA** you control:
- Issue certificates only for your devices
- No external dependencies
- Full control over revocation
- Works offline

## How Device Pairing Works

### Step 1: Initialize Private CA (Once)

The administrator creates a self-signed Certificate Authority:

```bash
aofctl init ca
```

**What happens:**
- Generates 10-year root certificate
- Stores CA private key at `~/.local/share/aof/ca/ca.key` (0600 permissions)
- Stores CA certificate at `~/.local/share/aof/ca/ca.crt`

**Security:** The CA private key is the "master key" - protect it like root access credentials.

### Step 2: Register a Device

When a new device needs access:

```bash
aofctl device register --name mission-control-laptop --type cli
```

**What happens:**
- Generates a unique device ID (UUID)
- Issues a client certificate signed by the CA (1-year validity)
- Saves certificate and private key to `~/.local/share/aof/devices/{device-id}/`
- Registers device in approval workflow (status: Pending)

**Device Types:**
- `cli` - Command-line aofctl client
- `web_ui` - Mission Control dashboard
- `slack_bot` - Slack integration
- `discord_bot` - Discord integration
- `api_client` - Generic API client
- Custom types supported

### Step 3: Approve the Device

The operator reviews and approves the device:

```bash
aofctl device list                    # See pending devices
aofctl device approve <device-id>     # Approve specific device
```

**What happens:**
- Device status changes from Pending → Approved
- Records approval timestamp and approver identity
- Device can now connect to the daemon

**Why manual approval?**

Human-in-the-loop prevents:
- Rogue devices auto-registering
- Compromised systems gaining access
- Insider threats (operator accountability)

### Step 4: Connect with mTLS

The approved device connects using its certificate:

```bash
aofctl --cert ~/.local/share/aof/devices/{device-id}/client.crt \
       --key ~/.local/share/aof/devices/{device-id}/client.key \
       --ca-cert ~/.local/share/aof/devices/{device-id}/ca.crt \
       status
```

**What happens during connection:**
1. TLS handshake: Client presents certificate
2. Server validates certificate signature (must be from CA)
3. Server extracts device ID from certificate
4. Server checks device approval status in registry
5. If Approved: Connection accepted
6. If Pending/Revoked: Connection rejected (403 Unauthorized)
7. Server records connection (last_seen timestamp, IP address)

## Security Properties

### 1. Trust but Verify

Even with a valid certificate, devices must be approved by an operator. This prevents:
- Stolen certificates (revoke and re-issue)
- Compromised devices (revoke immediately)
- Unauthorized devices (never approved)

### 2. Tamper-Proof Audit Trail

Every device action is tied to:
- **Device ID:** Who performed the action
- **Certificate Fingerprint:** Exact certificate used
- **IP Address:** Where the connection came from
- **Timestamp:** When the action occurred

This creates a forensic trail for incident investigation.

### 3. Defense in Depth

Multiple layers of security:
1. **TLS encryption:** All traffic encrypted in transit
2. **Certificate validation:** Only CA-signed certs accepted
3. **Approval workflow:** Human verification required
4. **Status checks:** Revoked devices blocked instantly
5. **Audit logging:** Full connection history

### 4. Least Privilege

Each device:
- Has unique credentials (no shared secrets)
- Can be revoked independently
- Has a limited validity period (1 year default)
- Is tracked individually in logs

## Common Security Scenarios

### Device Compromised

**Symptoms:**
- Unusual activity from device IP
- Commands executed outside normal hours
- Access from unexpected location

**Response:**
```bash
aofctl device revoke <device-id>  # Block immediately
aofctl device list                # Verify revocation
aofctl device register --name <device-name> --type cli  # Issue new cert
aofctl device approve <new-device-id>  # Approve replacement
```

**Recovery time:** Instant (revoked devices blocked at next connection attempt)

### Certificate Expired

**Symptoms:**
- Device connection fails with "certificate expired" error
- 1 year since device registration (default validity)

**Response:**
```bash
aofctl device register --name <device-name> --type cli  # Issue new cert
aofctl device approve <new-device-id>  # Approve new device
aofctl device revoke <old-device-id>   # Revoke old certificate
```

**Note:** Future enhancement will auto-rotate certificates 30 days before expiry.

### CA Key Compromised

**Symptoms:**
- Unauthorized access to CA private key file
- Unknown devices appearing in registry
- Security alert from file integrity monitoring

**Response (Nuclear Option):**
```bash
# 1. Revoke ALL devices
for id in $(aofctl device list -o json | jq -r '.[].device_id'); do
  aofctl device revoke $id
done

# 2. Re-initialize CA (generates new root)
rm -rf ~/.local/share/aof/ca
aofctl init ca

# 3. Re-register all legitimate devices
# (each device needs new certificate)
```

**Impact:** All existing device certificates become invalid. All devices must re-register.

### Insider Threat

**Scenario:** Malicious operator registers and approves a rogue device.

**Detection:**
- Audit device registry for unexpected approvals
- Review approval history (`aofctl device inspect <device-id>`)
- Monitor connection logs for unusual patterns

**Prevention:**
- Restrict CA key file access to senior operators
- Require approval from multiple operators (future feature)
- Alert on device registrations outside business hours

## Best Practices

### 1. Protect the CA Key

The CA private key is the most critical secret:

**Do:**
- Store in secure location (`~/.local/share/aof/ca/` with 0700 permissions)
- Back up offline (encrypted USB drive, password manager)
- Restrict access to senior administrators only
- Monitor file access (file integrity monitoring)

**Don't:**
- Store in source control (even private repos)
- Share via email or messaging platforms
- Keep unencrypted backups on network storage

### 2. Regular Certificate Rotation

Limit certificate lifetime:

**Recommendations:**
- CLI/API clients: 1 year (default)
- Bots/automation: 90 days
- High-privilege devices: 30 days

**Automate rotation:**
```bash
# Cron job: Check certificate expiry
0 0 * * * aofctl device list --status approved | \
  jq -r 'select(.valid_until < now + (30 * 86400)) | .device_id' | \
  xargs -I {} aofctl device register --name {} --type cli
```

### 3. Monitor Device Activity

Track device behavior:

**Metrics to monitor:**
- Connection frequency (last_seen gaps)
- IP address changes (unexpected locations)
- Command patterns (unusual queries)
- Failure rates (authentication errors)

**Alerting:**
```bash
# Alert on new device registrations
aofctl device list --status pending | \
  jq 'if length > 0 then "ALERT: New pending devices" else empty end'
```

### 4. Separation of Duties

**Roles:**
- **CA Administrator:** Controls CA key, initializes CA
- **Device Approver:** Reviews and approves device registrations
- **Device Owner:** Operates approved devices

**Why separate?**
- Prevents single point of compromise
- Creates accountability trail
- Detects insider threats

### 5. Regular Audits

Quarterly reviews:

```bash
# List all approved devices
aofctl device list --status approved

# Check for stale devices (no activity in 90 days)
aofctl device list -o json | \
  jq 'select(.last_seen < now - (90 * 86400))'

# Review pending approvals
aofctl device list --status pending
```

## Comparison with Other Approaches

### vs. API Keys

**API Keys:**
```
Pros: Simple to implement, widely understood
Cons: Shared secret, no automatic expiry, manual rotation, hard to revoke
```

**mTLS:**
```
Pros: No shared secret, automatic expiry, instant revocation, audit trail
Cons: More complex setup, requires CA management
```

**Verdict:** mTLS is superior for production deployments with multiple devices.

### vs. OAuth/OIDC

**OAuth/OIDC:**
```
Pros: Centralized identity, token-based, automatic expiry
Cons: Requires external identity provider, network dependency, token refresh complexity
```

**mTLS:**
```
Pros: Self-contained, no external dependencies, works offline
Cons: No centralized identity, manual approval workflow
```

**Verdict:** mTLS is better for self-hosted, high-security environments. OAuth is better for SaaS integrations.

### vs. VPN + API Keys

**VPN + API Keys:**
```
Pros: Network-level isolation, familiar to IT teams
Cons: Broad access once inside VPN, API keys still shared secret
```

**mTLS:**
```
Pros: Per-device authentication, granular revocation, zero-trust model
Cons: Requires TLS configuration on all clients
```

**Verdict:** mTLS provides defense-in-depth and works well alongside VPNs.

## Frequently Asked Questions

**Q: Can I use Let's Encrypt certificates?**

A: No. Let's Encrypt and other public CAs no longer issue client authentication certificates (as of May 2026). They focus on server authentication (HTTPS).

**Q: What happens if the CA key is lost?**

A: All existing device certificates become unverifiable. You must:
1. Re-initialize the CA (new root certificate)
2. Re-register all devices
3. Re-approve all devices

This is why CA key backups are critical.

**Q: Can I use the same certificate on multiple devices?**

A: Technically yes, but highly discouraged:
- If one device is compromised, all devices using that cert are compromised
- You cannot selectively revoke access
- Audit trail becomes ambiguous (which device performed which action?)

Best practice: One certificate per device.

**Q: How do I connect from a device without a certificate?**

A: You cannot. mTLS requires a valid, approved certificate. For initial setup:
1. Generate certificate on server: `aofctl device register`
2. Securely transfer certificate to device (SCP, USB, etc.)
3. Approve device on server: `aofctl device approve`
4. Connect from device using transferred certificate

**Q: What if I accidentally revoke a critical device?**

A: Revocation is permanent. To restore access:
1. Register a new device: `aofctl device register --name <device> --type cli`
2. Transfer new certificate to device
3. Approve new device: `aofctl device approve <new-id>`
4. Update device configuration to use new certificate

**Q: Can I automate device approval?**

A: Not recommended for production. Approval workflow is a critical security control. However, for development/testing:
```bash
# Auto-approve all pending (TESTING ONLY)
aofctl device list --status pending -o json | \
  jq -r '.[].device_id' | \
  xargs -I {} aofctl device approve {}
```

## Next Steps

1. **Read the setup guide:** `docs/guides/device-pairing-setup.md`
2. **Initialize your CA:** `aofctl init ca`
3. **Register your first device:** `aofctl device register --name laptop --type cli`
4. **Approve and test:** `aofctl device approve <device-id>`

For deeper technical details, see `docs/dev/device-pairing.md`.
