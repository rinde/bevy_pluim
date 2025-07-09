

# Todo
- [v] ScrollBar
- [v] List as 2D grid + List horizontal
- [ ] Custom item bundle (color square)
- [ ] Customizable List bundle to show tooltip
- [ ] Tabs
- [ ] TabPane
- [ ] Card
- [ ] ClearColor: auto update ClearColor on Theme change
- [ ] Create separate scroll component?
- [ ] List: allow optionally setting maximum height
- [ ] Combo: allow specifying placeholder text
- [ ] Combo: ensure List is visible within window
- [ ] ScrollArea: allow dragging thumb


# Open questions
- How to ensure that an item is fully within the window? How to obtain the actual coordinates of the node? (`ComputedNode` only contains the size, not location). TODO try to get the GlobalTransform of the node. transform.compute_transform()

# Blocked
- Scrolling in a container with rounded corners doesn't clip the rounded corners
