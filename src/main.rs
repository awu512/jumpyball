#![allow(dead_code)]
use frenderer::camera::Camera;
use frenderer::types::*;
use frenderer::{Engine, Key, Result, WindowSettings};
use std::rc::Rc;

// GAME SETTINGS
const DT: f64 = 1.0 / 60.0; // time step

struct PlayerSettings {
    radius: f32,
    velocity: f32,
    gravity: f32
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
}

struct Sphere {
    pos: Vec3,
    r: f32,
}

fn handle_collision(p: &mut Player, b: &BoundingBox) {
    let s: Sphere = Sphere { 
        pos: p.trf.translation,
        r: p.settings.radius,
    };

    let closest = Vec3::new(
        s.pos.x.clamp(b.min_x, b.max_x),
        s.pos.y.clamp(b.min_y, b.max_y),
        s.pos.z.clamp(b.min_z, b.max_z),
    );

    let dist = closest - s.pos;

    if dist.mag() < s.r {
        let rest = (s.pos - closest).normalized() * (s.r-(s.pos - closest).mag());

        if rest.x.abs() > rest.y.abs() && rest.x.abs() > rest.z.abs() {
            p.trf.translation.x += rest.x;
            p.v.x = 0.;
        } else if rest.y.abs() > rest.z.abs() {
            p.trf.translation.y += rest.y;
            p.v.y = 0.;
            p.jump_count = 0;
        } else {
            p.trf.translation.z += rest.z;
            p.v.z = 0.;
        }
    }
}

struct Player {
    settings: PlayerSettings,
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
    v: Vec3,
    jump_count: u8,
}

struct Level {
    trf: Similarity3,
    // model: Rc<frenderer::renderer::textured::Model>,
    model: Rc<frenderer::renderer::flat::Model>,
    bounding_boxes: Vec<BoundingBox>,
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
        let mut dc: Vec3 = self.player.trf.translation;

        // X MOVEMENT
        if input.is_key_down(Key::Right) {
            self.player.v.x = self.player.settings.velocity
        } else if input.is_key_down(Key::Left) {
            self.player.v.x = -self.player.settings.velocity
        } else {
            self.player.v.x = 0.
        };

        // Z MOVEMENT
        if input.is_key_down(Key::Down) {
            self.player.v.z = self.player.settings.velocity
        } else if input.is_key_down(Key::Up) {
            self.player.v.z = -self.player.settings.velocity
        } else {
            self.player.v.z = 0.
        };

        // JUMP MECHANICS
        if input.is_key_pressed(Key::Space) && self.player.jump_count < 2 {
            self.player.v.y = 3. * self.player.settings.velocity;
            self.player.jump_count += 1;
        }

        // MAKE MOVEMENTS
        self.player.v.y += self.player.settings.gravity;
        self.player.trf.append_translation(self.player.v);

        // GROUND CHECK
        if self.player.trf.translation.y < self.player.settings.radius {
            self.player.trf.translation.y = self.player.settings.radius;
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
                xy: (self.player.v.x / self.player.settings.radius) * rot_mult,
                xz: 0.,
                yz: -(self.player.v.z / self.player.settings.radius) * rot_mult,
            },
        });

        // HANDLE COLLISION
        for b in &self.level.bounding_boxes {
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

        rs.render_flat(self.level.model.clone(), self.level.trf, 1);
        // rs.render_textured(self.level.model.clone(), self.level.trf, 1);
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

    let settings = PlayerSettings {
        radius: 1.,
        velocity: 0.2,
        gravity: -0.03
    };

    let player_tex = engine.load_texture(std::path::Path::new("content/sphere_test_spiral.png"))?;
    let player_mesh = engine.load_textured(std::path::Path::new("content/test.obj"))?;
    let player_model = engine.create_textured_model(player_mesh, vec![player_tex]);

    // let level_tex = engine.load_texture(std::path::Path::new("content/level_1.png"))?;
    // let level_mesh = engine.load_textured(std::path::Path::new("content/level_1.obj"))?;
    // let level_model = engine.create_textured_model(level_mesh, vec![level_tex, level_tex]);

    let level_model = engine.load_flat(std::path::Path::new("content/untitled.obj"));

    // let bounding_boxes = vec![
    //     // BoundingBox::new(25.076244354248047, 25.076244354248047, 6.903861999511719, 6.903861999511719, 1.0054539442062378, 1.0054539442062378),
    //     // BoundingBox::new(24.358890533447266, 24.358890533447266, 4.958309173583984, 4.958309173583984, -22.925790786743164, -22.925790786743164),
    //     BoundingBox::new(1.5   , 2.5   , 0. , 3. , 1.    , 3.    ),
    //     BoundingBox::new(-3.   , -2.   , 0. , 3. , -4.75 , -3.75 ),
    //     BoundingBox::new(-5.25 , -4.25 , 0. , 3. , -4.75 , -3.75 ),
    //     BoundingBox::new(-0.75 , 0.25  , 0. , 3. , -4.75 , -3.75 ),
    //     BoundingBox::new(-6.   , -5.   , 0. , 3. , 4.    , 5.    ),
    //     BoundingBox::new(-3.5  , -2.5  , 0. , 3. , 4.    , 5.    ),
    //     BoundingBox::new(1.5   , 2.5   , 0. , 3. , -4.75 , -3.75 ),
    //     BoundingBox::new(1.5   , 2.5   , 0. , 3. , -3.   , -1.   ),
    //     BoundingBox::new(-1.   , 0.0   , 0. , 3. , 4.    , 5.    ),
    //     // BoundingBox::new(-1.0006559021421708, 0.9993443362764083, -1.255562663078308, 4.24443781375885, 0.9995791912078857, 2.999579429626465)
    // ];

    let bounding_boxes = vec![
        BoundingBox::new(-1., 1., 0., 2., -1., 1.),
        BoundingBox::new(3., 5., 0., 2., -1., 1.),
        BoundingBox::new(7., 9., 0., 2., -1., 1.),
        BoundingBox::new(11., 13., 0., 2., -1., 1.),
        BoundingBox::new(15., 17., 0., 2., -1., 1.),
    ];

    let world = World {
        camera,
        player: Player {
            settings,
            trf: Similarity3::new(Vec3::new(0.0, 3.0, 0.0), Rotor3::identity(), 1.),
            model: player_model,
            v: Vec3::new(0., 0., 0.),
            jump_count: 0,
        },
        level: Level {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0), Rotor3::identity(), 1.),
            model: level_model.unwrap(),
            bounding_boxes,
        },
    };
    engine.play(world)
}
