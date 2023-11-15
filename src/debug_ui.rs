use bevy::{
    diagnostic::{DiagnosticId, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::particle::{NEW_PARTICLE_COUNT, PARTICLE_COUNT};
// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText(DiagnosticId);

// A unit struct to help identify the color-changing Text component
#[derive(Component)]
struct ColorText;

#[derive(Bundle)]
struct DiagnosticBundle {
    pub text_bundle: TextBundle,
    pub diagnostic: FpsText,
}
impl DiagnosticBundle {
    pub fn new(text: &str, diagnostic: DiagnosticId, asset_server: &AssetServer) -> Self {
        Self {
            text_bundle: TextBundle::from_sections([
                TextSection::new(
                    text,
                    TextStyle {
                        // This font is loaded and will be used instead of the default font.
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font_size: 30.0,
                    color: Color::GOLD,
                    // If no font is specified, it will use the default font.
                    ..default()
                }),
            ]),
            diagnostic: FpsText(diagnostic),
        }
    }
}
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(DiagnosticBundle::new(
        "fps",
        FrameTimeDiagnosticsPlugin::FPS,
        &asset_server,
    ));
    if false {
        commands.spawn(DiagnosticBundle::new(
            "particles",
            PARTICLE_COUNT,
            &asset_server,
        ));
        commands.spawn(DiagnosticBundle::new(
            "new particles",
            NEW_PARTICLE_COUNT,
            &asset_server,
        ));
    }
}

fn text_color_system(time: Res<Time>, mut query: Query<&mut Text, With<ColorText>>) {
    for mut text in &mut query {
        let seconds = time.elapsed_seconds();

        // Update the color of the first and only section.
        text.sections[0].style.color = Color::Rgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
    }
}

fn text_update_system(diagnostics: Res<DiagnosticsStore>, mut query: Query<(&mut Text, &FpsText)>) {
    for (mut text, FpsText(diag)) in &mut query {
        if let Some(fps) = diagnostics.get(*diag) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
                text.sections[1].style.color = match value as u32 {
                    58.. => Color::GREEN,
                    50..=57 => Color::CYAN,
                    40..=49 => Color::YELLOW_GREEN,
                    30..=39 => Color::YELLOW,
                    _ => Color::RED,
                };
            }
        }
    }
}
pub struct DebugUiPlugin;
impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (text_update_system, text_color_system));
    }
}
