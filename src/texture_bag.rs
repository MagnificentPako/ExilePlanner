use std::{borrow::Cow, collections::HashMap, io::Cursor, path::PathBuf, rc::Rc};

use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Texture2d,
};
use imgui::{TextureId, Textures};
use imgui_glium_renderer::Texture;

#[derive(Clone, Copy, Debug)]
pub struct SpriteSheet {
    pub size: [f32; 2],
    pub texture: TextureId,
}

pub struct Sprite {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub sheet_size: [f32; 2],
    pub texture: TextureId,
}

pub struct TextureBag {
    path_id_map: HashMap<String, SpriteSheet>,
    arbitrary_map: HashMap<String, Sprite>,
}

impl TextureBag {
    pub fn new() -> TextureBag {
        TextureBag {
            path_id_map: HashMap::default(),
            arbitrary_map: HashMap::default(),
        }
    }

    pub fn has_texture(&self, path: &String) -> bool {
        println!(
            "Checking path: {:?}, {:?}",
            path,
            self.path_id_map.contains_key(path)
        );
        self.path_id_map.contains_key(path)
    }

    pub fn get_sheet_for_file(&self, path: &String) -> SpriteSheet {
        self.path_id_map.get(path).unwrap().clone()
    }

    pub fn load_texture<F: Facade>(
        &mut self,
        gl_ctx: &F,
        textures: &mut Textures<Texture>,
        path: &String,
    ) -> SpriteSheet {
        let full_path = format!("resources/ggg_assets/{}", path);
        let dyn_image = image::open(full_path).unwrap();
        let img = dyn_image.to_rgba8();
        let (width, height) = img.dimensions();
        let raw = RawImage2d {
            data: Cow::Owned(img.to_vec()),
            width,
            height,
            format: ClientFormat::U8U8U8U8,
        };
        let gl_texture = Texture2d::new(gl_ctx, raw).unwrap();
        let texture = Texture {
            texture: Rc::new(gl_texture),
            sampler: SamplerBehavior {
                magnify_filter: MagnifySamplerFilter::Linear,
                minify_filter: MinifySamplerFilter::Linear,
                ..Default::default()
            },
        };
        let texture_id = textures.insert(texture);
        let sheet = SpriteSheet {
            size: [width as f32, height as f32],
            texture: texture_id,
        };
        self.path_id_map.insert(path.to_string(), sheet);
        sheet
    }

    pub fn create_sprite(
        &mut self,
        name: &String,
        pos: [f32; 2],
        size: [f32; 2],
        sheet: &SpriteSheet,
    ) {
        self.arbitrary_map.insert(
            name.to_string(),
            Sprite {
                pos: pos,
                size: size,
                sheet_size: sheet.size,
                texture: sheet.texture,
            },
        );
    }

    pub fn fetch_sprite(&self, name: String) -> Option<&Sprite> {
        self.arbitrary_map.get(&name)
    }
}
