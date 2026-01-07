#!/bin/bash
echo "Building WASM package..."
wasm-pack build --target web --out-dir web/pkg

if [ $? -eq 0 ]; then
    echo ""
    echo "Build successful! WASM package created in web/pkg/"
    echo ""
    echo "To test locally, serve the web/ directory with a web server:"
    echo "  python3 -m http.server 8000"
    echo "  or"
    echo "  npx serve web"
    echo ""
    echo "Then open http://localhost:8000 in your browser"
else
    echo ""
    echo "Build failed! Make sure wasm-pack is installed:"
    echo "  cargo install wasm-pack"
fi








