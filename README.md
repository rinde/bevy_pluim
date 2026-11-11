# bevy-pluim

A reusable widget library for [Bevy](https://bevy.org) UI, built on Bevy's
native `Node` UI. Buttons, lists, combos, scroll areas, tabs, sliders and more,
all themed from one `Theme` resource with light and dark modes.

The crate holds no application or game state; it takes Bevy types only. It is
private software (`publish = false`).

## Bevy version

Targets **Bevy 0.20**. The library and the Bevy in your app must resolve to
the same version.

## Adding it

The crate is not on crates.io; use it as a git dependency:

```toml
[dependencies]
bevy = "0.20"
bevy_pluim = { git = "https://github.com/rinde/bevy-pluim" }
```

Pin a `rev` for reproducible builds. 

## A minimal app

Add `WidgetsPlugin` and spawn a camera and one root `Node`; widgets go below
the root.

```rust
use bevy::prelude::*;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::push_button::PushButton;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            padding: UiRect::all(px(20.0)),
            ..default()
        })
        .with_child(PushButton::from_label("Hello"));
}
```

## Widgets

Buttons: `PushButton`, `IconButton`, `ColorPushButton`, `Link`.
Selection: `List`, `VirtualList` (thousands of rows), `Combo`, `ColorPalette`,
`ColorPaletteCombo`, `SegmentedControl`, `TabBar`, `FilterChips`, `Gallery`,
`Toolbar`, `BreadCrumbs`, `Stack`.
Inputs: `Slider`, `NumericStepper`, `TextField`, `LightDarkToggle`.
Layout: `Card`, `Panel`, `ScrollArea`, `SettingRow`, `Divider`.
Display: `Label`, `Icon`, `Kbd`, `Badge`, `Pill`, `Tooltip`, `Toast`.

Colours, fonts and radii come from the `Theme` resource; switch modes with
`Theme::set_mode` or a `LightDarkToggle`.

## Examples

Examples need a window. With [`just`](https://just.systems):

```sh
just example overview
```

or `cargo run --example overview`. `overview` shows every standalone widget on
its own page (the theme editor too), `hud` composes a game-style HUD, and `gallery` is the scenario
tester's window. The full list is in [AGENTS.md](AGENTS.md).

## Contributing

Work is tracked in GitHub issues and lands through pull requests. See
[AGENTS.md](AGENTS.md) for the module map, conventions, tests and the checks
(`just fmt`, `just lint`, `just test`) a change must pass.
