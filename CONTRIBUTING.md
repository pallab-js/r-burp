# Contributing to r-burp

Thank you for your interest in contributing to r-burp! This document provides guidelines for contributing to the project.

## Code of Conduct

This project is committed to providing a welcoming and inclusive environment for all contributors.

## Getting Started

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Write or update tests as needed
5. Ensure all tests pass (`cargo test` for Rust, `npm test` for frontend)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## Development Workflow

### Backend (Rust)

```bash
cd src-tauri
cargo check      # Quick compile check
cargo test       # Run unit tests
cargo clippy     # Lint with clippy
```

### Frontend (Next.js)

```bash
cd src
npm run dev      # Start development server
npm run build    # Build for production
npm run lint     # Run ESLint
```

### Full Application

```bash
cargo tauri dev   # Run full Tauri app in dev mode
cargo tauri build # Build for production
```

## Code Style

### Rust

- Follow idiomatic Rust conventions
- Use `cargo clippy` for linting
- All public APIs must have documentation
- Input validation is mandatory for all Tauri commands

### TypeScript/JavaScript

- Use TypeScript for all new code
- Follow the existing ESLint configuration
- Components should be functional with clear prop types
- Follow the design system in `DESIGN.md` for UI changes

### CSS/Tailwind

- Use Tailwind CSS utility classes wherever possible
- Reference design tokens from `globals.css` for custom values
- Follow the warm minimalism design system strictly

## Pull Request Process

1. Update the README.md with details of changes if applicable
2. Update the CHANGELOG.md with a description of your changes
3. The PR should work on all target platforms (macOS, Windows, Linux)
4. All tests must pass
5. Two approvals from maintainers are required for merging

## Reporting Bugs

When reporting bugs, include:

- Clear description of the issue
- Steps to reproduce
- Expected behavior
- Actual behavior
- Screenshots if applicable
- Platform and version information

## Feature Requests

Feature requests are welcome. Please provide:

- Clear description of the feature
- Use case or problem it solves
- Any relevant design considerations

## License

By contributing to r-burp, you agree that your contributions will be licensed under the MIT License.
