# Device Pairing Setup Guide

**Audience:** System Administrators, DevOps Engineers
**Time Required:** 15-20 minutes
**Prerequisites:** AOF daemon installed, `aofctl` in PATH

## Overview

This guide walks you through setting up device pairing with mTLS authentication. You'll:

1. Initialize a private Certificate Authority (CA)
2. Start the AOF daemon with mTLS enabled
3. Register a client device
4. Approve the device
5. Connect using mTLS

## Step 1: Initialize the Private CA

The Certificate Authority (CA) issues and signs client certificates.

```bash
$ aofctl init ca
```

**Expected Output:**
```
Initializing private Certificate Authority...
✓ CA initialized successfully
  CA directory: /Users/admin/.local/share/aof/ca
  CA certificate: /Users/admin/.local/share/aof/ca/ca.crt
  CA private key: /Users/admin/.local/share/aof/ca/ca.key

⚠ Security Warning:
  - Keep ca.key secure (permissions: 0600)
  - Back up the CA directory
  - CA certificate is valid for 10 years
```

**Verify CA Files:**
```bash
$ ls -la ~/.local/share/aof/ca/
total 8
-rw-------  1 admin  staff  1679 Feb 14 10:00 ca.key   # Private key (0600)
-rw-r--r--  1 admin  staff  1318 Feb 14 10:00 ca.crt   # Public certificate
```

**Security Checkpoint:**
- ✅ CA key has 0600 permissions (owner read/write only)
- ✅ CA directory is in a secure location
- ✅ Backup CA directory to offline storage

## Step 2: Generate Server Certificate

The AOF daemon needs its own certificate for the TLS handshake.

```bash
$ aofctl device register \
    --name aof-server \
    --type api_client \
    --validity-days 365
```

**Expected Output:**
```
✓ Device registered successfully
  Device ID: 550e8400-e29b-41d4-a716-446655440000
  Name: aof-server
  Type: api_client
  Status: Pending
  Valid until: 2027-02-14

  Certificates saved to:
    ~/.local/share/aof/devices/550e8400-e29b-41d4-a716-446655440000/client.crt
    ~/.local/share/aof/devices/550e8400-e29b-41d4-a716-446655440000/client.key
    ~/.local/share/aof/devices/550e8400-e29b-41d4-a716-446655440000/ca.crt

  Next steps:
    1. Approve this device: aofctl device approve 550e8400-e29b-41d4-a716-446655440000
    2. Connect using mTLS with the certificates above
```

**Approve the Server Certificate:**
```bash
$ aofctl device approve 550e8400-e29b-41d4-a716-446655440000
```

**Copy Server Certificate (for serve command):**
```bash
$ SERVER_CERT_DIR=~/.local/share/aof/devices/550e8400-e29b-41d4-a716-446655440000
$ mkdir -p ~/.local/share/aof/server/
$ cp $SERVER_CERT_DIR/client.crt ~/.local/share/aof/server/server.crt
$ cp $SERVER_CERT_DIR/client.key ~/.local/share/aof/server/server.key
$ cp $SERVER_CERT_DIR/ca.crt ~/.local/share/aof/server/ca.crt
```

## Step 3: Start Daemon with mTLS

Start the AOF daemon with mTLS enabled:

```bash
$ aofctl serve \
    --mtls \
    --ca-cert ~/.local/share/aof/server/ca.crt \
    --server-cert ~/.local/share/aof/server/server.crt \
    --server-key ~/.local/share/aof/server/server.key \
    --port 8080
```

**Expected Output:**
```
Starting AOF Trigger Server
  Bind address: 0.0.0.0:8080
  Event bus: initialized (buffer: 1000)
  Session ID: 7c9e6679-7425-40de-944b-e07fc1f90ae7
  Workspace root: /Users/admin/aof-project
  mTLS: enabled
    CA cert: ~/.local/share/aof/server/ca.crt
    Client auth: required
  Device registry: initialized (0 approved devices)

Server starting...
  Health check: http://0.0.0.0:8080/health
  WebSocket: ws://0.0.0.0:8080/ws
  Webhook endpoint: http://0.0.0.0:8080/webhook/{platform}

Press Ctrl+C to stop
```

