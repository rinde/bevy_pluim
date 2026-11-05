//! `TextField`: a single line of text a person can type in.

use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{
    AcquireFocus, FocusCause, FocusGained, FocusLost, FocusedInput, InputFocus,
};
use bevy::prelude::*;
use bevy::text::{EditableText, EditableTextSystems, TextCursorStyle, TextEdit, TextEditChange};
use bevy::ui_widgets::TextInput;
use itertools::Either;

use crate::focus::{
    AccessibilitySync, RingOn, RingTarget, TabStop, clear_label, sync_label, sync_value,
};
use crate::theme::Theme;

/// Registers the systems of [`TextField`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin). A text field also needs Bevy's
/// `TextInputPlugin` and `PointerFocusPlugin`, which `UiWidgetsPlugins` (and so
/// `DefaultPlugins`) adds: the first edits the text, the second gives the field the focus
/// on a click and takes it away on a click anywhere else.
#[derive(Copy, Clone, Debug)]
pub struct TextFieldPlugin;

impl Plugin for TextFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (|| None::<Entity>)
                .pipe(theme_text_fields)
                .run_if(resource_changed::<Theme>),
        )
        .add_systems(
            PostUpdate,
            sync_accessibility
                .in_set(AccessibilitySync)
                .after(EditableTextSystems),
        );
    }
}

/// The padding between the border of a [`TextField`] and its text.
const PADDING: UiRect = UiRect {
    left: Val::Px(10.0),
    right: Val::Px(10.0),
    top: Val::Px(7.0),
    bottom: Val::Px(7.0),
};

/// The default [`TextField::min_width`], as `Val::Px` (logical pixels).
pub const DEFAULT_MIN_WIDTH_PX: f32 = 160.0;

/// The default [`TextField::max_width`], as `Val::Px` (logical pixels).
pub const DEFAULT_MAX_WIDTH_PX: f32 = 320.0;

/// A single-line text input: a bordered box that a person clicks to type in.
///
/// The box has the theme's `card_surface` fill, a 1 px `border` and the control radius; the
/// text is body text in `text_primary`. While the field is empty it shows its `placeholder`,
/// if it has one, in `text_tertiary`. While it has the keyboard focus the focus ring is
/// around the box, whether the focus came from a click or from Tab, so a person sees where
/// the keys go.
///
/// A click anywhere on the box, its border included, gives the field the focus (it is a tab
/// stop as well). `Enter` (also the numpad's) gives the focus up
/// and so does `Escape`, which also puts back the text the field had when it took the focus;
/// both keys stop there and do not bubble on to the field's ancestors;
/// a press anywhere else gives it up too, as does Tab.
///
/// [`TextFieldChange`] is triggered on the field whenever its text changes, and
/// [`TextFieldCommit`] when the field gives the focus up other than by `Escape`: by `Enter`,
/// Tab or a press elsewhere, even when the text did not change.
///
/// The box is at least `min_width` and at most `max_width` wide; within those it takes the
/// width its parent gives it, never the width of its text, which scrolls inside the box
/// instead.
///
/// The two widths are Bevy layout constraints, not a validated pair of pixel measurements:
/// each is a [`Val`] passed unchanged to the field's `Node` as `min_width` / `max_width`, so
/// `Val::Auto`, percentages (of the parent's width), viewport units and the rest work, and
/// are resolved by layout. Only `Val::Px` is in logical pixels. Nothing is converted or
/// compared before layout. When both resolve and the minimum is above the maximum, the
/// minimum wins, as in Bevy's layout: a field with a 400 px minimum and the default 320 px
/// maximum is 400 px wide. That is a layout rule, not an error; neither bound is clamped and
/// nothing is logged.
///
/// The fields are read when the component is inserted: insert a new `TextField` to set the
/// text, placeholder, name or widths. That sets the text without a [`TextFieldChange`].
/// The text being typed is in the `EditableText` of [`TextFieldRefs::input`], and in the
/// events.
///
/// To a screen reader the field is a text input (on [`TextFieldRefs::input`]) with its text
/// as its value and `name`, if any, as its label.
///
/// `TextField` is an immutable component: it is configuration, replaced by inserting a new
/// one, never edited in place, so a system cannot take it as `&mut`. Migrating: a `Query<&mut TextField>` becomes
/// `commands.entity(e).insert(..)` of a prepared `TextField`. Inserting a new `TextField` sets its text without a `TextFieldChange`.
///
/// A `&mut` query does not compile:
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use bevy_pluim::text_field::TextField;
///
/// fn edit(mut widgets: Query<&mut TextField>) {
///     for mut widget in &mut widgets {
///         let _ = &mut *widget;
///     }
/// }
/// ```
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[component(immutable, on_add = Self::on_add, on_insert = Self::on_insert)]
#[require(
    Name::new("TextField"),
    Node {
        border: UiRect::all(Val::Px(1.0)),
        overflow: Overflow::clip(),
        ..default()
    },
    BorderColor,
    BackgroundColor,
    RingTarget,
)]
pub struct TextField {
    /// The text the field holds when it is inserted.
    pub value: String,
    /// The text shown, in `text_tertiary`, while the field is empty.
    pub placeholder: Option<String>,
    /// The accessible name of the field, the label a screen reader reads.
    pub name: Option<String>,
    /// The least width of the box, a layout constraint for the field's `Node::min_width`
    /// ([`DEFAULT_MIN_WIDTH_PX`] logical pixels by default). It wins over `max_width` when
    /// both resolve and it is the greater.
    pub min_width: Val,
    /// The greatest width of the box, a layout constraint for the field's `Node::max_width`
    /// ([`DEFAULT_MAX_WIDTH_PX`] logical pixels by default).
    pub max_width: Val,
}

