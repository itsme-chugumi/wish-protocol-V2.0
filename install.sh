#!/bin/bash
# Wish Protocol v2.0 - Automated Installation Script
# For AI agents and automated deployments

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "================================================"
echo "  Wish Protocol v2.0 - Installation"
echo "================================================"
echo ""

# Check if running as root (we need it for final installation)
if [ "$EUID" -eq 0 ]; then
    echo -e "${YELLOW}Warning: Running as root${NC}"
fi

# 1. Check prerequisites
echo "Checking prerequisites..."

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Cargo not found${NC}"
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}✓ Rust installed${NC}"
else
    echo -e "${GREEN}✓ Cargo found${NC}"
fi

# Check for git
if ! command -v git &> /dev/null; then
    echo -e "${RED}✗ Git not found${NC}"
    echo "Please install git first"
    exit 1
else
    echo -e "${GREEN}✓ Git found${NC}"
fi

# 2. Clone repository (if not already in it)
if [ ! -f "Cargo.toml" ]; then
    echo ""
    echo "Cloning repository..."
    git clone https://github.com/itsme-chugumi/wish-protocol-V2.0.git
    cd wish-protocol-V2.0
    echo -e "${GREEN}✓ Repository cloned${NC}"
else
    echo -e "${GREEN}✓ Already in repository${NC}"
fi

# 3. Build
echo ""
echo "Building wishp (this may take a few minutes)..."
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

# 4. Install binary
echo ""
echo "Installing wishp to /usr/local/bin/..."

if [ "$EUID" -eq 0 ]; then
    cp target/release/wishp /usr/local/bin/
else
    sudo cp target/release/wishp /usr/local/bin/
fi

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Binary installed${NC}"
else
    echo -e "${RED}✗ Installation failed${NC}"
    exit 1
fi

# 5. Create directories
echo ""
echo "Creating configuration directories..."
mkdir -p ~/.wish-protocol/keys

echo -e "${GREEN}✓ Directories created${NC}"

# 6. Verify installation
echo ""
echo "Verifying installation..."
if command -v wishp &> /dev/null; then
    echo -e "${GREEN}✓ wishp command available${NC}"
    WISHP_VERSION=$(wishp --version 2>&1 || echo "unknown")
    echo "  Version: $WISHP_VERSION"
else
    echo -e "${RED}✗ wishp command not found${NC}"
    echo "  You may need to add /usr/local/bin to your PATH"
fi

# 7. Next steps
echo ""
echo "================================================"
echo -e "${GREEN}Installation complete!${NC}"
echo "================================================"
echo ""
echo "Next steps:"
echo "1. Generate keys:        wishp keygen"
echo "2. Create config:        nano ~/.wish-protocol/config.toml"
echo "3. Create handler:       nano ~/.wish-protocol/handler"
echo "4. Start daemon:         wishp daemon"
echo ""
echo "For detailed instructions, see:"
echo "  README_FOR_AGENTS.md"
echo ""
echo "To uninstall:"
echo "  sudo rm /usr/local/bin/wishp"
echo "  rm -rf ~/.wish-protocol"
echo ""
