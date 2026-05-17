#!/usr/bin/env python3
import re
import os
import subprocess

sdk_path = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-path']).decode().strip()
headers_dir = f"{sdk_path}/System/Library/Frameworks/AVFoundation.framework/Headers"

symbols = {}

# Find all AVCapture headers
for filename in sorted(os.listdir(headers_dir)):
    if not filename.startswith('AVCapture') and filename != 'AVFCapture.h':
        continue
    
    filepath = os.path.join(headers_dir, filename)
    with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
        content = f.read()
    
    # Extract @interface lines with their availability attributes
    for match in re.finditer(r'(API_[A-Z_]+.*?\n)?@interface\s+(\w+)', content):
        avail = match.group(1) or ""
        name = match.group(2)
        if name not in symbols:
            symbols[name] = {'kind': 'interface', 'header': filename, 'avail': avail.strip()}
    
    # Extract @protocol lines with their availability attributes
    for match in re.finditer(r'(API_[A-Z_]+.*?\n)?@protocol\s+(\w+)', content):
        avail = match.group(1) or ""
        name = match.group(2)
        if name not in symbols:
            symbols[name] = {'kind': 'protocol', 'header': filename, 'avail': avail.strip()}

# Print found symbols
for name in sorted(symbols.keys()):
    info = symbols[name]
    avail_str = info['avail'][:60] if info['avail'] else "NO_AVAILABILITY"
    print(f"{name:50} | {info['kind']:15} | {info['header']:35} | {avail_str}")

print(f"\n\nTotal macOS-available symbols found: {len([s for s in symbols.values() if 'API_UNAVAILABLE(macos)' not in s['avail']])}")
print(f"Total API_UNAVAILABLE(macos) symbols: {len([s for s in symbols.values() if 'API_UNAVAILABLE(macos)' in s['avail']])}")
