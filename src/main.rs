#![allow(dead_code)]
use frenderer::camera::Camera;
use frenderer::types::*;
use frenderer::{Engine, MousePos, Key, Result, WindowSettings};
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
            distance: 10.0,
            player_pos: Vec3::zero(),
        }
    }
    fn update(&mut self, events: &frenderer::Input, player: &Player) {
        let MousePos { x: dx, y: dy } = events.mouse_delta();
        self.pitch += (DT * dy) as f32 / 10.0;
        self.pitch = self.pitch.clamp(0.0, PI / 3.0);

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
    v: Vec3,
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
}
struct Flat {
    trf: Similarity3,
    model: Rc<frenderer::renderer::flat::Model>,
}

impl frenderer::World for World {
    fn update(&mut self, input: &frenderer::Input, _assets: &mut frenderer::assets::Assets) {

        // JUMP MECHANICS
        if input.is_key_pressed(Key::Space) && self.player.jump_count < 2 {
            self.player.v.y = 3. * self.player.settings.velocity;
            self.player.jump_count += 1;
        }
      
        // CALCULATE PLAYER MOVEMENT
        let rotation = Rotor3::from_euler_angles(0.0, 0.0, self.camera_control.yaw);
        self.player.v.y += self.player.settings.gravity;
        let move_vec = rotation * Vec3::new(
            input.key_axis(Key::D, Key::A),
            self.player.v.y,
            input.key_axis(Key::S, Key::W)
        );
        
        // EXECUTE PLAYER MOVEMENT
        self.player.trf.translation.x += move_vec[0] * self.player.settings.velocity;
        self.player.trf.translation.y += move_vec[1];
        self.player.trf.translation.z += move_vec[2] * self.player.settings.velocity;
      
        // GROUND CHECK
        if self.player.trf.translation.y < self.player.settings.radius {
            self.player.trf.translation.y = self.player.settings.radius;
            self.player.v.y = 0.;
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
                xy: (move_vec[0] / 10.) * rot_mult,
                xz: 0.,
                yz: -(move_vec[2] / 10.) * rot_mult,
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
        Vec3::new(0., 4., 7.),
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

    let bounding_boxes = vec![
        BoundingBox::new(-18.0, 18.0, 0.0, 0.0, -18.0, 18.0), 
        BoundingBox::new(5.998920500278473, 8.998920857906342, 0.3587096929550171, 8.608710408210754, -7.499444782733917, -4.499444425106049), 
        BoundingBox::new(-7.50017112493515, -4.5001707673072815, 0.3587082624435425, 8.60870897769928, 9.75146108865738, 12.751461446285248), 
        BoundingBox::new(-14.253300368785858, -11.25330001115799, 0.20693814754486084, 8.456938862800598, 9.75294953584671, 12.752949893474579), 
        BoundingBox::new(-0.7519842088222504, 2.2480161488056183, 0.35869288444519043, 8.608693599700928, 9.751274406909943, 12.751274764537811), 
        BoundingBox::new(-16.501183211803436, -13.501182854175568, 0.35847795009613037, 8.608478665351868, -16.499910056591034, -13.499909698963165), 
        BoundingBox::new(-9.011482179164886, -6.011481821537018, 0.3514394760131836, 8.601440191268921, -16.504347503185272, -13.504347145557404), 
        BoundingBox::new(5.998819649219513, 8.998820006847382, 0.358479380607605, 8.608480095863342, 9.749907553195953, 12.749907910823822), 
        BoundingBox::new(5.998144447803497, 8.998144805431366, 0.35839176177978516, 8.608391761779785, 0.0007392168045043945, 6.000739932060242), 
        BoundingBox::new(-1.5021528888610192, 1.4978474687668495, 0.35870790481567383, 8.608708620071411, -16.499106109142303, -13.499105751514435), 
        BoundingBox::new(-1.5019675276125781, 1.4980328300152905, 0.35831236839294434, 8.608313083648682, -7.49873811006546, -4.4987377524375916), 
    ];

    let world = World {
        camera,
        camera_control: OrbitCamera::new(),
        player: Player {
            settings,
            trf: Similarity3::new(Vec3::new(0.0, 3.0, 0.0), Rotor3::identity(), 1.),
            model: player_model,
            v: Vec3::new(0., 0., 0.),
            jump_count: 0,
        },
        level: Level {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0), Rotor3::identity(), 3.),
            model: level_model,
            bounding_boxes,
        },
    };
    engine.play(world)
}