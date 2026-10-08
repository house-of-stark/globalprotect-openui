# Code Signing & Notarization

GP OpenUI can be distributed as a **signed and notarized** macOS app, so users don't have to bypass Gatekeeper. This requires an [Apple Developer](https://developer.apple.com) account ($99/yr).

## What signing buys you

| | Unsigned | Signed + notarized |
|---|---|---|
| Gatekeeper warning on first launch | Yes (right-click → Open, or `xattr -dr com.apple.quarantine`) | No — opens normally |
| Apple malware scan (notarization) | No | Yes |
| Trust for enterprise/sysadmin users | Lower | Higher |

## One-time setup

### 1. Generate a Developer ID Application certificate

1. Go to [developer.apple.com/account/resources/certificates](https://developer.apple.com/account/resources/certificates)
2. Click **+** → **Developer ID Application** (under "Software → Developer ID").
3. Upload a Certificate Signing Request. Generate one locally if you don't have one:
   ```bash
   # From Keychain Access: Keychain Access → Certificate Assistant → Request a Certificate From a Certificate Authority
   # Or via CLI:
   openssl genrsa -out key.pem 2048
   openssl req -new -key key.pem -out request.csr \
     -subj "/emailAddress=you@example.com/CN=Developer ID Application: Your Name (TEAMID)/C=US"
   ```
4. Download the `.cer`, double-click to install into your login keychain.
5. To use it in CI, export it as a `.p12` with a password:
   ```bash
   # From Keychain Access: right-click the cert's private key → Export → .p12 → set a password
   ```
6. Base64-encode it for the CI secret:
   ```bash
   base64 -i "Developer ID Application - YOURNAME.p12" | pbcopy
   ```

### 2. Create an App Store Connect API key

1. Go to [appstoreconnect.apple.com/access/api](https://appstoreconnect.apple.com/access/api)
2. **Keys** → **+** → give it a name, select **Developer** access.
3. Download the `.p8` key file (this is your `APPLE_API_KEY`).
4. Note the **Issuer ID** (top of the page) and **Key ID** (from the key row).

## GitHub Actions secrets

Add these to the repo's **Settings → Secrets and variables → Actions**:

| Secret | Value |
|---|---|
| `APPLE_SIGNING_CERTIFICATE_BASE64` | The base64-encoded `.p12` (from step 1.6) |
| `APPLE_SIGNING_CERTIFICATE_PASSWORD` | Password you set on the `.p12` |
| `KEYCHAIN_PASSWORD` | A throwaway password for the CI keychain (any string) |
| `APPLE_SIGNING_IDENTITY` | `"Developer ID Application: Your Name (TEAMID)"` — get it via `security find-identity -v -p codesigning` |
| `APPLE_API_KEY` | Contents of the `.p8` file (full text, including `-----BEGIN PRIVATE KEY-----`) |
| `APPLE_API_ISSUER` | Issuer ID from App Store Connect |
| `APPLE_API_KEY_ID` | Key ID from App Store Connect |
| `TAURI_SIGNING_PRIVATE_KEY` | (optional) Tauri updater signing key |

## How it works

- `tauri.conf.json` sets `macOS.hardenedRuntime = true` (required for notarization).
- The CI workflow imports the `.p12` into a fresh keychain, then `tauri build` signs with `APPLE_SIGNING_IDENTITY` and notarizes automatically using the `APPLE_API_*` credentials (Tauri 2 calls `notarytool` under the hood).
- When signing secrets are absent, the workflow falls back to an **unsigned** build (current behavior).

## Local signing

To sign a local build (after installing the cert in your keychain):

```bash
export APPLE_API_KEY="$(cat AuthKey_XXXXXXXXXX.p8)"
export APPLE_API_ISSUER="your-issuer-id"
export APPLE_API_KEY_ID="your-key-id"
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
pnpm tauri build
```
