# VHP GitHub Pages Theme

The VHP project includes a custom GitHub Pages theme inspired by the **PHP 8.5 landing page design**. This theme is located in the `_vhp_theme` directory.

## What's Included

### 📁 Theme Structure

```
_vhp_theme/
├── _layouts/                 # Jekyll layout templates
│   ├── default.html         # Standard page layout
│   ├── home.html           # Hero-focused homepage layout
│   └── page.html           # Article layout
├── assets/
│   └── css/
│       └── style.css       # Main stylesheet (650+ lines)
├── README.md               # Theme documentation
├── theme.gemspec           # Gem specification
└── _config.example.yml     # Example configuration
```

### 🎨 Design Features

#### Color Palette
- **Primary**: #777BB3 (Purple) — Main branding color
- **Secondary**: #1F5F83 (Blue) — Links and accents
- **Accent**: #F37830 (Orange) — Call-to-action highlights
- **Text**: #1A1A1A (Dark gray) — High contrast for readability
- **Background**: #FFFFFF (White) — Clean, minimal design

#### Typography
- **Body Font**: System font stack (optimized for each OS)
  - macOS: San Francisco
  - Windows: Segoe UI
  - Linux: Roboto, Helvetica Neue
- **Monospace**: SFMono-Regular, Consolas, Liberation Mono
- **Font Sizes**: Responsive and accessible

#### Layout
- **Mobile-first** design with breakpoints at 768px and 480px
- **Max-width**: 48rem (768px) for content, 60rem (960px) for wide layouts
- **Consistent spacing** using CSS custom properties

### ✨ Key Features

1. **Lightweight & Fast**
   - Pure CSS (no heavy frameworks)
   - Minimal JavaScript (mobile nav only)
   - System fonts (no web font downloads)
   - ~650 lines of CSS

2. **Responsive**
   - Mobile-first design
   - Works on all screen sizes
   - Hamburger menu for mobile
   - Touch-friendly buttons and links

3. **Accessible**
   - Semantic HTML5
   - ARIA labels for navigation
   - Good contrast ratios (WCAG AA+)
   - Keyboard navigation support

4. **Developer-Friendly**
   - CSS custom properties for easy customization
   - Well-documented code
   - Example configuration file
   - Easy to extend

### 📱 Responsive Breakpoints

```css
/* Desktop: 769px and up */
@media (max-width: 768px) { /* Tablet: 481px to 768px */ }
@media (max-width: 480px) { /* Mobile: 480px and below */ }
```

### 🧩 Provided Components

The theme CSS includes styles for:

- **Navigation**: Sticky header with responsive menu
- **Buttons**: Primary, secondary, and small variants
- **Cards**: Feature cards with `feature-card` class
- **Grids**: Auto-fit responsive grid layout
- **Code Blocks**: Syntax-highlighted code with proper styling
- **Tables**: Responsive table layouts
- **Forms**: Input styling (ready for enhancement)
- **Utilities**: Margin, padding, text utilities

### 🎯 Layout Templates

#### `default.html`
Used for standard content pages. Includes:
- Header with navigation
- Main content area
- Footer
- Mobile menu toggle

#### `home.html`
Specialized for homepage. Features:
- Same header and navigation
- `.hero` class for hero section styling
- Perfect for landing page design

#### `page.html`
Simple article layout extending default layout.

## Configuration

The theme is configured in `docs/_config.yml`:

```yaml
title: VHP
description: Vibe-coded Hypertext Preprocessor...
theme: _vhp_theme

nav:
  - title: Home
    url: /
  - title: Features
    url: /features
  # ... more items
```

## Customization

### Change Colors
Edit the CSS variables in `_vhp_theme/assets/css/style.css`:

```css
:root {
  --color-primary: #777bb3;      /* Change primary color */
  --color-primary-dark: #665ba0;  /* Hover state */
  --color-secondary: #1f5f83;     /* Links */
  --color-accent: #f37830;        /* CTAs */
  /* ... more colors ... */
}
```

### Adjust Typography
Modify font stacks and sizes in CSS variables:

```css
--font-base: /* your font stack */;
--font-mono: /* your monospace font */;
```

### Change Spacing
Update the spacing scale:

```css
--spacing-md: 1rem;    /* Base spacing */
--spacing-lg: 1.5rem;  /* Larger spacing */
/* ... adjust as needed ... */
```

### Extend CSS
Add custom styles to a new file or edit `style.css` directly. The theme uses standard CSS with no preprocessor, so changes are straightforward.

## Philosophy

The VHP theme follows the **PHP 8.5 landing page design philosophy**:

1. **Documentation First** — Visual hierarchy focused on content readability
2. **Minimal Decoration** — Tasteful, unobtrusive visual design
3. **Performance** — System fonts, no bloat, fast page loads
4. **Accessibility** — Semantic HTML, good contrast, keyboard navigation
5. **Simplicity** — Clean code, easy to understand and modify

## Design Inspiration

This theme is inspired by:
- [PHP 8.5 Release Page](https://www.php.net/releases/8.5/en.php) — Clean, modern design
- PHP.net Official Design — Professional, trustworthy aesthetic
- Modern Web Standards — CSS Grid, Flexbox, CSS Variables

## Browser Support

- Chrome/Edge (latest)
- Firefox (latest)
- Safari (latest)
- Mobile browsers (iOS Safari, Chrome Mobile)
- IE 11 (with graceful degradation)

## Files Modified

The following files were created/modified to use the theme:

- `_vhp_theme/` — New custom theme directory
- `docs/_config.yml` — Updated to use local theme
- `docs/index.md` — Updated to use `home` layout

## Future Enhancements

Possible improvements:
- Add dark mode support (CSS variables ready)
- Add search functionality
- Add sidebar navigation for docs
- Add syntax highlighting for code blocks
- Add print stylesheet optimization

## Credits

**Theme Creator**: Claude (AI Agent)
**Inspiration**: PHP 8.5 Landing Page Design Contest
**Based on**: Web Performance Best Practices, Accessibility Standards (WCAG)

## License

This theme is part of the VHP project and follows the same license.
