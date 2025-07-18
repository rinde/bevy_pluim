

# Todo
- [v] ScrollBar
- [v] List as 2D grid + List horizontal
- [v] Custom item bundle (color square)
- [v] Theming of custom items
- [v] Color palette (with scrollbars)
- [v] Color palette combo
- [v] Tabs
- [v] TabPane (stack?)
- [ ] Get rid of WidgetId?
- [ ] Card -> just ads a bit of style to a node
- [ ] Label with different text sizes (Header, Normal)

# Future
- [ ] Toggle with images (e.g. for theme switcher)
- [ ] Detect dark/light mode (change)
- [ ] Theme editor
- [ ] Tooltips
- [ ] ClearColor: auto update ClearColor on Theme change
- [ ] Customizable List bundle to show tooltip (needed or not?)
- [ ] List: allow optionally setting maximum height
- [ ] Combo: allow specifying placeholder text
- [ ] Combo: ensure List is visible within window
- [ ] ScrollArea: allow dragging thumb
- [ ] Give Floats their own window so they can get out of the main window


# Open questions
- How to ensure that an item is fully within the window? How to obtain the actual coordinates of the node? (`ComputedNode` only contains the size, not location). TODO try to get the GlobalTransform of the node. transform.compute_transform()

# Blocked
- Scrolling in a container with rounded corners doesn't clip the rounded corners
