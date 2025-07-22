

# Todo
- [v] ScrollBar
- [v] List as 2D grid + List horizontal
- [v] Custom item bundle (color square)
- [v] Theming of custom items
- [v] Color palette (with scrollbars)
- [v] Color palette combo
- [v] Tabs
- [v] TabPane (stack?)
- [v] Get rid of WidgetId?
- [v] Label with different text sizes (Header, Normal)
- [v] allow setting default selection (e.g. for Stack)
- [v] Card -> just adds a bit of style to a node

# Future
- [ ] slider
- [ ] Toggle/switch with images (e.g. for theme switcher)
- [ ] Triple switch (Light, System, Dark)
- [ ] Detect dark/light mode (change)
- [ ] Theme editor
- [ ] Tooltips
- [ ] ClearColor: auto update ClearColor on Theme change
- [ ] Customizable List bundle to show tooltip (needed or not?)
- [ ] List: allow optionally setting maximum height
- [ ] Combo: allow specifying placeholder text
- [ ] Combo: ensure List is visible within window
- [ ] ScrollArea: allow dragging thumb
- [ ] ScrollArea: don't reserve space for scrollbars when not needed
- [ ] Give Floats their own window so they can get out of the main window



# Open questions
- How to ensure that an item is fully within the window? How to obtain the actual coordinates of the node? (`ComputedNode` only contains the size, not location). TODO try to get the GlobalTransform of the node. transform.compute_transform()

# Blocked
- Scrolling in a container with rounded corners doesn't clip the rounded corners
