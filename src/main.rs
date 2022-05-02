#![allow(dead_code)]
use frenderer::camera::Camera;
use frenderer::types::*;
use frenderer::{Engine, MousePos, Key, Result, WindowSettings};
use std::fs::File;
use std::io::{BufRead, BufReader};
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

    fn from_file(filepath: &str) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        let mut boxes: Vec<Self> = vec![];

        for line in reader.lines() {
            let unwrap = line.unwrap();
            let split: Vec<&str> = unwrap.split(' ').collect::<Vec<&str>>();
            let cast: Vec<f32> = split.iter().map(|s| s.parse::<f32>().unwrap()).collect();
            boxes.push(Self {
                min_x: cast[0],
                max_x: cast[1],
                min_y: cast[2],
                max_y: cast[3],
                min_z: cast[4],
                max_z: cast[5],
            })
        }

        Ok(boxes)
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
        } else if rest.y.abs() > rest.z.abs() {
            p.trf.translation.y += rest.y;
            p.vy = 0.;
            p.jump_count = 0;
        } else {
            p.trf.translation.z += rest.z;
        }
    }
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
            pitch: 0.,
            yaw: 0.,
            distance: 5.,
            player_pos: Vec3::zero(),
        }
    }
    fn update(&mut self, events: &frenderer::Input, player: &Player) {
        let MousePos { x: dx, y: dy } = events.mouse_delta();
        self.pitch += (DT * dy) as f32 / 10.0;
        self.pitch = self.pitch.clamp(-PI / 4.0, PI / 4.0);

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

struct Player {
    settings: PlayerSettings,
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
    vy: f32,
    jump_count: u8,
}

struct Level {
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
    bounding_boxes: Vec<BoundingBox>,
}
struct World {
    camera: Camera,
    camera_control: OrbitCamera,
    player: Player,
    level: Level,
    start: Vec3,
    end: Vec3,
}
struct Flat {
    trf: Similarity3,
    model: Rc<frenderer::renderer::flat::Model>,
}

impl frenderer::World for World {
    fn update(&mut self, input: &frenderer::Input, _assets: &mut frenderer::assets::Assets) {

        // JUMP MECHANICS
        if input.is_key_pressed(Key::Space) && self.player.jump_count < 2 {
            self.player.vy = 3. * self.player.settings.velocity;
            self.player.jump_count += 1;
        }
      
        // CALCULATE PLAYER MOVEMENT
        let rotation = Rotor3::from_euler_angles(0.0, 0.0, self.camera_control.yaw);
        self.player.vy += self.player.settings.gravity;
        let move_vec = rotation * Vec3::new(
            input.key_axis(Key::D, Key::A),
            self.player.vy,
            input.key_axis(Key::S, Key::W)
        );

        // EXECUTE PLAYER MOVEMENT
        self.player.trf.translation.x += self.player.settings.velocity * move_vec.x;
        self.player.trf.translation.y += move_vec.y;
        self.player.trf.translation.z += self.player.settings.velocity * move_vec.z;
      
        // GROUND CHECK
        if self.player.trf.translation.y < self.player.settings.radius {
            self.player.trf.translation = self.start;
            self.player.vy = 0.;
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
      
        // HANDLE COLLISION
        for b in &self.level.bounding_boxes {
            handle_collision(&mut self.player, b);
        }

        // ROTATE PLAYER
        self.player.trf.prepend_rotation(Rotor3 {
            s: 1.,
            bv: Bivec3 {
                xy: (move_vec.x / (2.*PI*self.player.settings.radius)) * rot_mult,
                xz: 0.,
                yz: -(move_vec.z / (2.*PI*self.player.settings.radius)) * rot_mult,
            },
        });

        // ADJUST CAMERA
        self.camera_control.update(input, &self.player);
        self.camera_control.update_camera(&mut self.camera);
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
        Vec3::new(0., 4., 0.),
        Vec3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
    );

    let settings = PlayerSettings {
        radius: 1.,
        velocity: 0.2,
        gravity: -0.03
    };

    let player_tex = engine.load_texture(std::path::Path::new("content/sphere.png"))?;
    let player_mesh = engine.load_textured(std::path::Path::new("content/sphere.obj"))?;
    let player_model = engine.create_textured_model(player_mesh, vec![player_tex]);

    let level_tex = engine.load_texture(std::path::Path::new("content/level_1.png"))?;
    let level_mesh = engine.load_textured(std::path::Path::new("content/level_1.obj"))?;
    let level_model = engine.create_textured_model(level_mesh, vec![level_tex, level_tex]);

    let bounding_boxes = BoundingBox::from_file("./content/level_1_bb.txt").unwrap();

    let world = World {
        camera,
        camera_control: OrbitCamera::new(),
        player: Player {
            settings,
            trf: Similarity3::new(Vec3::new(-12., 10., 11.5), Rotor3::identity(), 1.),
            model: player_model,
            vy: 0.,
            jump_count: 0,
        },
        level: Level {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0), Rotor3::identity(), 3.),
            model: level_model,
            bounding_boxes,
        },
        start: Vec3::new(-12., 10., 11.5),
        end: Vec3::new(0., 0., 0.)
    };
    engine.play(world)
}
// START + END
// NEW MODEL
// LOAD BOXES FROM FILE