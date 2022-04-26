#![allow(dead_code)]
// DeviceEvent
use frenderer::animation::{AnimationSettings, AnimationState};
use frenderer::assets::AnimRef;
use frenderer::camera::Camera;
use frenderer::types::*;
use frenderer::{Engine, Key, Result, WindowSettings};
use std::rc::Rc;

const DT: f64 = 1.0 / 60.0;
const CIRC: f32 = 50. / PI;

struct GameObject {
    trf: Similarity3,
    model: Rc<frenderer::renderer::skinned::Model>,
    animation: AnimRef,
    state: AnimationState,
}
impl GameObject {
    fn tick_animation(&mut self) {
        self.state.tick(DT);
    }
}

struct Player {
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
}

struct Level {
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
}
struct Sprite {
    trf: Isometry3,
    tex: frenderer::assets::TextureRef,
    cel: Rect,
    size: Vec2,
}
struct World {
    camera: Camera,
    player: Player,
    level: Level,
}
struct Flat {
    trf: Similarity3,
    model: Rc<frenderer::renderer::flat::Model>,
}
impl frenderer::World for World {
    fn update(&mut self, input: &frenderer::Input, _assets: &mut frenderer::assets::Assets) {
        let dx = if input.is_key_down(Key::Right) {
            1.
        } else if input.is_key_down(Key::Left) {
            -1.
        } else {
            0.0
        };

        let dz = if input.is_key_down(Key::Down) {
            1.
        } else if input.is_key_down(Key::Up) {
            -1.
        } else {
            0.0
        };

        self.player.trf.translation.x += dx;
        self.player.trf.translation.z += dz;

        self.player.trf.prepend_rotation(Rotor3 {
            s: 1.,
            bv: Bivec3 {
                xy: dx / CIRC,
                xz: 0.,
                yz: -dz / CIRC,
            },
        });

        self.camera
            .transform
            .prepend_translation(Vec3::new(-dx, 0., -dz));
    }
    fn render(
        &mut self,
        _a: &mut frenderer::assets::Assets,
        rs: &mut frenderer::renderer::RenderState,
    ) {
        rs.set_camera(self.camera);

        rs.render_textured(self.player.model.clone(), self.player.trf, 0);

        rs.render_textured(self.level.model.clone(), self.level.trf, 1);
    }
} 
fn main() -> Result<()> {
    frenderer::color_eyre::install()?;

    let mut engine: Engine = Engine::new(WindowSettings::default(), DT);

    let camera = Camera::look_at(
        Vec3::new(0., 100., 100.),
        Vec3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
    );

    let player_tex = engine.load_texture(std::path::Path::new("content/sphere_test_spiral.png"))?;
    let player_mesh = engine.load_textured(std::path::Path::new("content/sphere_test.obj"))?;
    let player_model = engine.create_textured_model(player_mesh, vec![player_tex]);

    let level_tex = engine.load_texture(std::path::Path::new("content/test_lvl_texture.png"))?;
    let level_mesh = engine.load_textured(std::path::Path::new("content/test_lvl.obj"))?;
    let level_model = engine.create_textured_model(level_mesh, vec![level_tex]);

    let world = World {
        camera,
        player: Player {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 50.0), Rotor3::identity(), 50.0),
            model: player_model,
        },
        level: Level {
            trf: Similarity3::new(Vec3::new(0.0, -20.0, 00.0), Rotor3::identity(), 20.0),
            model: level_model,
        },
    };
    engine.play(world)
}
