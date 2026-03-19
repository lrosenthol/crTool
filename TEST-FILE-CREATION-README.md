# Test File Formats

Reference documentation for the JSON file formats used by crTool's `--create-test` mode.

---

## Test Case JSON Format

Test case files drive the `--create-test` mode. They bundle all signing configuration — manifest, certificate, key, algorithm, and input asset — into a single reusable file. The schema is defined in `INTERNAL/schemas/test-case.schema.json`. All file paths are resolved relative to the test case file's directory.

```json
{
  "testId": "validator.claimSignature.valid.created",
  "title": "Valid Claim Signature — c2pa.created Action",
  "description": "Optional human-readable description of what this test verifies.",
  "specVersion": "2.2",
  "inputAsset": "../../tests/fixtures/assets/Dog.jpg",
  "manifest": {
    "alg": "Ed25519",
    "claim_generator_info": [{ "name": "crTool/0.3.0", "version": "0.3.0" }],
    "title": "tc-created",
    "assertions": [
      {
        "label": "c2pa.actions",
        "data": {
          "actions": [
            {
              "action": "c2pa.created",
              "digitalSourceType": "http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia",
              "when": "2026-01-17T14:44:19Z"
            }
          ]
        }
      }
    ],
    "ingredients": []
  },
  "signingCert": "../../tests/fixtures/certs/ed25519.pub",
  "signingKey": "../../tests/fixtures/certs/ed25519.pem",
  "expectedResults": {
    "validationStatus": [{ "code": "claimSignature.validated" }]
  }
}
```

### Fields

| Field             | Required | Description                                                                                                                                                                                               |
| ----------------- | -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `testId`          | Yes      | Unique identifier for the test case                                                                                                                                                                       |
| `title`           | No       | Human-readable title                                                                                                                                                                                      |
| `description`     | No       | Description of what the test verifies                                                                                                                                                                     |
| `specVersion`     | No       | C2PA specification version                                                                                                                                                                                |
| `inputAsset`      | No       | Path to the input media file (relative to this JSON file). Can be omitted when an input file is supplied on the command line, which always takes precedence. An error is returned if neither is provided. |
| `manifest`        | Yes      | C2PA manifest object (see [Manifest JSON Format](#manifest-json-format) below)                                                                                                                            |
| `signingCert`     | Yes      | Path to the signing certificate in PEM format (relative to this JSON file)                                                                                                                                |
| `signingKey`      | No       | Path to the private key in PEM format. Defaults to `signingCert` if omitted.                                                                                                                             |
| `tsaUrl`          | No       | Timestamp Authority URL                                                                                                                                                                                   |
| `expectedResults` | Yes      | Expected validation results (used by validators, not the tool itself)                                                                                                                                     |

**Algorithm auto-detection:** If `manifest.alg` is absent, the tool examines `signingCert` to determine the algorithm automatically (ES256/ES384/ES512 from ECDSA curve, Ed25519 from Ed25519 key, PS256 from RSA key).

---

## Manifest JSON Format

The `manifest` field inside a test case follows the c2pa-rs JSON manifest format:

```json
{
  "alg": "Ed25519",
  "claim_generator_info": [{ "name": "my-app/1.0.0", "version": "1.0.0" }],
  "title": "My Asset",
  "assertions": [
    {
      "label": "c2pa.actions",
      "data": {
        "actions": [
          {
            "action": "c2pa.edited",
            "when": "2024-01-07T12:00:00Z",
            "softwareAgent": "MyApp 1.0"
          }
        ]
      }
    }
  ],
  "ingredients": []
}
```

### Using File-Based Ingredients

Add entries with a `file_path` field to the `ingredients` array to load ingredient assets from files. Paths are resolved relative to the test case JSON file's directory.

```json
{
  "alg": "Ed25519",
  "claim_generator_info": [{ "name": "my-app/1.0.0", "version": "1.0.0" }],
  "title": "Edited Photo",
  "assertions": [
    {
      "label": "c2pa.actions",
      "data": {
        "actions": [
          {
            "action": "c2pa.placed",
            "when": "2024-01-07T12:00:00Z",
            "parameters": { "ingredientIds": ["source_image"] }
          }
        ]
      }
    }
  ],
  "ingredients": [
    {
      "file_path": "../../tests/fixtures/assets/Dog.jpg",
      "label": "source_image",
      "title": "Original Image",
      "relationship": "componentOf"
    }
  ]
}
```

Each ingredient entry supports:

| Field          | Required | Description                                                          |
| -------------- | -------- | -------------------------------------------------------------------- |
| `file_path`    | Yes      | Path to the ingredient file (relative to the test case JSON file)    |
| `title`        | No       | Human-readable title                                                 |
| `relationship` | No       | `"parentOf"` or `"componentOf"`                                      |
| `label`        | No       | Instance ID for referencing in actions via `ingredientIds`           |
| `metadata`     | No       | Object of custom key/value metadata fields attached to the ingredient |

---

## Test Cases Directory

The `test-cases/` directory contains pre-built test case files organized by conformance intent:

```
test-cases/
├── positive/             # Conformant assets — expect claimSignature.validated
│   ├── tc-created.json
│   ├── tc-changes-spatial.json
│   ├── tc-placed-with-ingredient.json
│   └── tc-opened-with-ingredient.json
└── negative/             # Non-conformant assets — validly signed but profile-violating
    ├── tc-n-created-nodst.json
    ├── tc-n-removed.json
    ├── tc-n-inception.json
    ├── tc-n-placed-empty-params.json
    └── tc-n-redacted-bad-reason.json
```

All test cases use the Ed25519 test certificates in `tests/fixtures/certs/` and `tests/fixtures/assets/Dog.jpg` as the default input asset.
