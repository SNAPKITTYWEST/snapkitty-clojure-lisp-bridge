#!/bin/bash
# Record demo.html as MP4 using ffmpeg screen capture

OUTPUT="demo.mp4"
DURATION="20"  # 20 second recording
FPS="30"

echo "[Record] Starting screen capture..."
echo "[Record] Open demo.html in browser and click 'Run Full Demo'"
echo "[Record] Recording for ${DURATION} seconds..."

# Windows screen capture via ffmpeg
ffmpeg -f gdigrab -framerate $FPS -i desktop \
  -c:v libx264 -preset medium -crf 23 -pix_fmt yuv420p \
  -t $DURATION "$OUTPUT"

if [ -f "$OUTPUT" ]; then
  SIZE=$(du -h "$OUTPUT" | cut -f1)
  echo "[Record] ✓ Video saved: $OUTPUT ($SIZE)"
else
  echo "[Record] ✗ Failed to create video"
  exit 1
fi
