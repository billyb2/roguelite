use macroquad::prelude::*;

pub trait Drawable {
    fn size(&self) -> Vec2;
    fn pos(&self) -> Vec2;
    fn rotation(&self) -> f32 {
        0.0

    }
    fn texture(&self) -> Option<Texture2D> {
        None

    }
    fn draw(&self) {
        let pos = self.pos();
        let size = self.size();
        
        match self.texture() {
            Some(texture) => {
                let texture_params = DrawTextureParams {
                    rotation: self.rotation(),
                    flip_x: true,
                    dest_size: Some(size),
                    ..Default::default()

                };
                draw_texture_ex(texture, pos.x, pos.y, WHITE, texture_params)

            },
            None =>  draw_rectangle(pos.x, pos.y, size.x, size.y, RED),
        };

    }

}
