

# Todo
- [ ] Combo: fix bug in unhighlighting label when deselecting
- [ ] explore using <query>.get(entity_id) e.g. for finding combos etc
- [ ] List: allow optionally setting maximum height
- [ ] Create separate scroll component?
- [ ] Refactor: use observers for mouse interaction. EntityCommands::observe(). See bevy scroll example.
- [ ] Combo: allow specifying placeholder text
- [ ] Combo: ensure List is visible within window
- [ ] ClearColor: auto update ClearColor on Theme change


# Open questions
- How to ensure that an item is fully within the window? How to obtain the actual coordinates of the node? (`ComputedNode` only contains the size, not location). TODO try to get the GlobalTransform of the node. transform.compute_transform()

# Blocked
- Scrolling in a container with rounded corners doesn't clip the rounded corners
