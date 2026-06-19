#!/usr/bin/env bash
set -euo pipefail

destination="${1:-resources}"
model_id="parakeet-tdt-0.6b-v2-smcleod-int8"
base_url="https://huggingface.co/smcleod/parakeet-tdt-0.6b-v2-int8/resolve/main"
model_dir="${destination}/${model_id}"

mkdir -p "$model_dir"

files=(
  "encoder-model.int8.onnx"
  "decoder_joint-model.int8.onnx"
  "nemo128.onnx"
  "vocab.txt"
  "config.json"
  "README.md"
)

for file in "${files[@]}"; do
  url="${base_url}/${file}"
  echo "Downloading ${url}"
  curl -L --retry 3 --retry-delay 2 --connect-timeout 30 -o "${model_dir}/${file}" "$url"
done

for required in encoder-model.int8.onnx decoder_joint-model.int8.onnx nemo128.onnx vocab.txt; do
  test -f "${model_dir}/${required}"
done

find "$model_dir" -maxdepth 1 -type f -printf '%p %s bytes\n' 2>/dev/null || ls -al "$model_dir"
