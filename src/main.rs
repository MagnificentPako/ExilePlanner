use image::{codecs::png::PngDecoder, ImageDecoder};
use imgui::*;
use imgui_glium_renderer::glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Texture2d,
};
use imgui_glium_renderer::Texture;
use stats::{Stat, StatType};
use std::{borrow::Cow, collections::HashMap, io::Cursor, rc::Rc};
use texture_bag::TextureBag;
use tree::TreeExport;

mod colors;
mod stats;
mod support;
mod texture_bag;
mod tree;

#[derive(Default)]
struct Tree {
    pos: [f32; 2],
    zoom: f32,
}

#[derive(Default)]
struct TextureIDs {
    background: Option<TextureId>,
    group: Option<TextureId>,
    frame: Option<TextureId>,
    line: Option<TextureId>,
    normal_active: Option<TextureId>,
}

#[derive(Default)]
struct State {
    menu_bar_height: f32,
    textures: TextureIDs,
    selected_main_skill: String,
    main_skills_available: Vec<String>,
    stats: Vec<Stat>,
    tree: Tree,
    last_recorded_drag_delta: [f32; 2],
    tree_data: tree::TreeExport,
    sprite_texture_map: HashMap<String, TextureId>,
    debug_data: Option<String>,
    texture_bag: Option<texture_bag::TextureBag>,
}

fn group_of_sprite(state: &State, sprite: String) -> String {
    for group in state.tree_data.sprites.keys() {
        let sprite_group = state.tree_data.sprites.get(group).unwrap();
        for zoom_level in sprite_group.values().into_iter() {
            if zoom_level.coords.contains_key(&sprite) {
                return group.to_string();
            }
        }
    }
    "".to_string()
}

fn orbit_position(x: f32, y: f32, radius: f32, offset: usize, orbit: usize) -> [f32; 2] {
    [
        radius
            * (tree::ORBIT_ANGLES[orbit] as f32 * offset as f32)
                .to_radians()
                .cos()
            + x,
        radius
            * (tree::ORBIT_ANGLES[orbit] as f32 * offset as f32)
                .to_radians()
                .tan()
            + y,
    ]
}

fn menu(ui: &Ui, state: &mut State) {
    let mut menu_bar_height = 0.0;
    ui.main_menu_bar(|| {
        menu_bar_height = ui.window_size()[1];
        ui.menu("Test", || {
            ui.menu_item("Oops");
        })
    });
    state.menu_bar_height = menu_bar_height;
}

fn side_bar(ui: &Ui, state: &mut State) {
    let display_size = ui.io().display_size;
    let w = ui
        .window("Side Bar")
        .size(
            [300.0, display_size[1] - state.menu_bar_height],
            Condition::Always,
        )
        .position([0.0, state.menu_bar_height], Condition::Always)
        .resizable(false)
        .movable(false)
        .no_decoration();

    w.build(|| {
        ui.group(|| {
            ui.button("Import/Export");
            ui.same_line();
            ui.button("Notes");
            ui.same_line();
            ui.button("Configuration");
        });
        ui.group(|| {
            ui.button("Tree");
            ui.same_line();
            ui.button("Skills");
            ui.same_line();
            ui.button("Items");
            ui.same_line();
            ui.button("Calcs");
        });
        ui.separator();
        ui.text("Main Skill:");
        ui.set_next_item_width(-1.0);
        let main_skill_combo = ui.begin_combo("##", &state.selected_main_skill);
        if let Some(token) = main_skill_combo {
            for skill in state.main_skills_available.iter() {
                if ui.selectable(skill) {
                    state.selected_main_skill = skill.clone();
                }
            }
            token.end();
        }

        let stat_child_window = ui.child_window("Stats").always_vertical_scrollbar(true);
        stat_child_window.build(|| {
            ui.columns(2, "stat columns", false);

            for stat in state.stats.iter() {
                stat.show(ui);
            }
        });
    });
}

