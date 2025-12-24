from PIL import Image
import os
import shutil

# Sources:
# For Factory/Sleigh: use _temp.png (high qual source available)
# For Breaker/Stealth: use .jpg (must convert)
# For MP 1-5: use .jpg (must convert)

# Destinations: .png (230x230, indexed)

tasks = [
    {"src": "sprites/thumb_factory_temp.png", "dest": "sprites/thumb_factory.png", "clean": True},
    {"src": "sprites/thumb_sleigh_temp.png", "dest": "sprites/thumb_sleigh.png", "clean": True},
    {"src": "sprites/thumb_breaker.jpg", "dest": "sprites/thumb_breaker.png", "clean": True},
    {"src": "sprites/thumb_stealth.jpg", "dest": "sprites/thumb_stealth.png", "clean": True},
    {"src": "sprites/thumb_mp_lvl1.jpg", "dest": "sprites/thumb_mp_lvl1.png", "clean": True},
    {"src": "sprites/thumb_mp_lvl2.jpg", "dest": "sprites/thumb_mp_lvl2.png", "clean": True},
    {"src": "sprites/thumb_mp_lvl3.jpg", "dest": "sprites/thumb_mp_lvl3.png", "clean": True},
    {"src": "sprites/thumb_mp_lvl4.jpg", "dest": "sprites/thumb_mp_lvl4.png", "clean": True},
    {"src": "sprites/thumb_mp_lvl5.jpg", "dest": "sprites/thumb_mp_lvl5.png", "clean": True},
]

for task in tasks:
    src = task["src"]
    dest = task["dest"]
    
    if not os.path.exists(src):
        # Fallback: maybe .png already exists from previous runs or I messed up filenames?
        # Check if dest exists to skip? No, we must ensure 230x230 and quantize.
        
        # Special check for Factory/Sleigh, if temp missing, check jpg?
        fallback = src.replace("_temp.png", ".jpg")
        if os.path.exists(fallback):
            src = fallback
        else:
            print(f"Skipping {src} (Source not found)")
            continue
            
    try:
        img = Image.open(src)
        
        # Resize to 230x230
        img = img.resize((230, 230), Image.Resampling.LANCZOS)
        
        # Quantize to 64 colors PALETTE mode.
        # This reduces unique colors drastically and saves as 8-bit Png.
        # IMPORTANT: Do NOT convert back to RGB. Stay in P mode.
        img = img.quantize(colors=64)
        
        # Save as PNG
        img.save(dest)
        print(f"Processed {dest} from {src}")
        
        # Cleanup source if it's different and requested
        if task["clean"] and src != dest and os.path.exists(src):
            # Be careful not to delete something we need later?
            # We are processing sequentially.
            os.remove(src)
            pass
            
    except Exception as e:
        print(f"Error processing {src} -> {dest}: {e}")
