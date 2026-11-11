# HyperYap Local HTTP API

## Overview

The local HTTP API allows trusted local applications and CLI tools to send audio files to HyperYap for transcription without using the graphical interface.

**Status:** Experimental feature - API design may change in future releases.

## Quick Start

1. Open HyperYap
2. Go to **Settings** -> **System**
3. Find **Local API (Experimental)**
4. Toggle it **ON**
5. The API will start immediately on `http://127.0.0.1:4800`
6. (Optional) Change the port number if needed (default: `4800`)
7. Copy your `api_token` from `settings.json` (`%APPDATA%/com.avalonreset.hyperyap/settings.json` on Windows, `~/.config/com.avalonreset.hyperyap/settings.json` on Linux, or `~/Library/Application Support/com.avalonreset.hyperyap/settings.json` on macOS)

The API will remain running as long as HyperYap is open. It will stop when you close the application or toggle the API off in settings.

## Authentication and Loopback Security

To protect against cross-origin browser requests (CSRF) and DNS-rebinding attacks:

- Every request must include `Authorization: Bearer <api_token>` matching the `api_token` generated in `settings.json`.
- Requests carrying a browser `Origin` header or `Sec-Fetch-Site: cross-site` / `same-site` header are rejected with `403 Forbidden`.
- Requests with a `Host` header other than `127.0.0.1` or `localhost` (with optional port) are rejected with `403 Forbidden`.

## API Endpoint

**POST** `http://127.0.0.1:4800/api/transcribe`

### Request

Send a multipart form with an audio file field named `audio` containing a `.wav` file and your bearer token:

```bash
curl -X POST http://127.0.0.1:4800/api/transcribe \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -F "audio=@/path/to/audio.wav;type=audio/wav"
```

### Response

**Success (200 OK):**

```json
{
    "text": "Hello everyone, here is the complete transcript..."
}
```

**Error (4xx/5xx):**

```json
{
    "error": "Error message describing what went wrong"
}
```

## Requirements

- Audio file must be in **WAV format** (`.wav`)
- File is automatically resampled to 16kHz mono if needed
- Works best with complete sentences
- The default NVIDIA Parakeet TDT 0.6B v2 model is optimized for **English** transcription

## Usage Examples

### Python

```python
import requests

API_TOKEN = "YOUR_API_TOKEN"

with open("audio.wav", "rb") as f:
    files = {"audio": f}
    headers = {"Authorization": f"Bearer {API_TOKEN}"}
    response = requests.post(
        "http://127.0.0.1:4800/api/transcribe",
        headers=headers,
        files=files,
        timeout=60,
    )
    response.raise_for_status()
    result = response.json()
    print(result["text"])
```

### JavaScript/Node.js

```javascript
const fs = require('fs');
const FormData = require('form-data');
const axios = require('axios');

const API_TOKEN = process.env.HYPERYAP_API_TOKEN;

async function transcribe(audioPath) {
    const form = new FormData();
    form.append('audio', fs.createReadStream(audioPath));

    const response = await axios.post('http://127.0.0.1:4800/api/transcribe', form, {
        headers: {
            ...form.getHeaders(),
            Authorization: `Bearer ${API_TOKEN}`,
        },
    });

    console.log(response.data.text);
}

transcribe('recording.wav');
```

### Bash/curl

```bash
#!/usr/bin/env bash
set -euo pipefail

curl -fsSL -X POST http://127.0.0.1:4800/api/transcribe \
  -H "Authorization: Bearer ${HYPERYAP_API_TOKEN}" \
  -F "audio=@recording.wav" \
  | jq '.text'
```

## Important Notes

- **Security:** The API rejects browser `Origin` headers, validates `Host` against loopback (`127.0.0.1` / `localhost`), and requires `Authorization: Bearer <api_token>`.
- **Sequential Processing:** Transcription requests are processed sequentially due to the single transcription engine (concurrent requests will queue).
- **Custom Dictionary:** Custom dictionary and formatting rules are automatically applied to transcriptions.
- **Language Support:** The default Parakeet TDT 0.6B v2 INT8 model is English-focused (consistent with the main desktop app).
- **WAV Format Only:** Currently only supports WAV files. Other formats must be converted first.

## Troubleshooting

### Port Already in Use

If you get an "Address already in use" error:

- Change the port in Settings -> System to an unused port (`1024`-`65535`)
- Or close any other application using that port

### "Unauthorized" or "Forbidden" (401/403)

- Verify that you passed `-H "Authorization: Bearer <api_token>"` matching `api_token` in `settings.json`.
- Ensure you are calling from a non-browser client (no `Origin` header) and using `127.0.0.1` or `localhost` in the URL.

### "Model not available" Error

- Make sure HyperYap runs correctly in the UI and that your model files are not corrupted.
- If the issue persists, re-run `bash .github/scripts/download-parakeet-model.sh resources` or reinstall HyperYap.

### No Response from API

- Check that the API toggle is ON in Settings -> System
- Verify the port number matches your request
- Ensure HyperYap is still running
- Check your firewall is not blocking localhost access

### Slow Transcription

- First request will be slower (model warming up)
- Subsequent requests should be faster
- Very long audio files may take time to process
- Check that HyperYap is not being used for other tasks simultaneously

## Limitations

- Audio files must be in **WAV format** - other formats will return an error
- Maximum file size: 100 MB
- Only 16kHz mono audio is optimal (others are resampled automatically)
- Real-time streaming is not supported (only pre-recorded files)
- No request queueing status endpoint (submit one request at a time)
