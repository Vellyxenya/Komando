# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release with SQLite-based command storage
- Optional semantic search using vector embeddings
- Interactive command selection with arrow keys
- Auto-migration from legacy JSON storage
- Shell integration for bash and zsh
- Automatic ONNX Runtime setup for embeddings feature

### Changed
- Migrated from JSON to SQLite storage
- Command history capture using `fc -ln` for better reliability

### Fixed
- Test isolation issues with shared temp files
- GLIBC compatibility by using ONNX Runtime 1.23.2 with load-dynamic feature
