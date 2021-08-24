use std::string;

use bevy::prelude::*;
use bevy_egui::*;

#[derive(Default)]
pub struct GameState {
    pub errors: Vec<egui::Pos2>,
    pub success: usize,
}

mod GameButton {
    use super::*;

    pub struct ButtonInfo {
        pub text: String,
        pub position: egui::Rect,
        pub visible: bool,
    }
    #[derive(Default)]
    pub struct UserInput {
        pub clicked_on_frame: bool,
    }
    #[derive(Default)]
    pub struct LetterByLetter {
        pub current_index: usize,
        pub full_string: String,
    }
    pub struct Blinking {
        pub displayed: f32,
        pub hidden: f32,
    }
    pub enum BlinkState {
        Visible(f32),
        Hidden(f32),
    }

    pub fn button_blink(
        time: Res<&Time>,
        mut q_buttons: Query<(&mut Blinking, &mut BlinkState, &mut ButtonInfo)>,
    ) {
        for (def, mut state, mut info) in q_buttons.iter_mut() {
            match *state {
                BlinkState::Visible(_) => todo!(),
                BlinkState::Hidden(_) => todo!(),
            }
        }
    }
    pub fn button_letters(
        mut q_buttons: Query<(&mut LetterByLetter, &UserInput, &mut ButtonInfo)>,
    ) {
        for (mut letters, input, mut info) in q_buttons.iter_mut() {
            if input.clicked_on_frame {
                letters.current_index += 1;
                info.text = letters
                    .full_string
                    .chars()
                    .skip(letters.current_index)
                    .take(1)
                    .collect::<String>();
            }
        }
    }
    pub fn display_buttons(
        egui_context: ResMut<EguiContext>,
        mut q_buttons: Query<(&ButtonInfo, &mut UserInput)>,
    ) {
        egui::Area::new("Hello")
            .fixed_pos(egui::Pos2::new(0f32, 0f32))
            .show(egui_context.ctx(), |ui| {
                for (info, mut input) in q_buttons.iter_mut() {
                    ui.allocate_ui_at_rect(info.position, |ui| {
                        if ui.button(info.text.to_string()).is_pointer_button_down_on()
                            && ui.input().pointer.any_pressed()
                        {
                            input.clicked_on_frame = true;
                        }
                    });
                }
            });
    }
    pub fn reset_input(mut q_buttons: Query<&mut UserInput>) {
        for mut input in q_buttons.iter_mut() {
            input.clicked_on_frame = false;
        }
    }
}

pub struct LevelDef {
    pub buttons: Vec<(egui::Rect, String, usize, bool)>,
    pub show_errors: bool,
    pub error: String,
    pub text_placeholder: String,
    pub text_win: String,
}
use GameButton::*;
fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(GameState::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup_level.system())
        //.add_system(ui_example.system())
        .add_system_to_stage(CoreStage::PreUpdate, reset_input.system())
        .add_system_to_stage(CoreStage::Update, display_buttons.system())
        .add_system_to_stage(CoreStage::PostUpdate, button_letters.system())
        .run();
}

fn setup_level(mut commands: Commands) {
    commands
        .spawn()
        .insert(ButtonInfo {
            text: "S".into(),
            position: egui::Rect::from_min_size([200f32, 10f32].into(), [200f32, 50f32].into()),
            visible: true,
        })
        .insert(LetterByLetter {
            current_index: 0,
            full_string: "Safety".into(),
        })
        .insert(UserInput::default());
    commands.insert_resource(LevelDef {
        buttons: vec![
            (
                egui::Rect::from_min_size([100f32, 10f32].into(), [200f32, 50f32].into()),
                "Reliability".into(),
                0,
                false,
            ),
            (
                egui::Rect::from_min_size([100f32, 10f32].into(), [100f32, 50f32].into()),
                "Don't worry!".into(),
                11,
                false,
            ),
        ],
        show_errors: true,
        error: "Illusion".into(),
        text_placeholder: "Click on what you see".into(),
        text_win: "You are in security".into(),
    });
}

// Note the usage of `ResMut`. Even though `ctx` method doesn't require
// mutability, accessing the context from different threads will result
// into panic if you don't enable `egui/multi_threaded` feature.
fn ui_example(egui_context: ResMut<EguiContext>, mut state: ResMut<GameState>, def: Res<LevelDef>) {
    egui::Area::new("Hello2")
        .fixed_pos(egui::Pos2::new(10f32, 10f32))
        .show(egui_context.ctx(), |ui| {
            let mut is_correct_click = false;
            for b in def.buttons.iter() {
                ui.allocate_ui_at_rect(b.0, |ui| {
                    if b.3 {
                        if b.2 == state.success
                            && ui.button(b.1.to_string()).is_pointer_button_down_on()
                            && ui.input().pointer.any_pressed()
                        {
                            state.success += 1;
                            is_correct_click = true;
                        }
                    } else if b.2 <= state.success
                        && state.success - b.2 < b.1.len()
                        && ui
                            .button(
                                b.1.chars()
                                    .skip(state.success - b.2)
                                    .take(1)
                                    .collect::<String>(),
                            )
                            .is_pointer_button_down_on()
                        && ui.input().pointer.any_pressed()
                    {
                        state.success += 1;
                        is_correct_click = true;
                    }
                });
            }
            if !is_correct_click && ui.input().pointer.any_pressed() {
                if let Some(position) = ui.input().pointer.press_origin() {
                    state.errors.push(position);
                }
            }
            ui.allocate_ui_at_rect(
                egui::Rect::from_min_size([0f32, 50f32].into(), [200f32, 50f32].into()),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(
                            egui::Color32::BLUE,
                            def.text_win.chars().take(state.success).collect::<String>(),
                        );
                        ui.colored_label(
                            egui::Color32::WHITE,
                            def.text_placeholder
                                .chars()
                                .skip(state.success)
                                .collect::<String>(),
                        );
                    })
                },
            );
            if def.show_errors {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    def.error
                        .chars()
                        .take(state.errors.len())
                        .collect::<String>(),
                );
            }
        });
}
