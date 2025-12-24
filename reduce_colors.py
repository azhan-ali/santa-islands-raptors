from PIL import Image
import os

files = [
    "sprites/thumb_factory.jpg",
    "sprites/thumb_sleigh.jpg",
    "sprites/thumb_breaker.jpg",
    "sprites/thumb_stealth.jpg"
]

for f in files:
    try:
        # Open
        img = Image.open(f)
        
        # Resize small (to reduce color bleeding details and fit icon usage)
        # Original intent was 60x60, so let's resize to 64x64 or 128x128
        img = img.resize((64, 64), Image.NEAREST)
        
        # Quantize to 256 colors maximum (8-bit)
        img = img.quantize(colors=64) 
        
        # Convert back to RGB to save as jpg if needed, or keeping P mode for PNG is fine but user asked for JPG.
        # However, JPG compression adds noise (more colors). PNG is safer for limited palette.
        # But User insisted on JPG/renaming.
        # Let's try saving as PNG first with the same name (replace extension if needed, but here files are already .jpg named).
        # We will overwrite them.
        
        img = img.convert("RGB") # JPG needs RGB
        
        # Save
        img.save(f, quality=100)
        print(f"Processed {f}")
        
    except Exception as e:
        print(f"Error {f}: {e}")
