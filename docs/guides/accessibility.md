# Accessibility Standards (WCAG AA)

AOF targets WCAG AA compliance across the web application to ensure usability for keyboard-only users, screen reader users, and mobile device users.

## Keyboard Navigation

### Skip-to-Content Link
- First focusable element on every page
- Press `Tab` on page load to reveal the skip link
- Activating it jumps focus to `<main id="main-content">`

### Tab Order
1. Skip-to-content link
2. Header (logo, connection status, theme toggle)
3. Navigation tabs (desktop) or bottom nav (mobile)
4. Page content (interactive elements in DOM order)

### Modal Focus Trapping
All modals use the `useFocusTrap` hook:
- Focus moves to the first focusable element on open
- `Tab` and `Shift+Tab` cycle within the modal
- `Escape` closes the modal
- Focus returns to the trigger element on close

### Kanban Board Keyboard Drag-and-Drop
dnd-kit provides native keyboard support:
- `Tab` to navigate between task cards
- `Space` or `Enter` to pick up a task
- Arrow keys to move between columns
- `Space` or `Enter` to drop
- `Escape` to cancel

Screen reader instructions are available via `aria-describedby`.

## Screen Reader Support

### ARIA Live Regions
- **Toast notifications**: `aria-live="polite"` region announces new toasts
- **Chat messages**: `aria-live="polite"` region in MessageFeed announces new messages
- **Connection status**: `role="status"` with `aria-live="polite"` on the connection indicator

### Modal Dialogs
- `role="dialog"` and `aria-modal="true"` on all modals
- `aria-labelledby` points to the modal title
- Close button has `aria-label="Close"`

### Navigation
- Desktop nav uses `role="tab"` and `aria-selected` for tab semantics
- Mobile nav uses `role="navigation"` and `aria-current="page"`
- Bottom navigation has `aria-label="Main navigation"`

### Lists and Logs
- Chat message feed uses `role="log"` and `aria-label="Chat messages"`
- Kanban columns use `role="list"` with `aria-label` including column name and count
- Task cards use `role="listitem"` with descriptive `aria-label`

## Color Contrast

AOF meets WCAG AA contrast ratios:

| Element | Ratio Target | Implementation |
|---------|-------------|----------------|
| Primary text (light) | >= 7:1 | `text-gray-900` on white |
| Primary text (dark) | >= 7:1 | `text-white` on `bg-gray-900` |
| Secondary text (light) | >= 4.5:1 | `text-gray-600` on white |
| Secondary text (dark) | >= 4.5:1 | `text-gray-300` on `bg-gray-900` |
| Interactive elements | >= 3:1 | `text-sky-600` on white |

The `text-secondary` utility class applies accessible secondary text colors automatically:
```css
.text-secondary {
  @apply text-gray-600 dark:text-gray-300;
}
```

## Touch Targets

All interactive elements have a minimum 44x44px touch target on mobile:
- Navigation buttons: `min-h-[44px] min-w-[44px]`
- Mobile bottom nav items: `min-h-[44px] min-w-[44px]`
- Modal close buttons: padded to meet minimum
- Send button in chat: sized to minimum target

## Reduced Motion

AOF respects `prefers-reduced-motion`:
```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

The `useReducedMotion` hook is available for component-level motion control.

## Testing Guide

### Automated Testing
- **eslint-plugin-jsx-a11y**: Catches common issues at development time (configured in `.eslintrc.json`)
- **Lighthouse**: Run Chrome DevTools > Lighthouse > Accessibility audit (target score: 90+)
- **axe DevTools**: Browser extension for in-page accessibility scanning

### Manual Testing

#### Keyboard Testing
1. Press `Tab` from page load -- skip link should appear
2. Navigate entire app without mouse
3. Open a modal -- focus should be trapped
4. Close modal -- focus should return to trigger

#### Screen Reader Testing (macOS VoiceOver)
1. Enable VoiceOver: `Cmd + F5`
2. Navigate with `VO + Right Arrow` through page elements
3. Verify all interactive elements have accessible names
4. Open SquadChat -- new messages should be announced
5. Trigger a toast -- notification should be announced

#### Mobile Testing
1. Resize to 375px width
2. Bottom navigation should appear
3. Desktop nav should be hidden
4. All touch targets should be >= 44px
5. SquadChat member toggle should work
6. WorkflowBuilder should show "Desktop Recommended" message

## Component Checklist

Every new component must have:
- [ ] `aria-label` or `aria-labelledby` on interactive elements
- [ ] Keyboard focusable (no `tabIndex > 0`, use natural tab order)
- [ ] Color contrast passing 4.5:1 for text, 3:1 for large text
- [ ] Touch target >= 44x44px on mobile
- [ ] No information conveyed by color alone
- [ ] Meaningful alt text for images (if any)
- [ ] No `eslint-plugin-jsx-a11y` violations
