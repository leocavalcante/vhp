# VHP Theme

A modern, minimalist, and elegant GitHub Pages theme. Clean typography, sophisticated color palette, and smooth interactions.

## Features

- **Modern Design** — Elegant visual hierarchy with sophisticated color palette
- **Minimalist** — Clean, spacious layouts without unnecessary decoration
- **Fast** — Pure CSS, minimal JavaScript, system fonts
- **Responsive** — Mobile-first design that works everywhere
- **Smooth** — Gentle transitions and hover effects for polish
- **Accessible** — Semantic HTML, WCAG AA+ contrast, keyboard navigation

## Color Palette

| Purpose | Color | Hex |
|---------|-------|-----|
| Dark | #1a202c | Used for headings and text |
| Slate | #2d3748 | Secondary dark shade |
| Teal | #06b6d4 | Primary accent (tech feel) |
| Teal Dark | #0891b2 | Hover state |
| Amber | #f59e0b | Warm accent (highlights) |
| Text | #1f2937 | Body text |
| Gray | #e5e7eb | Borders and dividers |

## Typography

- **System Font Stack** — `-apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto"...`
- **Monospace** — `"Fira Code", "Courier New", monospace`
- **Modern Sizing** — Responsive font scales (3rem → 1.75rem on mobile)
- **Spacing** — 7-level spacing system (0.25rem to 4rem)

## Design Highlights

### Gradients & Depth
- Hero heading uses cyan-to-dark gradient
- Site title uses cyan-to-amber gradient
- Code blocks have dark gradient backgrounds
- Table headers use dark gradient

### Subtle Interactions
- Links have animated underline on hover
- Buttons have lift effect with shadows
- Feature cards move up on hover with glow
- Navigation items highlight with background

### Components
- **Navigation** — Sticky header with backdrop blur
- **Hero Section** — Large, gradient text with prominent links
- **Feature Cards** — Grid layout with top gradient border
- **Code Blocks** — Dark gradient with left accent border
- **Tables** — Hover effects with smooth transitions
- **Footer** — Dark gradient background with warm accent links

## Responsive Breakpoints

- **Desktop**: 769px and up
- **Tablet**: 481px to 768px
- **Mobile**: 480px and below

## Customization

### Change Colors
Edit CSS variables in `style.css`:

```css
:root {
  --color-teal: #06b6d4;        /* Primary accent */
  --color-accent: #f59e0b;      /* Warm highlight */
  --color-dark: #1a202c;        /* Headings */
}
```

### Adjust Typography
Modify fonts in CSS variables:

```css
--font-base: your-font-stack;
--font-mono: your-mono-font;
```

### Change Spacing
Update the spacing scale:

```css
--space-md: 1rem;    /* Base spacing */
--space-lg: 1.5rem;  /* Larger */
```

## Layout Templates

- **default.html** — Standard content page with sidebar support
- **home.html** — Hero-focused landing page
- **page.html** — Simple article layout

## Browser Support

- Chrome/Edge (latest)
- Firefox (latest)
- Safari (latest)
- Mobile browsers
- IE 11 (graceful degradation)

## File Structure

```
_vhp_theme/
├── _layouts/
│   ├── default.html
│   ├── home.html
│   └── page.html
├── assets/css/
│   └── style.css      (~700 lines)
├── README.md
├── theme.gemspec
└── _config.example.yml
```

## Philosophy

The VHP theme embraces:
1. **Simplicity** — No unnecessary elements or bloat
2. **Elegance** — Tasteful colors and smooth interactions
3. **Readability** — Clean typography and good contrast
4. **Performance** — Fast load times, no external dependencies
5. **Accessibility** — Inclusive design for all users

## License

This theme is part of the VHP project and follows the same license.

## Credits

**Theme**: Modern, minimalist design with elegant touches
**Colors**: Carefully chosen for tech projects (teal/cyan + warm amber)
**Typography**: System fonts optimized for every platform
**Interactions**: Smooth transitions and subtle hover effects
