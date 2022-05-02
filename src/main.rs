#![allow(dead_code)]
// DeviceEvent
use frenderer::animation::{AnimationSettings, AnimationState};
use frenderer::assets::AnimRef;
use frenderer::camera::Camera;
use frenderer::types::*;
use frenderer::{Engine, MousePos, Key, Result, WindowSettings};
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
    camera_control: OrbitCamera,
    player: Player,
    level: Level,
    // level2: Level,
}
struct Flat {
    trf: Similarity3,
    model: Rc<frenderer::renderer::flat::Model>,
}
impl frenderer::World for World {
    fn update(&mut self, input: &frenderer::Input, _assets: &mut frenderer::assets::Assets) {
        let rotation = Rotor3::from_euler_angles(0.0, 0.0, self.camera_control.yaw);

        let move_vec = rotation * Vec3::new(
            input.key_axis(Key::D, Key::A),
            0.0,
            input.key_axis(Key::S, Key::W)
        );


        self.player.trf.translation.x += move_vec[0];
        self.player.trf.translation.z += move_vec[2];

        self.player.trf.prepend_rotation(Rotor3 {
            s: 1.,
            bv: Bivec3 {
                xy: move_vec[0] / CIRC,
                xz: 0.,
                yz: -move_vec[2] / CIRC,
            },
        });

        self.camera_control.update(input, &self.player);
        self.camera_control.update_camera(&mut self.camera);
        if input.is_key_pressed(Key::L) {
            println!("Load Level");
            
        }
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

    let player_tex = engine.load_texture(std::path::Path::new("content/robot.png"))?;
    let player_mesh = engine.load_textured(std::path::Path::new("content/sphere_test.obj"))?;
    let player_model = engine.create_textured_model(player_mesh, vec![player_tex]);

    let level_tex = engine.load_texture(std::path::Path::new("content/test_lvl_texture.png"))?;
    let level_mesh = engine.load_textured(std::path::Path::new("content/test_lvl.obj"))?;
    let level_model = engine.create_textured_model(level_mesh, vec![level_tex]);

    // let level_tex = engine.load_texture(std::path::Path::new("content/test_lvl_texture.png"))?;
    // let level_mesh = engine.load_textured(std::path::Path::new("content/test_lvl.obj"))?;
    // let level_model2 = engine.create_textured_model(level_mesh, vec![level_tex]);

    let world = World {
        camera,
        camera_control: OrbitCamera::new(),
        player: Player {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 50.0), Rotor3::identity(), 50.0),
            model: player_model,
        },
        level: Level {
            trf: Similarity3::new(Vec3::new(0.0, -20.0, 00.0), Rotor3::identity(), 20.0),
            model: level_model,
        },
        // level2: Level {
        //     trf: Similarity3::new(Vec3::new(0.0, -20.0, 00.0), Rotor3::identity(), 20.0),
        //     model: level_model,
        // }
    };
    engine.play(world)
}

pub struct OrbitCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
    player_pos: Vec3,
}
impl OrbitCamera {
    fn new() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            distance: 100.0,
            player_pos: Vec3::zero(),
        }
    }
    fn update(&mut self, events: &frenderer::Input, player: &Player) {
        let MousePos { x: dx, y: dy } = events.mouse_delta();
        self.pitch += (DT * dy) as f32 / 10.0;
        self.pitch = self.pitch.clamp(-0.1, PI / 4.0);

        self.yaw += (DT * dx) as f32 / 10.0;
        // self.yaw = self.yaw.clamp(-PI / 4.0, PI / 4.0);
        // self.distance += events.key_axis(Key::Up, Key::Down) * 5.0 * DT as f32;
        self.player_pos = player.trf.translation;
        // self.player_rot = player.trf.rotation;
        // TODO: when player moves, slightly move yaw towards zero
    }
    fn update_camera(&self, c: &mut Camera) {
        // The camera should point at the player (you could transform
        // this point to make it point at the player's head or center,
        // or at point in front of the player somewhere, instead of
        // their feet)
        let at = self.player_pos;
        // And rotated around the player's position and offset backwards
        let camera_rot = Rotor3::from_euler_angles(0.0, self.pitch, self.yaw);
        // self.player_rot = camera_rot;
        let offset = camera_rot * Vec3::new(0.0, 0.0, -self.distance);
        let eye = self.player_pos + offset;
        // To be fancy, we'd want to make the camera's eye an object
        // in the world whose rotation is locked to point towards the
        // player, and whose distance from the player is locked, and
        // so on---so we'd have player OR camera movements apply
        // accelerations to the camera which could be "beaten" by
        // collision.
        *c = Camera::look_at(eye, at, Vec3::unit_y());
    }
}
