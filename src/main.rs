#![allow(dead_code)]

use frenderer::animation::{AnimationState};
use frenderer::assets::AnimRef;
use frenderer::camera::Camera;
use frenderer::types::*;
use frenderer::{Engine, Key, Result, WindowSettings};
use std::rc::Rc;

const DT: f64 = 1.0 / 60.0;
const CIRC: f32 = 2. / PI;
const PLAYER_R: f32 = 1.0;

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

struct BoundingBox {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
}

impl BoundingBox {
    fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32, min_z: f32, max_z: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z
        }
    }

    fn center(self) -> Vec3 {
        Vec3::new(
            (self.max_x - self.min_x) / 2.,
            (self.max_y - self.min_y) / 2.,
            (self.max_z - self.min_z) / 2.,
        )
    }
}

struct Sphere {
    pos: Vec3,
    r: f32,
}

struct Plane {
    // A normal, has to be a unit vector
    n: Vec3,
    // And a distance of how far along the normal this is
    d: f32,
}

fn sphere_in_box(p: &Player, b: &BoundingBox) -> bool {
    // Find the distance of the sphere's center to the plane
    let s: Sphere = Sphere { 
        pos: p.trf.translation + Vec3::new(0.32, 0.32, 0.32),
        r: 0.32,
    };

    let offsets: Vec<Vec3> = vec![
        Vec3::new(s.r, 0., 0.),
        Vec3::new(-s.r, 0., 0.),
        Vec3::new(0., s.r, 0.),
        Vec3::new(0., -s.r, 0.),
        Vec3::new(0., 0., s.r),
        Vec3::new(0., 0., -s.r),
    ];

    for o in offsets {
        let p: Vec3 = s.pos + o;
        if p.x >= b.min_x && p.x <= b.max_x &&
            p.y >= b.min_y && p.y <= b.max_y &&
            p.z >= b.min_z && p.z <= b.max_z
        {
            return true;
        }
    }
    false
}

fn handle_collision(p: &mut Player, b: &BoundingBox) {
    let s: Sphere = Sphere { 
        pos: p.trf.translation + Vec3::new(0.32, 0.32, 0.32),
        r: 0.32,
    };

    let closest = Vec3::new(
        s.pos.x.clamp(b.min_x, b.max_x),
        s.pos.y.clamp(b.min_y, b.max_y),
        s.pos.z.clamp(b.min_z, b.max_z),
    );

    dbg!(closest);

    let dist = closest - s.pos;

    if dist.mag() < s.r {
        let rest = (s.pos - closest).normalized() * (s.r-(s.pos - closest).mag());

        // dbg!(s.pos, dist, rest);

        if rest.x.abs() > rest.y.abs() && rest.x.abs() > rest.z.abs() {
            p.trf.translation.x += rest.x;
            p.v.x = 0.;
        } else if rest.y.abs() > rest.z.abs() {
            p.trf.translation.y += rest.y;
            p.v.y = 0.;
        } else {
            p.trf.translation.z += rest.z;
            p.v.z = 0.;
        }
    }
}

struct Player {
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
    a: Vec3,
    v: Vec3,
    jump_count: u8,
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
    bounding_boxes: Vec<BoundingBox>,
}
struct Flat {
    trf: Similarity3,
    model: Rc<frenderer::renderer::flat::Model>,
}
impl frenderer::World for World {
    fn update(&mut self, input: &frenderer::Input, _assets: &mut frenderer::assets::Assets) {
        let mut dc: Vec3 = self.player.trf.translation;

        // X MOVEMENT
        if input.is_key_down(Key::Right) {
            self.player.v.x = 0.1
        } else if input.is_key_down(Key::Left) {
            self.player.v.x = -0.1
        } else {
            self.player.v.x = 0.
        };

        // Z MOVEMENT
        if input.is_key_down(Key::Down) {
            self.player.v.z = 0.1
        } else if input.is_key_down(Key::Up) {
            self.player.v.z = -0.1
        } else {
            self.player.v.z = 0.
        };

        // JUMP MECHANICS
        if input.is_key_pressed(Key::Space) && self.player.jump_count < 2 {
            self.player.v.y = 0.5;
            self.player.jump_count += 1;
        }

        // MAKE MOVEMENTS
        self.player.v += self.player.a;
        self.player.trf.append_translation(self.player.v);

        // GROUND CHECK
        if self.player.trf.translation.y < 0.32 {
            self.player.trf.translation.y = 0.32;
            self.player.jump_count = 0;
        }

        // ADJUST ROTATION BASED ON JUMP
        let rot_mult = if self.player.jump_count == 0 {
            1.
        } else if self.player.jump_count == 1 {
            0.5
        } else {
            2.
        };

        // ROTATE PLAYER
        self.player.trf.prepend_rotation(Rotor3 {
            s: 1.,
            bv: Bivec3 {
                xy: (self.player.v.x / CIRC) * rot_mult,
                xz: 0.,
                yz: (-self.player.v.z / CIRC) * rot_mult,
            },
        });

        // HANDLE COLLISION
        for b in &self.bounding_boxes {
            handle_collision(&mut self.player, b);
        }

        // MATCH CAMERA
        dc -= self.player.trf.translation;
        self.camera.transform.prepend_translation(dc);
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
        Vec3::new(0., 4., 7.),
        Vec3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
    );

    let player_tex = engine.load_texture(std::path::Path::new("content/sphere_test_spiral.png"))?;
    let player_mesh = engine.load_textured(std::path::Path::new("content/sphere_test.obj"))?;
    let player_model = engine.create_textured_model(player_mesh, vec![player_tex]);

    let level_tex = engine.load_texture(std::path::Path::new("content/level_1.png"))?;
    let level_mesh = engine.load_textured(std::path::Path::new("content/level_1.obj"))?;
    let level_model = engine.create_textured_model(level_mesh, vec![level_tex, level_tex]);

    let bounding_boxes = vec![
        BoundingBox::new(-4.0, -2.0, 0.0, 2.0, -1.0, 1.0),
    ];

    let world = World {
        camera,
        player: Player {
            trf: Similarity3::new(Vec3::new(0.0, 2.0, 2.5), Rotor3::identity(), 1.),
            model: player_model,
            a: Vec3::new(0., -0.03, 0.),
            v: Vec3::new(0., 0., 0.),
            jump_count: 0,
        },
        level: Level {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0), Rotor3::identity(), 1.),
            model: level_model,
        },
        bounding_boxes
    };
    engine.play(world)
}
