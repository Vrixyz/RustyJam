use bevy::prelude::*;
use bevy_egui::*;

pub struct FlashMessage {
    pub message: String,
    pub color: egui::Color32,
    pub time_expire: f32,
}

fn flash_message(
    mut commands: Commands,
    time: Res<Time>,
    mut general_input: ResMut<GeneralInput>,
    egui_context: ResMut<EguiContext>,
    mut q_messages: Query<(Entity, &mut FlashMessage)>,
) {
    egui::Area::new("flash")
        .fixed_pos(egui::Pos2::new(10f32, 10f32))
        .show(egui_context.ctx(), |ui| {
            for (e, mut f) in q_messages.iter_mut() {
                ui.colored_label(f.color, f.message.clone());
                if f.time_expire <= time.time_since_startup().as_secs_f32() {
                    commands.entity(e).despawn();
                }
            }
        });
}

mod GameButton {
    use super::*;

    pub struct GeneralInput {
        pub clicks: usize,
        pub catched: usize,
    }
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

    pub struct MovingDef {
        pub path: Vec<egui::Pos2>,
        pub speed: f32,
    }
    pub struct MovingState {
        pub target_index: usize,
    }

    // Translated from https://answers.unity.com/questions/414829/any-one-know-maths-behind-this-movetowards-functio.html
    fn move_towards(current: egui::Pos2, target: egui::Pos2, max_distance: f32) -> egui::Pos2 {
        let a = target - current;
        let magnitude = a.length();
        if magnitude <= max_distance || magnitude == 0f32 {
            return target;
        }
        current + a / magnitude * max_distance
    }
    pub fn button_move(
        time: Res<Time>,
        mut q_buttons: Query<(&mut MovingDef, &mut MovingState, &mut ButtonInfo)>,
    ) {
        let delta_time = time.delta().as_secs_f32();
        for (def, mut state, mut info) in q_buttons.iter_mut() {
            let target = def.path[state.target_index];
            let new_pos = move_towards(info.position.min, target, def.speed * delta_time);
            info.position = egui::Rect::from_min_size(new_pos, info.position.size());
            if new_pos == target {
                state.target_index = (state.target_index + 1) % def.path.len();
            }
        }
    }

    pub fn button_blink(
        time: Res<Time>,
        mut q_buttons: Query<(&mut Blinking, &mut BlinkState, &mut ButtonInfo)>,
    ) {
        let elapsed_time = time.time_since_startup().as_secs_f32();
        for (def, mut state, mut info) in q_buttons.iter_mut() {
            match *state {
                BlinkState::Visible(v) => {
                    if v <= elapsed_time {
                        *state = BlinkState::Hidden(elapsed_time + def.hidden);
                        info.visible = false;
                    }
                }
                BlinkState::Hidden(h) => {
                    if h <= elapsed_time {
                        *state = BlinkState::Visible(elapsed_time + def.hidden);
                        info.visible = true;
                    }
                }
            }
        }
    }
    pub fn button_letters(
        mut commands: Commands,
        mut time: Res<Time>,
        general_input: Res<GeneralInput>,
        mut q_buttons: Query<(&mut LetterByLetter, &UserInput, &mut ButtonInfo)>,
    ) {
        for (mut letters, input, mut info) in q_buttons.iter_mut() {
            if 0 < general_input.clicks && 0 == general_input.catched {
                letters.current_index = 0;
                info.text = letters.full_string.chars().take(1).collect::<String>();
                // TODO: spawn entity to show "ILLUSION" as label for a short time
                commands.spawn().insert(FlashMessage {
                    message: "ILLUSION".into(),
                    color: egui::Color32::YELLOW,
                    time_expire: time.time_since_startup().as_secs_f32() + 0.5f32,
                });
            }
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
        mut general_input: ResMut<GeneralInput>,
        egui_context: ResMut<EguiContext>,
        mut q_buttons: Query<(&ButtonInfo, &mut UserInput)>,
    ) {
        egui::Area::new("Hello")
            .fixed_pos(egui::Pos2::new(0f32, 0f32))
            .show(egui_context.ctx(), |ui| {
                for (info, mut input) in q_buttons.iter_mut() {
                    ui.allocate_ui_at_rect(info.position, |ui| {
                        let button = if info.visible {
                            ui.button(info.text.to_string())
                        } else {
                            ui.label(" ")
                        };
                        if button.is_pointer_button_down_on() && ui.input().pointer.any_pressed() {
                            input.clicked_on_frame = true;
                            general_input.catched += 1;
                        }
                    });
                }
                if ui.input().pointer.any_pressed() {
                    general_input.clicks = 1;
                }
            });
    }
    pub fn reset_input(
        mut general_input: ResMut<GeneralInput>,
        mut q_buttons: Query<&mut UserInput>,
    ) {
        for mut input in q_buttons.iter_mut() {
            input.clicked_on_frame = false;
        }
        general_input.catched = 0;
        general_input.clicks = 0;
    }
}

use GameButton::*;
fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(GeneralInput {
            clicks: 0,
            catched: 0,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup_level.system())
        //.add_system(ui_example.system())
        .add_system_to_stage(CoreStage::PreUpdate, reset_input.system())
        .add_system_to_stage(CoreStage::Update, display_buttons.system())
        .add_system_to_stage(CoreStage::Update, flash_message.system())
        .add_system_to_stage(CoreStage::PostUpdate, button_letters.system())
        .add_system_to_stage(CoreStage::PostUpdate, button_blink.system())
        .add_system_to_stage(CoreStage::PostUpdate, button_move.system())
        .run();
}

fn setup_level(time: Res<Time>, mut commands: Commands) {
    commands
        .spawn()
        .insert(ButtonInfo {
            text: "S".into(),
            position: egui::Rect::from_min_size([10f32, 20f32].into(), [10f32, 50f32].into()),
            visible: true,
        })
        .insert(LetterByLetter {
            current_index: 0,
            full_string: "Security".into(),
        })
        .insert(UserInput::default())
        .insert(Blinking {
            displayed: 0.33f32,
            hidden: 1f32,
        })
        .insert(BlinkState::Visible(
            time.time_since_startup().as_secs_f32() + 2f32,
        ))
        .insert(MovingDef {
            path: vec![[10f32, 20f32].into(), [10f32, 100f32].into()],
            speed: 25f32,
        })
        .insert(MovingState { target_index: 0 });
}