impl TextField {
    /// A field holding `value`, without a placeholder or a name, between
    /// [`DEFAULT_MIN_WIDTH_PX`] and [`DEFAULT_MAX_WIDTH_PX`] wide.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            placeholder: None,
            name: None,
            min_width: Val::Px(DEFAULT_MIN_WIDTH_PX),
            max_width: Val::Px(DEFAULT_MAX_WIDTH_PX),
        }
    }

    /// The same field, showing `placeholder` while it is empty.
    #[must_use]
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// The same field, with `name` as its accessible name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The same field, at least `min_width` wide: any [`Val`], passed to the `Node` as is
    /// (logical pixels only for `Val::Px`). It takes precedence over a smaller `max_width`.
    #[must_use]
    pub const fn with_min_width(mut self, min_width: Val) -> Self {
        self.min_width = min_width;
        self
    }

    /// The same field, at most `max_width` wide: any [`Val`], passed to the `Node` as is
    /// (logical pixels only for `Val::Px`). A greater `min_width` takes precedence over it.
    #[must_use]
    pub const fn with_max_width(mut self, max_width: Val) -> Self {
        self.max_width = max_width;
        self
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let field = context.entity;
        let mut commands = world.commands();
        let input = commands
            .spawn((
                Name::new("TextFieldInput"),
                TextFieldInput {
                    field,
                    value: String::new(),
                    on_focus: None,
                },
                TextInput,
                TextLayout::no_wrap(),
                TabStop,
                Node {
                    flex_grow: 1.0,
                    min_width: Val::Px(0.0),
                    padding: PADDING,
                    ..default()
                },
                RingOn(field),
                ChildOf(field),
            ))
            .observe(on_focus_gained)
            .observe(on_focus_lost)
            .observe(on_key_input)
            .observe(on_text_edit)
            .id();
        let placeholder = commands
            .spawn((
                Name::new("TextFieldPlaceholder"),
                Text::default(),
                TextLayout::no_wrap(),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    padding: PADDING,
                    ..default()
                },
                Pickable::IGNORE,
                ChildOf(field),
            ))
            .id();
        commands
            .entity(field)
            .insert(TextFieldRefs { input, placeholder })
            .observe(on_field_acquire_focus);
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(apply_text_field, context.entity);
        world
            .commands()
            .run_system_cached_with(theme_text_fields, Some(context.entity));
    }
}

/// The parts of a [`TextField`].
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct TextFieldRefs {
    /// The `TextInput` that holds the text in its `EditableText` and takes the focus.
    pub input: Entity,
    /// The text shown while the field is empty.
    pub placeholder: Entity,
}

/// Triggered on a [`TextField`] when its text changes, by typing or by `Escape` putting it
/// back; not when the app sets it by inserting a new `TextField`.
#[derive(EntityEvent, Debug, Clone, PartialEq, Eq)]
pub struct TextFieldChange {
    /// The [`TextField`].
    pub entity: Entity,
    /// Its new text.
    pub value: String,
}

