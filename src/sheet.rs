//! Generate Texture Sheet.

use image;
use sheep;
use heck;

use sheep::{InputSprite, MaxrectsPacker, MaxrectsOptions};
use std::fs::read_dir;
use heck::ShoutySnakeCase;

pub struct Sprite {
    name: String,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

pub struct Named {
    sprites: Vec<Sprite>,
}

impl sheep::Format for Named {
    type Data = Named;
    type Options = Vec<String>;

    fn encode(
        dimensions: (u32, u32),
        in_sprites: &[sheep::SpriteAnchor],
        options: Self::Options
    ) -> Self::Data {
        let (w, h) = dimensions;
        let (w, h) = (w as f32, h as f32);
        let mut sprites = vec![];

        for i in 0..in_sprites.len() {
            let x1 = in_sprites[i].position.0 as f32 / w;
            let y1 = in_sprites[i].position.1 as f32 / h;
            let x2 = in_sprites[i].dimensions.0 as f32 / w;
            let y2 = in_sprites[i].dimensions.1 as f32 / h;
            let name = options[i].to_string();

            sprites.push(Sprite {
                name,
                x1,
                y1,
                x2,
                y2,
            });
        }

        Named {
            sprites,
        }
    }
}

pub fn write() -> String {
    // Find all PNG files.
    let paths = read_dir("./res/texture/").unwrap();
    let mut sprites = vec![];
    let mut names = vec![];

    for path in paths {
        let path = path.unwrap().path();
        let img = image::open(&path).unwrap();
        let img = img.as_rgba8().expect("Failed to convert image to rgba8");
        let dimensions = img.dimensions();
        let bytes = img
            .pixels()
            .flat_map(|it| it.data.iter().map(|it| *it))
            .collect::<Vec<u8>>();

        sprites.push(InputSprite {
            dimensions,
            bytes: bytes.clone(),
        });

        names.push(path.file_stem().unwrap().to_str().unwrap().to_string());
    }

    // Set texture sheet size.
    let options = MaxrectsOptions::default()
        .max_width(4096)
        .max_height(4096);

    // Do the actual packing! 4 defines the stride, since we're using rgba8 we
    // have 4 bytes per pixel.
    let results = sheep::pack::<MaxrectsPacker>(sprites, 4, options);

    // MaxrectsPacker always returns a single result. Other packers can return
    // multiple sheets; should they, for example, choose to enforce a maximum
    // texture size per sheet.
    let sprite_sheet = results
        .into_iter()
        .next()
        .expect("Should have returned a spritesheet");

    // Now, we can encode the sprite sheet in a format of our choosing to
    // save things such as offsets, positions of the sprites and so on.
    let meta = sheep::encode::<Named>(&sprite_sheet, names);

    // Next, we save the output to a file using the image crate again.
    let outbuf = image::RgbaImage::from_vec(
        sprite_sheet.dimensions.0,
        sprite_sheet.dimensions.1,
        sprite_sheet.bytes,
    ).expect("Failed to construct image from sprite sheet bytes");

    // SAVE OUTPUT

    let mut filename = std::env::var("OUT_DIR").unwrap();
    filename.push_str("/res/texture-sheet.png");
    outbuf.save(filename).expect("Failed to save image");

    // Lastly, we serialize the meta info using serde. This can be any format
    // you want, just implement the trait and pass it to encode.
    let mut meta_str = "pub(crate) const TEXTURE_SHEET: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/res/texture-sheet.png\"));\npub(crate) mod texture {\n".to_string();
    for i in &meta.sprites {
        meta_str.push_str(&format!("\tpub(crate) const {}: ([f32; 2],[f32; 2]) = ([{}f32, {}f32], [{}f32, {}f32]);\n", i.name.to_shouty_snake_case(), i.x1, i.y1, i.x2, i.y2));
    }
    meta_str.push_str("}\n");

    meta_str
}
