from PIL import Image
import os

# Files to process
# For Factory and Sleigh, we use the _temp.png we just copied.
# For Breaker and Stealth, we re-process the existing JPEGs to ensure 230x230 size.

tasks = [
    {"src": "sprites/thumb_factory_temp.png", "dest": "sprites/thumb_factory.jpg", "new": True},
    {"src": "sprites/thumb_sleigh_temp.png", "dest": "sprites/thumb_sleigh.jpg", "new": True},
    {"src": "sprites/thumb_breaker.jpg", "dest": "sprites/thumb_breaker.jpg", "new": False},
    {"src": "sprites/thumb_stealth.jpg", "dest": "sprites/thumb_stealth.jpg", "new": False}
]

for task in tasks:
    src = task["src"]
    dest = task["dest"]
    
    if task["new"] and not os.path.exists(src):
        print(f"Skipping {src} (Not found)")
        continue
    
    if not task["new"] and not os.path.exists(src):
        print(f"Skipping {src} (Existing file not found)")
        continue
        
    try:
        img = Image.open(src)
        
        # Resize to 230x230
        img = img.resize((230, 230), Image.Resampling.LANCZOS)
        
        # Quantize to 64 colors
        img = img.quantize(colors=64)
        
        img = img.convert("RGB")
        
        img.save(dest, quality=100)
        print(f"Processed {dest} from {src}")
        
        # Cleanup temp
        if task["new"]:
            os.remove(src)
            
    except Exception as e:
        print(f"Error processing {src} -> {dest}: {e}")
