#!/bin/bash

# Create cinematic title card (3.5s) + demo (8s) = 11.5s GIF

# Title card with cyan border, text overlay, status line
ffmpeg -f lavfi -i "color=c=0x020508:s=1280x800:d=3.5" \
  -vf "
  drawbox=x=20:y=20:w=1240:h=760:color=0x56e8ff:thickness=2,
  drawtext=text='SOVEREIGN KNOWLEDGE ENGINE':fontfile='C\:/Windows/Fonts/Consola.ttf':fontsize=48:fontcolor=0x56e8ff:x=(w-text_w)/2:y=120:shadowx=2:shadowy=2,
  drawtext=text='YOUR CODE. YOUR KNOWLEDGE. YOUR MACHINE.':fontfile='C\:/Windows/Fonts/Consola.ttf':fontsize=32:fontcolor=0xa9f7ff:x=(w-text_w)/2:y=220:enable='between(t,0.5,3)',
  drawtext=text='NO CLOUD RUNTIME. NO API DEPENDENCY. NO TRUST REQUIRED.':fontfile='C\:/Windows/Fonts/Consola.ttf':fontsize=24:fontcolor=0x8eb5c1:x=(w-text_w)/2:y=320:enable='between(t,1,3)',
  drawtext=text='CLICK TO ENTER.':fontfile='C\:/Windows/Fonts/Consola.ttf':fontsize=28:fontcolor=0x38ffc7:x=(w-text_w)/2:y=450:enable='between(t,1.5,3)',
  drawtext=text='LISP ONLINE  |  LTMS ONLINE  |  PROOFS VERIFIED':fontfile='C\:/Windows/Fonts/Consola.ttf':fontsize=18:fontcolor=0x38ffc7:x=(w-text_w)/2:y=700
  " \
  -c:v libx264 -preset medium -pix_fmt yuv420p /tmp/title-card.mp4

# Concatenate: title card (3.5s) + demo (20s)
cat > /tmp/concat.txt << 'EOF'
file '/tmp/title-card.mp4'
file 'docs/demo.mp4'
EOF

ffmpeg -f concat -safe 0 -i /tmp/concat.txt -c copy /tmp/combined.mp4

# Convert to GIF (12 FPS for compact file)
ffmpeg -i /tmp/combined.mp4 -vf "fps=12,scale=1280:-1" \
  -loop 0 assets/sovereign-runtime-demo.gif

echo "✓ Enhanced GIF created: assets/sovereign-runtime-demo.gif"
ls -lh assets/sovereign-runtime-demo.gif