/// Triggered on a [`TextField`] when it gives up the focus by `Enter`, Tab or a press
/// elsewhere (not by `Escape`), with the text it holds then.
#[derive(EntityEvent, Debug, Clone, PartialEq, Eq)]
pub struct TextFieldCommit {
    /// The [`TextField`].
    pub entity: Entity,
    /// Its text.
    pub value: String,
}

/// The text input of a [`TextField`], with what the field remembers about its text.
#[derive(Component, Clone, Debug)]
struct TextFieldInput {
    /// The [`TextField`] this input belongs to.
    field: Entity,
    /// The text at the last [`TextFieldChange`], or as the app last set it.
    value: String,
    /// The text when the field took the focus, while it has it.
    on_focus: Option<String>,
}

/// Puts the fields of a newly inserted [`TextField`] on its parts.
fn apply_text_field(
    field: In<Entity>,
    mut fields: Query<(&TextField, &TextFieldRefs, &mut Node)>,
    mut inputs: Query<(&mut TextFieldInput, &mut EditableText)>,
    mut placeholders: Query<(&mut Text, &mut Node), Without<TextField>>,
) {
    let Ok((text_field, refs, mut node)) = fields.get_mut(field.0) else {
        return;
    };
    node.min_width = text_field.min_width;
    node.max_width = text_field.max_width;
    if let Ok((mut input, mut editable)) = inputs.get_mut(refs.input) {
        if editable.value().to_string() != text_field.value {
            editable.pending_edits.clear();
            editable.editor.set_text(&text_field.value);
            editable.queue_edit(TextEdit::TextEnd(false));
        }
        input.value.clone_from(&text_field.value);
    }
    if let Ok((mut text, mut placeholder_node)) = placeholders.get_mut(refs.placeholder) {
        text.0 = text_field.placeholder.clone().unwrap_or_default();
        placeholder_node.display = placeholder_display(&text_field.value);
    }
}

/// Shows the placeholder only while the text is empty.
const fn placeholder_display(value: &str) -> Display {
    if value.is_empty() {
        Display::Flex
    } else {
        Display::None
    }
}

/// Gives the focus to the input of a field when a press lands on the box around it, its
/// 1 px border included, rather than letting the request bubble on and clear the focus.
fn on_field_acquire_focus(
    mut trigger: On<AcquireFocus>,
    fields: Query<&TextFieldRefs>,
    mut focus: ResMut<InputFocus>,
) {
    let Ok(refs) = fields.get(trigger.event_target()) else {
        return;
    };
    trigger.propagate(false);
    if focus.get() != Some(refs.input) {
        focus.set(refs.input, FocusCause::Navigated);
    }
}

/// Remembers the text when the field takes the focus, for `Escape` to put back.
fn on_focus_gained(
    trigger: On<FocusGained>,
    mut inputs: Query<(&mut TextFieldInput, &EditableText)>,
) {
    let Ok((mut input, editable)) = inputs.get_mut(trigger.event_target()) else {
        return;
    };
    input.on_focus = Some(editable.value().to_string());
}

/// Commits the text when the field gives the focus up, unless `Escape` did.
///
/// The focus changes after the text edits of the frame are applied, so the text is complete.
fn on_focus_lost(
    trigger: On<FocusLost>,
    mut inputs: Query<(&mut TextFieldInput, &EditableText)>,
    mut commands: Commands,
) {
    let Ok((mut input, editable)) = inputs.get_mut(trigger.event_target()) else {
        return;
    };
    if input.on_focus.take().is_some() {
        commands.trigger(TextFieldCommit {
            entity: input.field,
            value: editable.value().to_string(),
        });
    }
}

/// Marks the input of a [`TextField`] whose `Enter` and `Escape` belong to the widget around it
/// (a [`FilterCombo`](crate::filter_combo::FilterCombo)'s search box), so the field leaves them
/// alone instead of giving the focus up.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct OwnKeys;

