#!/usr/bin/env bash
set -euo pipefail

destination="${1:-resources}"
model_id="parakeet-tdt-0.6b-v2-smcleod-int8"
revision="d64884b484b919e9656d0b70cb95dfdc98852bef"
base_url="https://huggingface.co/smcleod/parakeet-tdt-0.6b-v2-int8/resolve/${revision}"
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
  curl -fL --retry 3 --retry-delay 2 --connect-timeout 30 -o "${model_dir}/${file}" "$url"
done

(
  cd "$model_dir"
  sha256sum -c - <<'EOF'
464e6ba9e9ee681361b52801d90f13be3cf01a9252a584f14023dab6197aedf9  encoder-model.int8.onnx
93fbb1856f8f897004a15793e7b78953879003e03fdac56dc76b76ccc2ce6505  decoder_joint-model.int8.onnx
34a4d9e18f9ebf7e69bbfb502a793d2a4c19ba538cd76db8abc5d3df0b2b74a5  nemo128.onnx
ec182b70dd42113aff6c5372c75cac58c952443eb22322f57bbd7f53977d497d  vocab.txt
6a2577a06b9de8dc201d08722d2fe8b27c5ba13cd5386ed3cf1edaa8807e7102  config.json
89d808d6df45ed5e01bc0368a4142ee52cb8674d1a8814ae40a29f6558446aaa  README.md
EOF
)

find "$model_dir" -maxdepth 1 -type f -printf '%p %s bytes\n' 2>/dev/null || ls -al "$model_dir"