**Verify mTLS is Active:**
```bash
# Try connecting without a certificate (should fail)
$ curl https://localhost:8080/health
curl: (35) error:14094410:SSL routines:SSL3_READ_BYTES:sslv3 alert handshake failure
```

Good! The server rejects connections without client certificates.

## Step 4: Register a Client Device

On the machine where you want to run `aofctl` commands:

```bash
$ aofctl device register \
    --name mission-control-laptop \
    --type cli
```

**Expected Output:**
```
✓ Device registered successfully
  Device ID: 123e4567-e89b-12d3-a456-426614174000
  Name: mission-control-laptop
  Type: cli
  Status: Pending
  Valid until: 2027-02-14

  Certificates saved to:
    ~/.local/share/aof/devices/123e4567-e89b-12d3-a456-426614174000/client.crt
    ~/.local/share/aof/devices/123e4567-e89b-12d3-a456-426614174000/client.key
    ~/.local/share/aof/devices/123e4567-e89b-12d3-a456-426614174000/ca.crt

  Next steps:
    1. Approve this device: aofctl device approve 123e4567-e89b-12d3-a456-426614174000
    2. Connect using mTLS with the certificates above
```

**Save Device ID for Later:**
```bash
$ export DEVICE_ID=123e4567-e89b-12d3-a456-426614174000
$ export CERT_DIR=~/.local/share/aof/devices/$DEVICE_ID
```

## Step 5: Approve the Device

List pending devices:

```bash
$ aofctl device list --status pending
```

**Expected Output:**
```
┌────────────────────────────────────┬─────────────────────────┬──────┬─────────┬───────────┬────┐
│ DEVICE ID                          │ NAME                    │ TYPE │ STATUS  │ LAST SEEN │ IP │
├────────────────────────────────────┼─────────────────────────┼──────┼─────────┼───────────┼────┤
│ 123e4567-e89b-12d3-a456-426614174000│ mission-control-laptop  │ cli  │ pending │ Never     │ -  │
└────────────────────────────────────┴─────────────────────────┴──────┴─────────┴───────────┴────┘

Total: 1 device(s)
```

Approve the device:

```bash
$ aofctl device approve $DEVICE_ID
```

**Expected Output:**
```
✓ Device approved successfully
  Device ID: 123e4567-e89b-12d3-a456-426614174000
  Approved by: admin
  Approved at: 2026-02-14 10:15:30

The device can now connect using mTLS.
```

## Step 6: Connect with mTLS

Test the connection:

```bash
$ curl --cert $CERT_DIR/client.crt \
       --key $CERT_DIR/client.key \
       --cacert $CERT_DIR/ca.crt \
       https://localhost:8080/health
```

**Expected Output:**
```json
{
  "status": "healthy",
  "timestamp": "2026-02-14T10:20:00Z"
}
```

**Success!** The device connected using mTLS authentication.

## Step 7: Configure aofctl for mTLS (Optional)

Instead of passing certificate flags every time, configure `aofctl`:

**Create aofctl config file:**
```bash
$ mkdir -p ~/.aof/
$ cat > ~/.aof/config.yaml <<EOF
server:
  url: https://localhost:8080
  tls:
    client_cert: $CERT_DIR/client.crt
    client_key: $CERT_DIR/client.key
    ca_cert: $CERT_DIR/ca.crt
EOF
```

**Test aofctl with config:**
```bash
$ aofctl --config ~/.aof/config.yaml status
```

## Verification Checklist

After setup, verify:

- [ ] CA initialized at `~/.local/share/aof/ca/`
- [ ] Server certificate generated and approved
- [ ] Daemon started with `--mtls` flag
- [ ] Client device registered and approved
- [ ] mTLS connection succeeds with valid certificate
- [ ] Connection fails without certificate (security check)
- [ ] Device shows in approved list: `aofctl device list --status approved`

## Troubleshooting

### Problem: "CA not found" Error

**Symptom:**
```
Error: CA certificate or key not found. Run 'aofctl init ca' first.
```

**Solution:**
```bash
# Re-initialize CA
$ aofctl init ca

# Verify files exist
$ ls -la ~/.local/share/aof/ca/
```

### Problem: Connection Refused (403 Unauthorized)

