# The GST Portal's Local Signer Protocol

**Provenance.** Everything here comes from what any user can observe: the
portal's own JavaScript (served to every browser), WebSocket frames captured
from our own sessions, and ASN.1 dumps of signatures our own tokens produced.
Nothing here is taken from decompiled vendor software.

Code: `crates/protocol` (frames), `crates/cms` (signature), `crates/server`
(transport).

## Transport

- The portal opens `new WebSocket("wss://127.0.0.1:" + port)`: TLS only.
- Ports are tried in order: **1585, 2095, 2568, 2868, 4587**. On a connection
  error, or a greeting that fails validation, the portal moves to the next
  port. Swakshar binds the first free one (`bind_signer_port`).

## Greeting: the server speaks first

Sent as soon as the WebSocket opens:

```text
status = success
port = 1585
version = 2.8
ID = gstnInfy
```

The portal splits the frame on `\n`, splits each line on `=`, trims both sides,
and needs both `version` equal to its `LATEST_EM_VERSION` and `ID` equal to
`gstnInfy`. If no port passes it shows "Please install and use correct version
of emSigner". A separate check looks for the literal `status = success`, with
the spaces. The version is therefore a setting (default `2.8`, accepted by the
live portal in August 2026).

## Request

One text frame, `key=value` per line:

```text
action=sign
tobesigned=<text>
panNo=<PAN>
signtype=1
expirycheck=true
issuername=
certclass=2|3
certtype=DSC
certdetails=
```

- `tobesigned` is plain text: the PAN for DSC registration, a 64-character hex
  digest for returns. It is signed as UTF-8 bytes, not base64-decoded.
- Keys are matched case-insensitively, values trimmed, last duplicate wins.
- Swakshar rejects frames over 16 KiB, any action but `sign`, and any
  `signtype` but `1` (the only one observed).

## Reply

Success is one text frame:

```text
signature= <base64url CMS, no padding>
SerialNo= <certificate serial, decimal>
CommonName= <subject CN>
IssuerName= <issuer CN>
IssuedDate= <dd-MM-yyyy>
ExpiryDate= <dd-MM-yyyy>
CertClass= Class 3
SigningTime= <dd-MM-yyyy HH:mm:ss, IST>
uniqueID= <echo of uniqueId, possibly empty>
```

The portal extracts the signature with
`data.substring(11, data.indexOf("SerialNo") - 1)`. So the frame starts with
exactly `signature= ` (11 characters) and has exactly one `\n` before
`SerialNo`. Values are flattened to one line so nothing can break that layout.

Failure is the bare string `signing canceled` (declined, timed out) or
`signing failed` (anything else). The portal shows "Signing Cancelled" for
both. It then posts the signature to its own server, which verifies it against
the DSC registered for the PAN.

## The signature (`signtype=1`)

```text
ContentInfo
  contentType        id-signedData
  content
    SignedData
      version            1
      digestAlgorithms   { sha1, NULL }
      encapContentInfo   id-data, eContent = the tobesigned bytes (attached)
      certificates       the signer certificate only
      signerInfos        exactly one
        version            1
        sid                issuerAndSerialNumber
        digestAlgorithm    sha1, NULL
        signedAttrs        contentType (id-data), signingTime (UTCTime), messageDigest (SHA-1 of content)
        signatureAlgorithm rsaEncryption, NULL
        signature          RSASSA-PKCS1-v1_5 SHA-1 over DER(signedAttrs as SET OF)
```

Encoded for the reply as URL-safe base64 without padding.

Observed rules:

- **Nothing extra.** Attributes such as `cmsAlgorithmProtection` or
  `signingCertificateV2` make the portal answer "Multiple signed data is not
  allowed". So the structure is assembled field by field (`crates/cms`).
- **Signer certificate only**, no chain.
- **Attached content**; detached content fails verification.
- **DER order.** `SET OF` sorting puts the attributes in the order contentType,
  signingTime, messageDigest for SHA-1 digests.

## Certificate selection hints

| Field | Meaning in Swakshar |
|---|---|
| `panNo` | The CCA puts SHA-256 of the PAN in the subject `serialNumber` (CCA-IOG). The matching certificate is offered first; others are flagged |
| `expirycheck` | Hide certificates outside their validity period |
| `certclass` | Require CCA policy `2.16.356.100.2.2` or `2.16.356.100.2.3` |
| `certtype` | Signing certificates only (digitalSignature or nonRepudiation) |
| `issuername` | When non-empty, the issuer CN must contain it |

## Known unknowns

| Question | Current answer |
|---|---|
| Live `LATEST_EM_VERSION` | `2.8` in August 2026; configurable in Settings |
| Other `signtype` values | None observed; refused and recorded |
| How long the portal waits | Long enough for a PIN; Swakshar gives five minutes |