fn main_window(ui: &Ui, state: &mut State) {
    let display_size = ui.io().display_size;
    side_bar(ui, state);
    ui.same_line();
    let w = ui
        .window("Main Window")
        .size(
            [
                display_size[0] - 300.0,
                display_size[1] - state.menu_bar_height,
            ],
            Condition::Always,
        )
        .position([300.0, state.menu_bar_height], Condition::Always)
        .resizable(false)
        .movable(false)
        .no_decoration()
        .scrollable(false)
        .draw_background(false);

    let style = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
    w.build(|| {
        style.pop();

        let [width, height] = ui.io().display_size;
        let bag = state.texture_bag.as_ref().unwrap();

        let background = bag.get_sheet_for_file(&"background-3.png".to_string());
        ui.get_window_draw_list()
            .add_image(background.texture, [0.0, 0.0], [width, height])
            .uv_max([(width / 98.0), (height / 98.0)])
            .build();

        for group in state.tree_data.groups.values().into_iter() {
            if let Some(bg) = &group.background {
                let bag_sprite = bag.fetch_sprite(bg.image.clone()).unwrap();
                let x = group.x;
                let y = group.y;
                let half_image = bg.is_half_image.or(Some(false)).unwrap();
                ui.set_cursor_pos([
                    (width / 2.0) + (state.tree.pos[0] + x) * state.tree.zoom,
                    (height / 2.0) + (state.tree.pos[1] + y) * state.tree.zoom,
                ]);
                Image::new(
                    bag_sprite.texture,
                    [
                        bag_sprite.size[0] * state.tree.zoom,
                        bag_sprite.size[1] * state.tree.zoom,
                    ],
                )
                .uv0([
                    bag_sprite.pos[0] / bag_sprite.sheet_size[0],
                    bag_sprite.pos[1] / bag_sprite.sheet_size[1] as f32,
                ])
                .uv1([
                    (bag_sprite.pos[0] + bag_sprite.size[0]) / bag_sprite.sheet_size[0] as f32,
                    (bag_sprite.pos[1] + bag_sprite.size[1]) / bag_sprite.sheet_size[1] as f32,
                ])
                .build(ui);
                if half_image {
                    ui.set_cursor_pos([
                        (width / 2.0) + (state.tree.pos[0] + x) * state.tree.zoom,
                        (height / 2.0)
                            + (state.tree.pos[1] + y + bag_sprite.size[1] as f32) * state.tree.zoom,
                    ]);
                    Image::new(
                        bag_sprite.texture,
                        [
                            bag_sprite.size[0] * state.tree.zoom,
                            bag_sprite.size[1] * state.tree.zoom,
                        ],
                    )
                    .uv1([
                        (bag_sprite.pos[0] + bag_sprite.size[0]) / bag_sprite.sheet_size[0],
                        bag_sprite.pos[1] / bag_sprite.sheet_size[1],
                    ])
                    .uv0([
                        bag_sprite.pos[0] / bag_sprite.sheet_size[0],
                        (bag_sprite.pos[1] + bag_sprite.size[1]) / bag_sprite.sheet_size[1],
                    ])
                    .build(ui);
                }
            }

            let draw_list = ui.get_window_draw_list();
            for node in state.tree_data.nodes.values().into_iter() {
                if let Some(icon) = &node.icon {
                    if let Some(orbit) = node.orbit {
                        let group_num = node.group.unwrap();
                        let group_str = format!("{}", group_num);
                        let group = state.tree_data.groups.get(&group_str).unwrap();
                        let sprite_ = bag.fetch_sprite(icon.to_string());
                        if let Some(sprite) = sprite_ {
                            let [x, y] = orbit_position(
                                group.x,
                                group.y,
                                state.tree_data.constants.orbit_radii[orbit] as f32,
                                node.orbit_index.unwrap_or(0),
                                orbit,
                            );
                            ui.set_cursor_pos([
                                (width / 2.0) + (state.tree.pos[0] + x) * state.tree.zoom,
                                (height / 2.0) + (state.tree.pos[1] + y) * state.tree.zoom,
                            ]);
                            /*
                               Image::new(
                                   sprite.texture,
                                   [
                                       sprite.size[0] as f32 * state.tree.zoom,
                                       sprite.size[1] as f32 * state.tree.zoom,
                                   ],
                               )
                               .uv0([
                                   sprite.pos[0] / sprite.sheet_size[0],
                                   sprite.pos[1] / sprite.sheet_size[1],
                               ])
                               .uv1([
                                   (sprite.pos[0] + sprite.size[0]) / sprite.sheet_size[0],
                                   (sprite.pos[1] + sprite.size[1]) / sprite.sheet_size[1],
                               ])
                               .build(ui);
                            */

                            let p_min = [
                                (width / 2.0) + (state.tree.pos[0] + x) * state.tree.zoom,
                                (height / 2.0) + (state.tree.pos[1] + y) * state.tree.zoom,
                            ];
                            let p_max = [p_min[0] + sprite.size[0], p_min[1] + sprite.size[1]];

                            draw_list
                                .add_image(sprite.texture, p_min, p_max)
                                .uv_min([
                                    sprite.pos[0] / sprite.sheet_size[0],
                                    sprite.pos[1] / sprite.sheet_size[1],
                                ])
                                .uv_max([
                                    (sprite.pos[0] + sprite.size[0]) / sprite.sheet_size[0],
                                    (sprite.pos[1] + sprite.size[1]) / sprite.sheet_size[1],
                                ])
                                .build();
                        }
                    }
                }
            }
        }
        if ui.is_mouse_dragging(MouseButton::Left) {
            let mouse_delta = ui.mouse_drag_delta();
            let delta = [
                -(state.last_recorded_drag_delta[0] - mouse_delta[0]),
                -(state.last_recorded_drag_delta[1] - mouse_delta[1]),
            ];
            state.tree.pos[0] += delta[0];
            state.tree.pos[1] += delta[1];

            state.last_recorded_drag_delta = mouse_delta;

            state.tree.pos = [
                state.tree.pos[0].clamp(state.tree_data.min_x, state.tree_data.max_x),
                state.tree.pos[1].clamp(state.tree_data.min_y, state.tree_data.max_y),
            ]
        } else {
            state.last_recorded_drag_delta = [0.0, 0.0];
        }

        if ui.is_window_hovered() {
            state.tree.zoom += ui.io().mouse_wheel * 0.05;
            state.tree.zoom = state.tree.zoom.clamp(0.3, 1.5)
        }
    });
}