**Symptom:**
```
curl: (22) The requested URL returned error: 403 Forbidden
```

**Cause:** Device not approved or revoked.

**Solution:**
```bash
# Check device status
$ aofctl device list | grep $DEVICE_ID

# If Pending, approve it
$ aofctl device approve $DEVICE_ID

# If Revoked, register a new device
$ aofctl device register --name my-device --type cli
```

### Problem: Certificate Verification Failed

**Symptom:**
```
curl: (60) SSL certificate problem: unable to get local issuer certificate
```

**Cause:** Client using wrong CA certificate or server cert not signed by CA.

**Solution:**
```bash
# Verify CA cert matches
$ diff ~/.local/share/aof/ca/ca.crt $CERT_DIR/ca.crt

# If different, copy correct CA cert
$ cp ~/.local/share/aof/ca/ca.crt $CERT_DIR/ca.crt
```

### Problem: Server Won't Start with mTLS

**Symptom:**
```
Error: Failed to build TLS config: Failed to parse server key: No valid private key found in PEM
```

**Cause:** Server key file corrupted or wrong format.

**Solution:**
```bash
# Regenerate server certificate
$ aofctl device register --name aof-server --type api_client
$ aofctl device approve <new-device-id>

# Copy to server directory
$ cp ~/.local/share/aof/devices/<new-device-id>/* ~/.local/share/aof/server/
```

### Problem: Device Not Appearing in List

**Symptom:**
```
$ aofctl device list
No devices found.
```

**Cause:** Registry file missing or corrupt.

**Solution:**
```bash
# Check registry file
$ cat ~/.local/share/aof/devices/registry.json

# If empty/missing, register a new device
$ aofctl device register --name first-device --type cli
```

## Security Best Practices

### 1. Back Up CA Directory

```bash
# Create encrypted backup
$ tar czf - ~/.local/share/aof/ca | \
  gpg --symmetric --cipher-algo AES256 > aof-ca-backup.tar.gz.gpg

# Store on offline media (USB drive, password manager vault)
```

### 2. Rotate Certificates Regularly

```bash
# Check certificate expiry
$ openssl x509 -in $CERT_DIR/client.crt -noout -dates

# Re-issue before expiry
$ aofctl device register --name my-device --type cli
$ aofctl device approve <new-device-id>
$ aofctl device revoke <old-device-id>
```

### 3. Monitor Device Activity

```bash
# List all approved devices
$ aofctl device list --status approved

# Check for inactive devices (potential for revocation)
$ aofctl device list -o json | \
  jq '.[] | select(.last_seen < (now - 7776000))' # 90 days
```

### 4. Restrict CA Key Access

```bash
# Verify permissions
$ ls -l ~/.local/share/aof/ca/ca.key
-rw------- 1 admin staff 1679 Feb 14 10:00 ca.key

# If wrong permissions
$ chmod 600 ~/.local/share/aof/ca/ca.key
```

## Advanced Configuration

### Multi-Device Setup

For organizations with many devices:

```bash
# Register multiple devices
for device in laptop-001 laptop-002 laptop-003; do
  aofctl device register --name $device --type cli
done

# Bulk approve (after review)
for id in $(aofctl device list --status pending -o json | jq -r '.[].device_id'); do
  aofctl device approve $id --approved-by admin
done
```

### Certificate Validity Periods

Customize certificate lifetimes:

```bash
# Short-lived (30 days) for high-security devices
$ aofctl device register --name secure-client --type cli --validity-days 30

# Long-lived (2 years) for stable infrastructure
$ aofctl device register --name prod-server --type api_client --validity-days 730
```

### Device Metadata

Track additional device information:

```bash
# After registration, inspect and add metadata
$ aofctl device inspect $DEVICE_ID
# Manually edit registry.json to add metadata fields
```

## Next Steps

Now that device pairing is set up:

1. **Configure other devices:** Register and approve additional clients
2. **Enable mTLS in production:** Add `--mtls` flag to production daemon
3. **Set up monitoring:** Track device connections and certificate expiry
4. **Plan certificate rotation:** Schedule quarterly certificate updates

For deeper understanding, read:
- **Concepts:** `docs/concepts/device-security.md`
- **Architecture:** `docs/dev/device-pairing.md`
