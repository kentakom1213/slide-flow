#!/bin/bash

# Get current branch name
current_branch=$(git rev-parse --abbrev-ref HEAD)

# Execute only on the "main" branch
if [ "$current_branch" = "main" ]; then
    echo "Building for main branch..."

    # Build in release mode
    cargo build -r

    # Install the binary
    cp target/release/slide-flow $HOME/.local/bin

    echo "Done."
else
    echo "Not on main branch, skipping build."
fi
