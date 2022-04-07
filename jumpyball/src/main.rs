#![allow(dead_code)]

use frenderer::animation::{AnimationSettings, AnimationState};
use frenderer::assets::AnimRef;
use frenderer::camera::Camera;
use frenderer::types::*;
use frenderer::{Engine, Key, Result, WindowSettings};
use std::rc::Rc;

const DT: f64 = 1.0 / 60.0;

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
    player: Player,
    level: Level,
}
struct Flat {
    trf: Similarity3,
    model: Rc<frenderer::renderer::flat::Model>,
}
impl frenderer::World for World {
    fn update(&mut self, input: &frenderer::Input, _assets: &mut frenderer::assets::Assets) {
        Camera::look_at(
            // eye
            Vec3::new(0., 0., 100.),
            // at
            Vec3::new(0., 0., -10.),
            // up
            Vec3::new(0., 1., 0.),
        );

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

        // let yaw = if input.is_key_down(Key::Z) {
        //     (PI / 4.0) * (1.0 / 60.0)
        // } else {
        //     0.0
        // };
        // let pitch = if input.is_key_down(Key::X) {
        //     (PI / 4.0) * (1.0 / 60.0)
        // } else {
        //     0.0
        // };
        // let roll = if input.is_key_down(Key::C) {
        //     (PI / 4.0) * (1.0 / 60.0)
        // } else {
        //     0.0
        // };
        // let dscale = if input.is_key_down(Key::Up) {
        //     1.0 / 60.0
        // } else {
        //     0.0
        // };

        // self.player.trf.rotation =
        //     Rotor3::from_euler_angles(roll, pitch, yaw) * self.player.trf.rotation;
        // self.player.trf.scale += dscale;
        self.player.trf.translation.x += dx;
        self.player.trf.translation.z += dz;
        // dbg!(obj.trf.rotation);
        // obj.tick_animation();

        // for s in self.sprites.iter_mut() {
        //     let yaw = if input.is_key_down(Key::A) {
        //         (PI / 4.0) * (1.0 / 60.0)
        //     } else {
        //         0.0
        //     };
        //     let pitch = if input.is_key_down(Key::S) {
        //         (PI / 4.0) * (1.0 / 60.0)
        //     } else {
        //         0.0
        //     };
        //     let roll = if input.is_key_down(Key::D) {
        //         (PI / 4.0) * (1.0 / 60.0)
        //     } else {
        //         0.0
        //     };
        //     let dscale = if input.is_key_down(Key::F) {
        //         1.0 / 60.0
        //     } else {
        //         0.0
        //     };
        //     s.trf.rotation = Rotor3::from_euler_angles(roll, pitch, yaw) * s.trf.rotation;
        //     s.size.x += dscale;
        //     s.size.y += dscale;
        // }
        // for m in self.flats.iter_mut() {
        //     let yaw = if input.is_key_down(Key::Q) {
        //         (PI / 4.0) * (1.0 / 60.0)
        //     } else {
        //         0.0
        //     };
        //     let pitch = if input.is_key_down(Key::W) {
        //         (PI / 4.0) * (1.0 / 60.0)
        //     } else {
        //         0.0
        //     };
        //     let roll = if input.is_key_down(Key::E) {
        //         (PI / 4.0) * (1.0 / 60.0)
        //     } else {
        //         0.0
        //     };
        //     let dscale = if input.is_key_down(Key::R) {
        //         1.0 / 60.0
        //     } else {
        //         0.0
        //     };
        //     m.trf.rotation = Rotor3::from_euler_angles(roll, pitch, yaw) * m.trf.rotation;
        //     m.trf.scale += dscale;
        // }
    }
    fn render(
        &mut self,
        _a: &mut frenderer::assets::Assets,
        rs: &mut frenderer::renderer::RenderState,
    ) {
        rs.render_textured(self.player.model.clone(), self.player.trf, 0);

        rs.render_textured(self.level.model.clone(), self.level.trf, 1);

        // for (s_i, s) in self.sprites.iter_mut().enumerate() {
        //     rs.render_sprite(s.tex, s.cel, s.trf, s.size, s_i);
        // }
        // for (m_i, m) in self.flats.iter_mut().enumerate() {
        //     rs.render_flat(m.model.clone(), m.trf, m_i);
        // }
    }
}
fn main() -> Result<()> {
    frenderer::color_eyre::install()?;

    let mut engine: Engine = Engine::new(WindowSettings::default(), DT);
    let mut looki = -10.;

    engine.set_camera(Camera::look_at(
        Vec3::new(0., 200., 200.),
        Vec3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
    ));

    let player_tex = engine.load_texture(std::path::Path::new("content/sphere_test.png"))?;
    let player_mesh = engine.load_textured(std::path::Path::new("content/sphere_test.obj"))?;
    let player_model = engine.create_textured_model(player_mesh, vec![player_tex]);

    let level_tex = engine.load_texture(std::path::Path::new("content/test_lvl_texture.png"))?;
    let level_mesh = engine.load_textured(std::path::Path::new("content/test_lvl.obj"))?;
    let level_model = engine.create_textured_model(level_mesh, vec![level_tex]);

    let world = World {
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
