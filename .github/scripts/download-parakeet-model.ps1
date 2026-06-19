param(
    [string]$Destination = "resources"
)

$ErrorActionPreference = "Stop"

$modelId = "parakeet-tdt-0.6b-v2-smcleod-int8"
$baseUrl = "https://huggingface.co/smcleod/parakeet-tdt-0.6b-v2-int8/resolve/main"
$files = @(
    "encoder-model.int8.onnx",
    "decoder_joint-model.int8.onnx",
    "nemo128.onnx",
    "vocab.txt",
    "config.json",
    "README.md"
)

$modelDir = Join-Path $Destination $modelId
New-Item -ItemType Directory -Force -Path $modelDir | Out-Null

foreach ($file in $files) {
    $out = Join-Path $modelDir $file
    $url = "$baseUrl/$file"
    Write-Host "Downloading $url"
    & curl.exe -L --retry 3 --retry-delay 2 --connect-timeout 30 -o $out $url
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to download $file"
    }
}

foreach ($required in @("encoder-model.int8.onnx", "decoder_joint-model.int8.onnx", "nemo128.onnx", "vocab.txt")) {
    if (!(Test-Path -LiteralPath (Join-Path $modelDir $required))) {
        throw "Missing required model file: $required"
    }
}

Get-ChildItem -LiteralPath $modelDir | Select-Object FullName, Length
