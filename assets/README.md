# TurboGitHub Assets Directory

This directory contains all the static assets used by the TurboGitHub application.

## Directory Structure

```
assets/
├── icons/           # Application icons and logos
│   ├── logo.png     # PNG format application logo
│   └── logo.svg     # SVG format application logo
├── fonts/           # Font files (if needed)
├── images/          # Other image assets
└── configs/         # Configuration files and templates
```

## Usage Guidelines

1. **Icons**: Store all application icons and logos in the `icons/` directory
2. **Fonts**: Place any custom font files in the `fonts/` directory
3. **Images**: Store other image assets (screenshots, UI elements, etc.) in `images/`
4. **Configs**: Keep configuration templates and static configs in `configs/`

## File Naming Convention

- Use lowercase with hyphens for file names (e.g., `app-icon.png`)
- Include size information for icons (e.g., `icon-16x16.png`, `icon-32x32.png`)
- Use descriptive names for images and configs

## Adding New Assets

When adding new assets to the project:
1. Place them in the appropriate subdirectory
2. Update this README if adding new categories
3. Ensure file sizes are optimized for application performance
4. Consider including multiple formats for compatibility (e.g., PNG + SVG)