/// `Enter` gives the focus up; `Escape` puts the text back as it was when the field took
/// the focus and gives the focus up too.
fn on_key_input(
    mut trigger: On<FocusedInput<KeyboardInput>>,
    mut inputs: Query<(&mut TextFieldInput, &mut EditableText), Without<OwnKeys>>,
    mut focus: ResMut<InputFocus>,
) {
    let key = &trigger.event().input;
    if !key.state.is_pressed() {
        return;
    }
    let key_code = key.key_code;
    let Ok((mut input, mut editable)) = inputs.get_mut(trigger.event_target()) else {
        return;
    };
    if editable.is_composing() {
        return;
    }
    match key_code {
        KeyCode::Enter | KeyCode::NumpadEnter => {
            focus.clear();
            trigger.propagate(false);
        }
        KeyCode::Escape => {
            if let Some(original) = input.on_focus.take()
                && editable.value().to_string() != original
            {
                editable.pending_edits.clear();
                editable.editor.set_text(&original);
                editable.queue_edit(TextEdit::TextEnd(false));
            }
            focus.clear();
            trigger.propagate(false);
        }
        _ => {}
    }
}

/// Triggers a [`TextFieldChange`] when an edit changed the text, and shows the placeholder
/// while it is empty.
fn on_text_edit(
    trigger: On<TextEditChange>,
    mut inputs: Query<(&mut TextFieldInput, &EditableText)>,
    refs: Query<&TextFieldRefs>,
    mut nodes: Query<&mut Node>,
    mut commands: Commands,
) {
    let Ok((mut input, editable)) = inputs.get_mut(trigger.event_target()) else {
        return;
    };
    let value = editable.value().to_string();
    if value == input.value {
        return;
    }
    if let Ok(refs) = refs.get(input.field)
        && let Ok(mut node) = nodes.get_mut(refs.placeholder)
    {
        let display = placeholder_display(&value);
        if node.display != display {
            node.display = display;
        }
    }
    input.value.clone_from(&value);
    commands.trigger(TextFieldChange {
        entity: input.field,
        value,
    });
}

/// Keeps the text of each field as the value of its accessibility node, and its name as the
/// label. A node is only looked at when its text, its `TextField` or the node itself changed.
fn sync_accessibility(
    mut inputs: Query<(&TextFieldInput, Ref<EditableText>, &mut AccessibilityNode)>,
    fields: Query<Ref<TextField>>,
) {
    for (input, editable, mut node) in &mut inputs {
        let field = fields.get(input.field).ok();
        let field_changed = field.as_ref().is_some_and(DetectChanges::is_changed);
        if !(editable.is_changed() || field_changed || node.is_changed()) {
            continue;
        }
        sync_value(&mut node, &editable.value().to_string());
        match field.as_ref().and_then(|field| field.name.as_deref()) {
            Some(name) => sync_label(&mut node, name),
            None => clear_label(&mut node),
        }
    }
}

fn theme_text_fields(
    field: In<Option<Entity>>,
    theme: Res<Theme>,
    mut fields: Query<(
        &TextFieldRefs,
        &mut Node,
        &mut BorderColor,
        &mut BackgroundColor,
    )>,
    mut texts: Query<(&mut TextColor, &mut TextFont, Option<&mut TextCursorStyle>)>,
    mut commands: Commands,
) {
    let iter = if let Some(field) = field.0 {
        Either::Left(fields.get_mut(field).into_iter())
    } else {
        Either::Right(fields.iter_mut())
    };

    let colors = theme.colors();
    let font = TextFont {
        font: theme.body_font.clone().into(),
        font_size: Theme::font_size(theme.font_size_body),
        ..default()
    };
    let cursor = TextCursorStyle {
        color: colors.text_primary,
        selection_color: colors.accent_subtle,
        unfocused_selection_color: colors.accent_subtle,
        selected_text_color: Some(colors.accent_subtle_text),
        selection_radius: 0.0,
    };
    for (refs, mut node, mut border, mut background) in iter {
        node.border_radius = BorderRadius::all(theme.control_radius);
        *border = BorderColor::all(colors.border);
        background.0 = colors.card_surface;
        for (part, color) in [
            (refs.input, colors.text_primary),
            (refs.placeholder, colors.text_tertiary),
        ] {
            let Ok((mut text_color, mut text_font, cursor_style)) = texts.get_mut(part) else {
                continue;
            };
            text_color.0 = color;
            *text_font = font.clone();
            if part == refs.input {
                match cursor_style {
                    Some(mut style) => *style = cursor,
                    None => {
                        commands.entity(part).insert(cursor);
                    }
                }
            }
        }
    }
}
