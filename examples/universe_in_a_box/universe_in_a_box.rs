use std::fmt;

extern crate rayon;
use rayon::prelude::*;

extern crate rmp;
use rmp::encode::*;

extern crate calcify;
pub use calcify::ThreeVec;
pub use calcify::Serializable;


#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Particle {
    pid: usize,
    mass: f64,
    radius: f64,
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
        let radius = 1.0;
        let charge = 0.0;
        let position = ThreeVec::random(max);
        let velocity = ThreeVec::new(0.0,0.0,0.0);
        let t_force = ThreeVec::new(0.0,0.0,0.0);
        Particle {
            pid,
            mass,
            radius,
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
        self.r().to_json()
    }
    fn to_jsonc(&self) -> String {
        self.r().to_jsonc()
    }
    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        self.r().to_msg()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access() {
        let test_vec1 = ThreeVec::new(1.0,2.0,3.0);
        assert_eq!(*test_vec1.x1(),2.0)
    }

    #[test]
    fn forcer() {
        let mut universe = Universe::new(
            vec![
                Particle::new(0,1.0,1.0,0.0,
                    ThreeVec::new(6.0,8.0,24.0),ThreeVec::new(1.0,2.0,3.0)),
                Particle::new(1,1.0,1.0,0.0,
                    ThreeVec::new(3.0,4.0,12.0),ThreeVec::new(1.0,2.0,3.0))
            ],
            0.1
        );
        let op = universe.state[1];
        universe.state[0].force(&op);
        assert_eq!(*universe.state[0].f(),
         ThreeVec::new(3.0/13.0,4.0/13.0,12.0/13.0)*(-6.67408e-11/(13.0*13.0)))
    }
}
