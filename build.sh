#!/bin/bash
# Build script for Toon Dash - WASM for Samsung Tizen TV

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$PROJECT_DIR/build"
DIST_DIR="$PROJECT_DIR/dist"
WEB_DIR="$PROJECT_DIR/web"

echo -e "${GREEN}=== Toon Dash Build Script ===${NC}"

check_requirements() {
    echo -e "${YELLOW}Checking requirements...${NC}"

    if ! command -v rustc &> /dev/null; then
        echo "Error: Rust not installed. Visit https://rustup.rs"
        exit 1
    fi

    if ! command -v wasm-pack &> /dev/null; then
        echo "Installing wasm-pack..."
        cargo install wasm-pack
    fi

    rustup target add wasm32-unknown-unknown 2>/dev/null || true
    echo -e "${GREEN}Requirements OK${NC}"
}

build_wasm() {
    echo -e "${YELLOW}Building WASM...${NC}"
    mkdir -p "$BUILD_DIR"

    wasm-pack build \
        --release \
        --target web \
        --out-name toon_dash \
        --out-dir "$BUILD_DIR"

    echo -e "${GREEN}WASM build complete${NC}"
}

package_dist() {
    echo -e "${YELLOW}Packaging...${NC}"
    mkdir -p "$DIST_DIR"

    cp "$BUILD_DIR/toon_dash.js" "$DIST_DIR/"
    cp "$BUILD_DIR/toon_dash_bg.wasm" "$DIST_DIR/"
    cp "$WEB_DIR"/*.html "$DIST_DIR/"
    cp "$WEB_DIR"/*.js "$DIST_DIR/"

    if command -v wasm-opt &> /dev/null; then
        wasm-opt -Oz -o "$DIST_DIR/toon_dash_bg.wasm" "$DIST_DIR/toon_dash_bg.wasm"
    fi

    echo ""
    echo -e "${GREEN}Distribution files:${NC}"
    ls -lh "$DIST_DIR"
}

serve() {
    echo -e "${YELLOW}Starting server at http://localhost:8080${NC}"
    cd "$DIST_DIR"
    python3 -m http.server 8080
}

clean() {
    echo -e "${YELLOW}Cleaning...${NC}"
    rm -rf "$BUILD_DIR" "$DIST_DIR" target/wasm32-unknown-unknown
    echo -e "${GREEN}Clean complete${NC}"
}

case "${1:-all}" in
    check) check_requirements ;;
    wasm)  check_requirements; build_wasm ;;
    package) check_requirements; build_wasm; package_dist ;;
    serve)  serve ;;
    all)    check_requirements; build_wasm; package_dist ;;
    clean)  clean ;;
    *)
        echo "Usage: $0 {check|wasm|package|serve|all|clean}"
        exit 1
        ;;
esac