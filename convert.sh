#!/bin/bash
for p in slides/*.png; do
   b=${p/png/bmp}
   echo "Converting $p to $b..."
   magick $p $b
   echo "Compressing $b..."
   gzip $b
done

