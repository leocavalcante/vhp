# VHP Theme

A clean, modern GitHub Pages theme inspired by the PHP 8.5 landing page design. This theme prioritizes simplicity, performance, and accessibility.

## Features

- **Lightweight** — Pure CSS, minimal JavaScript (only for mobile nav)
- **Responsive** — Mobile-first design that works on all screen sizes
- **Fast** — System fonts, no external dependencies, optimized for performance
- **Accessible** — Semantic HTML, ARIA labels, good contrast ratios
- **Modern** — CSS custom properties (variables), CSS Grid/Flexbox
- **PHP-inspired Colors** — Based on official PHP website branding

## Design Principles

The theme follows the PHP 8.5 landing page design philosophy:

1. **Documentation First** — Clean visual hierarchy focused on content
2. **Minimal Decoration** — Unobtrusive, tasteful visual design
3. **System Fonts** — Using device native fonts for better performance
4. **Zero JavaScript** — Except minimal mobile navigation (fully functional without)
5. **Responsive by Default** — Mobile-first approach with proper breakpoints

## Color Scheme

| Purpose | Color | CSS Variable |
|---------|-------|--------------|
| Primary | #777BB3 (Purple) | `--color-primary` |
| Secondary | #1F5F83 (Blue) | `--color-secondary` |
| Accent | #F37830 (Orange) | `--color-accent` |
| Text | #1A1A1A (Dark) | `--color-text` |
| Background | #FFFFFF | `--color-bg` |

## Typography

- **Body Font** — System stack: `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif`
- **Mono Font** — `SFMono-Regular, Consolas, "Liberation Mono", Menlo, monospace`

## Layout

The theme provides sensible max-widths:

- **Default content** — 48rem (768px)
- **Wide content** — 60rem (960px)

## Customization

### Color Variables

Edit `assets/css/style.css` to customize colors:

```css
:root {
  --color-primary: #777bb3;
  --color-primary-dark: #665ba0;
  --color-secondary: #1f5f83;
  /* ... more variables ... */
}
```

### Spacing

Adjust the spacing scale:

```css
:root {
  --spacing-xs: 0.25rem;
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
  /* ... more spacing ... */
}
```

### Fonts

Change fonts in the CSS variables:

```css
:root {
  --font-base: /* your font stack */;
  --font-mono: /* your mono font stack */;
}
```

## Configuration

The theme respects the following `_config.yml` settings:

```yaml
title: Site Title
description: Site description
theme: _vhp_theme

# Navigation
nav:
  - title: Page Title
    url: /page-url
```

## Responsive Breakpoints

- **Desktop** — 769px and up
- **Tablet** — 481px to 768px
- **Mobile** — 480px and down

## Browser Support

- Modern browsers (Chrome, Firefox, Safari, Edge)
- IE 11 may require polyfills for CSS Grid
- Graceful degradation for older browsers

## Usage

1. Copy the `_vhp_theme` directory to your Jekyll site root
2. Update `_config.yml`:
   ```yaml
   theme: _vhp_theme
   ```
3. Customize navigation in `_config.yml`
4. Create content pages with appropriate layouts

### Available Layouts

- **default** — Standard page layout with navigation
- **home** — Hero-focused layout for homepage
- **page** — Article layout with optional title

## License

MIT License — Feel free to use this theme for your projects!

## Credits

Inspired by:
- [PHP 8.5 Release Page Design](https://www.php.net/releases/8.5/en.php)
- Modern design best practices
- Web performance principles

## Contributing

Improvements and suggestions are welcome! Please open an issue or submit a pull request.