fn skeleton(ui: &Ui, state: &mut State) {
    menu(ui, state);
    main_window(ui, state);
    if let Some(debug_data) = &state.debug_data {
        ui.window("Debug Window").build(|| {
            ui.text(debug_data);
        });
    }
}

impl State {
    fn run(&mut self, ui: &Ui) {
        skeleton(&ui, self);
    }
}

fn filename_from_url(url: String) -> String {
    let parsed = url::Url::parse(url.as_str()).unwrap();
    let segments = parsed.path_segments().unwrap();
    segments.last().unwrap().to_string()
}

fn main() {
    let mut system = support::init(file!());
    let mut app = State::default();

    app.tree_data = TreeExport::new().unwrap();
    app.tree.pos = [0.0, 0.0];
    app.tree.zoom = 1.0;
    app.main_skills_available = vec!["Discharge".to_string(), "Conversion Trap".to_string()];
    app.stats = vec![Stat {
        name: "Dexterity".to_string(),
        value: StatType::Number(67.0),
        name_color: colors::GREEN,
        value_color: colors::WHITE,
    }];

    let mut texture_bag = TextureBag::new();

    for sprite_group in app.tree_data.sprites.values().into_iter() {
        if sprite_group.contains_key("0.3835") {
            let sprites = sprite_group.get("0.3835").unwrap();
            let filename = filename_from_url(sprites.filename.clone());
            let mut sheet_opt = None;
            if !texture_bag.has_texture(&filename) {
                println!("Loading texture");
                sheet_opt = Some(texture_bag.load_texture(
                    system.display.get_context(),
                    system.renderer.textures(),
                    &filename,
                ));
            } else {
                sheet_opt = Some(texture_bag.get_sheet_for_file(&filename))
            }
            println!("{:?}, {}", sheet_opt, filename);
            let sheet = sheet_opt.unwrap();
            for (k, v) in sprites.coords.iter() {
                texture_bag.create_sprite(
                    k,
                    [v.x as f32, v.y as f32],
                    [v.w as f32, v.h as f32],
                    &sheet,
                )
            }
        }
    }

    app.texture_bag = Some(texture_bag);

    system.main_loop(move |_, ui| app.run(ui));
}
