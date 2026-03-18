# C2PA Validator Test Case Description

## Overview

This document describes the JSON-based grammar for defining **C2PA validator test cases** used in automated conformance assessment. Each test case fully specifies how to produce a C2PA-signed test asset and what a conformant C2PA validator is expected to report when it processes that asset.

The grammar is formally defined in [`docs/schemas/test-case/test-case.schema.json`](schemas/test-case/test-case.schema.json) using [JSON Schema (Draft 2020-12)](https://json-schema.org/draft/2020-12).

### Purpose

Test cases enable automated, repeatable conformance assessment of C2PA **Validator Products**. A test harness can use a test case to:

1. Take the specified **input asset** and embed a **manifest** (as defined in crJSON format).
2. Sign the manifest with the specified **signing certificate** (and optionally obtain a timestamp from a **TSA**).
3. Run the resulting signed asset through the Validator Product under test.
4. Compare the validator's output against the **expected results** defined in the test case.

---

## Schema Reference

The schema file is located at:

```
docs/schemas/test-case/test-case.schema.json
```

### Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `testId` | string | ✅ | Unique identifier for the test case. SHOULD use dot-notation (e.g., `validator.claimSignature.valid`). |
| `title` | string | | Short, human-readable title. |
| `description` | string | | Detailed explanation of what the test case verifies. |
| `specVersion` | string | | The C2PA Content Credentials specification version targeted by this test case (e.g., `"2.2"`). |
| `inputAsset` | string | | Relative path to the raw media asset (image, video, audio, document) into which the manifest will be embedded. Optional — can be omitted when the input file is supplied on the command line. When both are present, the command-line value takes precedence. An error is returned if neither is provided at runtime. |
| `manifest` | object | ✅ | Manifest declaration/definition in crJSON format. Specifies what content to embed during test asset generation. See [Manifest Object](#manifest-object). |
| `signingCert` | string | ✅ | Relative path to the PEM-encoded X.509 signing certificate (and chain) used to sign the manifest. |
| `signingKey` | string | | Relative path to the PEM-encoded private key corresponding to `signingCert`. |
| `tsaUrl` | string (URI) | | URL of an RFC 3161-compliant Time-Stamping Authority. If omitted, no timestamp token is embedded in the signed asset. |
| `expectedResults` | object | ✅ | Expected validation output from a conformant validator. See [Expected Results Object](#expected-results-object). |

---

### Manifest Object

The `manifest` field contains a crJSON manifest definition—the same format accepted by tools such as [c2patool](https://github.com/contentauth/c2patool). It describes the C2PA manifest that will be embedded and signed into the input asset.

| Field | Type | Description |
|-------|------|-------------|
| `alg` | string | Signing algorithm (e.g., `"Es256"`, `"Es384"`, `"Es512"`, `"Ps256"`, `"Ed25519"`). |
| `claim_generator` | string | Identifies the software that produced this manifest (e.g., `"c2patool/0.9.0"`). |
| `claim_generator_info` | array | Structured metadata about the claim generator software. Each item has at least a `name` field. |
| `title` | string | Human-readable title to assign to the manifest and its asset. |
| `format` | string | MIME type of the asset (e.g., `"image/jpeg"`, `"audio/mpeg"`). MUST match the actual format of `inputAsset`. |
| `assertions` | array | Array of C2PA assertion objects. Each has a `label` (e.g., `"c2pa.actions"`) and a `data` payload. |
| `ingredients` | array | Array of ingredient objects for source assets used in producing this asset. |
| `credentials` | array | Array of W3C Verifiable Credentials to include in the manifest. |
| `redacted_assertions` | array | Array of JUMBF URI labels identifying assertions to redact. |

Additional crJSON manifest properties not listed above are permitted (`additionalProperties: true`).

---

### Expected Results Object

The `expectedResults` field describes the validation output that a conformant C2PA validator MUST produce when processing the test asset.

```json
"expectedResults": {
  "validationStatus": [ ... ]
}
```

#### `validationStatus` (array, required)

An array of expected validation status entries, modelled after the `validation_status` array in crJSON output. Each entry specifies a status code that MUST appear in the validator's output.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `code` | string | ✅ | A C2PA validation status code from the C2PA Content Credentials specification (e.g., `"claimSignature.validated"`, `"hardBindings.mismatch"`). |
| `url` | string | | Optional JUMBF URI fragment identifying the manifest component to which this status applies. |
| `explanation` | string | | Human-readable description of why this status code is expected. |

Common validation status codes (see C2PA Content Credentials specification, §15 for the full list):

| Code | Meaning |
|------|---------|
| `claimSignature.validated` | The claim signature is cryptographically valid. |
| `claimSignature.failed` | The claim signature is invalid. |
| `hardBindings.match` | Hard bindings (content hash) match the signed asset. |
| `hardBindings.mismatch` | Asset content was modified after signing. |
| `signingCredential.untrusted` | The signing certificate is not in a trusted chain. |
| `signingCredential.ocsp.revoked` | The signing certificate has been revoked. |
| `timeStamp.validated` | The RFC 3161 timestamp is valid. |
| `timeStamp.mismatch` | The timestamp does not match the claim signature. |

---

## Examples

### Example 1 — Valid, Trusted Asset

This test case verifies that a validator correctly reports a valid signature for an unmodified asset signed with a trusted certificate.

```json
{
  "testId": "validator.claimSignature.valid",
  "title": "Valid Claim Signature",
  "description": "Verifies that a conformant validator correctly reports a valid claim signature when the asset has not been tampered with and the signing certificate is trusted.",
  "specVersion": "2.2",
  "inputAsset": "assets/sample.jpg",
  "manifest": {
    "alg": "Es256",
    "claim_generator": "c2patool/0.9.0",
    "title": "Sample Test Asset",
    "format": "image/jpeg",
    "assertions": [
      {
        "label": "c2pa.actions",
        "data": {
          "actions": [
            {
              "action": "c2pa.created",
              "digitalSourceType": "http://cv.iptc.org/newscodes/digitalsourcetype/digitalCapture"
            }
          ]
        }
      }
    ]
  },
  "signingCert": "certs/test-signing.pem",
  "signingKey": "certs/test-signing.key",
  "tsaUrl": "http://timestamp.digicert.com",
  "expectedResults": {
    "validationStatus": [
      {
        "code": "claimSignature.validated",
        "explanation": "The claim signature must be valid for an unmodified, correctly signed asset."
      }
    ]
  }
}
```

### Example 2 — Tampered Asset (Hard Binding Mismatch)

This test case verifies that a validator correctly detects content modification after signing.

```json
{
  "testId": "validator.hardBindings.mismatch",
  "title": "Hard Binding Mismatch (Asset Tampered)",
  "description": "Verifies that a conformant validator correctly detects that the asset content has been modified after signing, resulting in a hard binding mismatch.",
  "specVersion": "2.2",
  "inputAsset": "assets/tampered-sample.jpg",
  "manifest": {
    "alg": "Es256",
    "claim_generator": "c2patool/0.9.0",
    "title": "Tampered Test Asset",
    "format": "image/jpeg",
    "assertions": [
      {
        "label": "c2pa.actions",
        "data": {
          "actions": [
            {
              "action": "c2pa.created",
              "digitalSourceType": "http://cv.iptc.org/newscodes/digitalsourcetype/digitalCapture"
            }
          ]
        }
      }
    ]
  },
  "signingCert": "certs/test-signing.pem",
  "signingKey": "certs/test-signing.key",
  "expectedResults": {
    "validationStatus": [
      {
        "code": "hardBindings.mismatch",
        "explanation": "A hard binding mismatch must be reported because the asset content was altered after signing."
      }
    ]
  }
}
```

### Example 3 — AI-Generated Audio with Untrusted Certificate

This test case illustrates an audio-specific scenario (relevant to the Audio TF use case) where the signing certificate is not on the C2PA Trust List.

```json
{
  "testId": "validator.audio.aiGenerated.untrustedCert",
  "title": "AI-Generated Audio — Untrusted Signing Certificate",
  "description": "Verifies that a validator correctly reports an untrusted signing credential for an AI-generated audio asset signed with a development certificate that is not on the C2PA Trust List.",
  "specVersion": "2.2",
  "inputAsset": "assets/ai-generated-audio.mp3",
  "manifest": {
    "alg": "Es256",
    "claim_generator": "c2patool/0.9.0",
    "title": "AI-Generated Audio Sample",
    "format": "audio/mpeg",
    "assertions": [
      {
        "label": "c2pa.actions",
        "data": {
          "actions": [
            {
              "action": "c2pa.created",
              "digitalSourceType": "http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia"
            }
          ]
        }
      }
    ]
  },
  "signingCert": "certs/dev-signing.pem",
  "signingKey": "certs/dev-signing.key",
  "expectedResults": {
    "validationStatus": [
      {
        "code": "signingCredential.untrusted",
        "explanation": "The development signing certificate is not on the C2PA Trust List, so the validator must report an untrusted signing credential."
      }
    ]
  }
}
```

---

## File Layout Convention

A test suite is recommended to follow this directory structure:

```
test-suite/
├── assets/                  # Raw input media assets
│   ├── sample.jpg
│   ├── tampered-sample.jpg
│   └── ai-generated-audio.mp3
├── certs/                   # Signing certificates and private keys
│   ├── test-signing.pem
│   ├── test-signing.key
│   ├── dev-signing.pem
│   └── dev-signing.key
└── test-cases/              # Test case description files (one per test)
    ├── validator.claimSignature.valid.json
    ├── validator.hardBindings.mismatch.json
    └── validator.audio.aiGenerated.untrustedCert.json
```

All paths in `inputAsset`, `signingCert`, and `signingKey` are resolved relative to the test case JSON file's directory. `inputAsset` may be omitted from the JSON and supplied instead as a positional argument on the command line, which allows the same test case file to be reused across different input assets.

---

## Validation

Test case description files SHOULD be validated against the JSON Schema before use:

```
docs/schemas/test-case/test-case.schema.json
```

The CI workflow in this repository automatically lints all `*.schema.json` files on commit using the [sourcemeta/jsonschema](https://github.com/sourcemeta/jsonschema) linter.
