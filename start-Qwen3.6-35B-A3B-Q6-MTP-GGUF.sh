/home/user/llama.cpp/build/bin/llama-server -m ./models/Qwen3.6-35B-A3B-Q6-MTP-GGUF/Qwen3.6-35B-A3B-UD-Q6_K.gguf --mmproj ./models/Qwen3.6-35B-A3B-Q6-MTP-GGUF/mmproj-BF16.gguf \
  --port 8080 \
  --host 0.0.0.0 \
  --poll 0 \
  --parallel 1 \
  --n-gpu-layers 999 \
  --n-cpu-moe 36 \
  --spec-type draft-mtp \
  --spec-draft-n-max 3 \
  --image-min-tokens 1024 \
  --temperature 0.7 \
  --cache-type-k q8_0 \
  --cache-type-v q8_0 \
  --reasoning-format auto \
  --ctx-size 163840 \
  --batch-size 512 \
  --ubatch-size 512 \
  --threads 8 \
  --threads-batch 8 \
  --flash-attn 1 \
  --jinja \
  --load-mode mlock

# Kontextgrößen: 32768 65536 81920 102400 122880 143360 159744 163840 204800 225280 262144


#   --cache-ram 16384
