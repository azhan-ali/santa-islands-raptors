from PIL import Image
import os

files = [
    "sprites/thumb_mp_lvl1.jpg",
    "sprites/thumb_mp_lvl2.jpg",
    "sprites/thumb_mp_lvl3.jpg",
    "sprites/thumb_mp_lvl4.jpg",
    "sprites/thumb_mp_lvl5.jpg"
]

for f in files:
    try:
        # Open
        img = Image.open(f)
        
        # Resize to 230x230 as requested by USER
        img = img.resize((230, 230), Image.Resampling.LANCZOS)
        
        # Quantize to 64 colors to be very safe against the 65k global limit.
        # JPG compression will add some colors back, but starting from 64 should keep the total unique colors largely reduced compared to a full photo.
        img = img.quantize(colors=64) 
        
        # Convert to RGB for JPG save
        img = img.convert("RGB")
        
        # Save
        img.save(f, quality=100)
        print(f"Processed {f}")
        
    except Exception as e:
        print(f"Error {f}: {e}")
