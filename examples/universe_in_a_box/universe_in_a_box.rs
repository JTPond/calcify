use std::fmt;
use std::str::FromStr;
use std::error;

extern crate rayon;
use rayon::prelude::*;

extern crate rmp;
use rmp::encode::*;

extern crate calcify;
pub use calcify::ThreeVec;
use calcify::{Serializable, Deserializable};


#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Particle {
    pid: usize,
    mass: f64,
    charge: f64,
    position: ThreeVec,
    velocity: ThreeVec,
    t_force: ThreeVec,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Universe {
    dt: f64,
    energy: f64,
    pub state: Vec<Particle>,
    pub previous_state: Vec<Particle>,
}

impl Particle {
    pub fn dust(pid: usize, max: f64) -> Particle {
        let mass = 100.0;
        let charge = 0.0;
        let position = ThreeVec::random(max);
        let velocity = ThreeVec::new(0.0,0.0,0.0);
        let t_force = ThreeVec::new(0.0,0.0,0.0);
        Particle {
            pid,
            mass,
            charge,
            position,
            velocity,
            t_force
        }
    }

    pub fn pid(&self) -> &usize {
        &self.pid
    }

    pub fn m(&self) -> &f64 {
        &self.mass
    }

    pub fn q(&self) -> &f64 {
        &self.charge
    }

    pub fn r(&self) -> &ThreeVec {
        &self.position
    }

    pub fn v(&self) -> &ThreeVec {
        &self.velocity
    }

    pub fn f(&self) -> &ThreeVec {
        &self.t_force
    }

    pub fn translate(&mut self, dr: &ThreeVec) {
        self.position += *dr;
    }

    pub fn accelerate(&mut self, dv: &ThreeVec) {
        self.velocity += *dv;
    }

    pub fn force(&mut self, parts2: &Vec<Particle>) {
        let g: f64 = 6.67408e-11;
        self.t_force = parts2.par_iter().map(|&part2| {
            if *part2.pid() != self.pid {
                let rmrp = self.position - *part2.r();
                let r2 = rmrp*rmrp;
                return (rmrp*(1.0/r2.sqrt()))*((-g*self.mass*part2.m())*(1.0/r2));
            } else {
                return ThreeVec::new(0.0,0.0,0.0);
            }
        }).sum();
    }

}

impl fmt::Display for Particle{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(pid:{}, m:{}, q:{}, r:{}, v:{})", self.pid(), self.m(), self.q(), self.r(), self.v())
    }
}

impl Serializable for Particle {
    fn to_json(&self) -> String {
        format!("{{\"pid\":{},\"m\":{},\"q\":{},\"r\":{},\"v\":{}}}", self.pid(), self.m(), self.q(), self.r().to_json(), self.v().to_json())
    }
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::new();
        write_array_len(&mut buf, 5)?;
        write_uint(&mut buf, *self.pid() as u64)?;
        buf.append(&mut self.m().to_msg()?);
        buf.append(&mut self.q().to_msg()?);
        buf.append(&mut self.r().to_msg()?);
        buf.append(&mut self.v().to_msg()?);
        Ok(buf)
    }
}

impl FromStr for Particle {
    type Err = Box<dyn error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pid: usize = 0;
        let mut mass: f64 = 0.0;
        let mut charge: f64 = 0.0;
        let mut position = ThreeVec::new(0.0,0.0,0.0);
        let mut velocity = ThreeVec::new(0.0,0.0,0.0);
        for (i,chunk) in s.split_terminator(",\"r\":").enumerate() {
            match i {
                1 => {
                    for (j,vec) in chunk.replace("}}","}!").trim_matches(|p| p == '!').split_terminator(",\"v\":").enumerate() {
                        match j {
                            0 => position = ThreeVec::from_json(vec)?,
                            1 => velocity = ThreeVec::from_json(vec)?,
                            _ => (),
                        }
                    }
                },
                0 => {
                    for (_j,ky_vl) in chunk.trim_matches(|p| p == '{').split_terminator(",").enumerate() {
                        let n_v: Vec<&str> = ky_vl.split(":").collect();
                        match n_v[0] {
                            "\"pid\"" => pid = n_v[1].parse::<usize>()?,
                            "\"m\"" => mass = n_v[1].parse::<f64>()?,
                            "\"q\"" => charge = n_v[1].parse::<f64>()?,
                            x => panic!("Unexpected invalid token {:?}", x),
                        }
                    }
                },
                _ => (),
            }
        }

        Ok(Particle{pid,mass,charge,position,velocity,t_force:ThreeVec::new(0.0,0.0,0.0)})
    }
}

impl Universe {
    pub fn cloud(size: f64, n: usize, dt: f64) -> Universe {
        let energy = 0.0;
        let mut state: Vec<Particle> = Vec::new();
        for i in 0..n{
            let part: Particle = Particle::dust(i, size);
            state.push(part);
        }
        let previous_state = state.clone().to_vec();
        Universe {
            dt,
            energy,
            state,
            previous_state,
        }
    }

    pub fn run(&mut self, t: usize) {
        let mut cur_state = self.state.clone();
        // let pool = ThreadPool::new(p);
        // let (tx,rx) = channel();
        println!("Start run");
        for ti in 0..t {
            if ti%100 == 0 {println!("timestamp: {}", ti);}
            let prev_state = self.previous_state.clone();
            for i in 0..self.state.len() as usize{
                cur_state[i].force(&prev_state);
                let diff = (*cur_state[i].v()*self.dt)+(*cur_state[i].f()*0.5*self.dt*self.dt);
                cur_state[i].translate(&diff);
            }
            let lo_state = cur_state.clone();
            for i in 0..self.state.len() as usize{
                let pre_force = *cur_state[i].f();
                cur_state[i].force(&lo_state);
                let diff = (*cur_state[i].f() + pre_force)*0.5*self.dt;
                cur_state[i].accelerate(&diff);
            }
            self.previous_state = cur_state.clone();
        }
        self.state = cur_state;
    }
}
