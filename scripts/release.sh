#!/bin/bash
# Release automation script for Komando
# Handles cargo-release with branch protection rules

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
info() { echo -e "${BLUE}ℹ${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warning() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

# Check if we're in the repo root
if [ ! -f "Cargo.toml" ] || [ ! -f "release.toml" ]; then
    error "Must be run from repository root"
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    error "Uncommitted changes detected. Please commit or stash them first."
fi

# Check if cargo-release is installed
if ! command -v cargo-release &> /dev/null; then
    error "cargo-release not found. Install with: cargo install cargo-release"
fi

# Show usage if no arguments
if [ $# -eq 0 ]; then
    echo "Usage: $0 <release-type> [options]"
    echo ""
    echo "Release types:"
    echo "  alpha     - Bump alpha version (e.g., 1.0.0-alpha.1 -> 1.0.0-alpha.2)"
    echo "  beta      - Bump to beta version (e.g., 1.0.0-alpha.1 -> 1.0.0-beta.1)"
    echo "  rc        - Bump to release candidate (e.g., 1.0.0-beta.1 -> 1.0.0-rc.1)"
    echo "  release   - Remove pre-release (e.g., 1.0.0-rc.1 -> 1.0.0)"
    echo "  patch     - Bump patch version (e.g., 1.0.0 -> 1.0.1)"
    echo "  minor     - Bump minor version (e.g., 1.0.0 -> 1.1.0)"
    echo "  major     - Bump major version (e.g., 1.0.0 -> 2.0.0)"
    echo ""
    echo "Options:"
    echo "  --dry-run    Run in dry-run mode (preview changes)"
    echo ""
    echo "Examples:"
    echo "  $0 alpha           # Release alpha version"
    echo "  $0 patch --dry-run # Preview patch release"
    exit 0
fi

RELEASE_TYPE=$1
DRY_RUN=""

# Check for --dry-run flag
if [ "$2" = "--dry-run" ]; then
    DRY_RUN="yes"
    warning "Running in DRY-RUN mode"
fi

# Get current branch
CURRENT_BRANCH=$(git branch --show-current)
info "Current branch: $CURRENT_BRANCH"

# Run cargo release (with or without -x flag)
if [ -n "$DRY_RUN" ]; then
    info "Running cargo release $RELEASE_TYPE (dry-run)..."
    cargo release "$RELEASE_TYPE"
    success "Dry-run completed successfully"
    echo ""
    info "To execute the release, run: $0 $RELEASE_TYPE"
    exit 0
else
    info "Running cargo release $RELEASE_TYPE..."
    echo ""
    info "This will:"
    echo "  1. Run all tests (--test-threads=1)"
    echo "  2. Update Cargo.toml version"
    echo "  3. Update CHANGELOG.md"
    echo "  4. Create a git commit"
    echo "  5. Create a git tag"
    echo ""
    cargo release "$RELEASE_TYPE" -x --no-confirm --verbose || error "cargo release failed"
    success "cargo-release completed"
fi

# Get the new version and tag
NEW_VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
TAG="v${NEW_VERSION}"

info "New version: $NEW_VERSION"
info "Tag: $TAG"

# Create release branch name
RELEASE_BRANCH="release/${TAG}"

info "Creating release branch: $RELEASE_BRANCH"
git branch "$RELEASE_BRANCH" || error "Failed to create branch"

# Push the release branch
info "Pushing release branch to origin..."
git push origin "$RELEASE_BRANCH" || error "Failed to push release branch"
success "Release branch pushed: $RELEASE_BRANCH"

# Push the tag
info "Pushing tag to origin..."
git push origin "$TAG" || error "Failed to push tag"
success "Tag pushed: $TAG"

# Print next steps
echo ""
success "Release process completed!"
echo ""
info "Next steps:"
echo "  1. Create a PR: ${RELEASE_BRANCH} -> master"
echo "  2. Review and merge the PR"
echo "  3. GitHub Actions will automatically:"
echo "     - Build binaries for all platforms"
echo "     - Create GitHub Release"
echo "     - Publish to crates.io"
echo ""
info "Create PR here:"
echo "  https://github.com/Vellyxenya/Komando/compare/master...${RELEASE_BRANCH}"
echo ""
info "You can return to your original branch with:"
echo "  git checkout $CURRENT_BRANCH"
