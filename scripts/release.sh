#!/bin/bash
# Release automation script for Komando
# Handles version bumping and release process

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
if [ ! -f "Cargo.toml" ]; then
    error "Must be run from repository root"
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    error "Uncommitted changes detected. Please commit or stash them first."
fi

# Show usage if no arguments
if [ $# -eq 0 ]; then
    echo "Usage: $0 <release-type>"
    echo ""
    echo "Release types:"
    echo "  alpha     - Bump alpha version (e.g., 1.0.0-alpha.1 -> 1.0.0-alpha.2)"
    echo "  patch     - Bump patch version (e.g., 1.0.0 -> 1.0.1)"
    echo "  minor     - Bump minor version (e.g., 1.0.0 -> 1.1.0)"
    echo "  major     - Bump major version (e.g., 1.0.0 -> 2.0.0)"
    echo ""
    echo "Examples:"
    echo "  $0 alpha   # Release new alpha version"
    echo "  $0 patch   # Patch release"
    exit 0
fi

RELEASE_TYPE=$1

# Get current version
CURRENT_VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
info "Current version: $CURRENT_VERSION"

# Calculate new version based on release type
case "$RELEASE_TYPE" in
    alpha)
        if [[ $CURRENT_VERSION =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)-alpha\.([0-9]+)$ ]]; then
            # Increment alpha number
            NEW_VERSION="${BASH_REMATCH[1]}.${BASH_REMATCH[2]}.${BASH_REMATCH[3]}-alpha.$((${BASH_REMATCH[4]} + 1))"
        elif [[ $CURRENT_VERSION =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
            # First alpha release
            NEW_VERSION="${BASH_REMATCH[1]}.${BASH_REMATCH[2]}.$((${BASH_REMATCH[3]} + 1))-alpha.1"
        else
            error "Unable to parse version for alpha bump"
        fi
        ;;
    patch)
        if [[ $CURRENT_VERSION =~ ^([0-9]+)\.([0-9]+)\.([0-9]+) ]]; then
            NEW_VERSION="${BASH_REMATCH[1]}.${BASH_REMATCH[2]}.$((${BASH_REMATCH[3]} + 1))"
        else
            error "Unable to parse version for patch bump"
        fi
        ;;
    minor)
        if [[ $CURRENT_VERSION =~ ^([0-9]+)\.([0-9]+)\. ]]; then
            NEW_VERSION="${BASH_REMATCH[1]}.$((${BASH_REMATCH[2]} + 1)).0"
        else
            error "Unable to parse version for minor bump"
        fi
        ;;
    major)
        if [[ $CURRENT_VERSION =~ ^([0-9]+)\. ]]; then
            NEW_VERSION="$((${BASH_REMATCH[1]} + 1)).0.0"
        else
            error "Unable to parse version for major bump"
        fi
        ;;
    *)
        error "Unknown release type: $RELEASE_TYPE"
        ;;
esac

info "New version: $NEW_VERSION"
TAG="v${NEW_VERSION}"

# Get current branch
CURRENT_BRANCH=$(git branch --show-current)
info "Current branch: $CURRENT_BRANCH"

# Update version in Cargo.toml
info "Updating Cargo.toml..."
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml

# Update Cargo.lock
info "Updating Cargo.lock..."
cargo update -p komando --quiet

# Run tests
info "Running tests..."
cargo test -- --test-threads=1 --quiet || error "Tests failed"
success "All tests passed"

# Create release commit
info "Creating release commit..."
git add Cargo.toml Cargo.lock
git commit --no-verify -m "chore: release komando v${NEW_VERSION}" || error "Failed to create commit"
success "Release commit created"

# Create tag
info "Creating tag: $TAG"
git tag "$TAG" || error "Failed to create tag"
success "Tag created: $TAG"

# Create release branch
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
echo ""
info "Create PR here:"
echo "  https://github.com/Vellyxenya/Komando/compare/master...${RELEASE_BRANCH}"
echo ""
info "You can return to your original branch with:"
echo "  git checkout $CURRENT_BRANCH"
