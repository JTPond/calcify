extern crate threadpool;
use threadpool::ThreadPool;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;

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
    pub fn new(pid: usize, mass: f64, radius: f64, charge: f64,
                position: ThreeVec, velocity: ThreeVec) -> Particle {
        Particle {
            pid,
            mass,
            radius,
            charge,
            position,
            velocity,
            t_force: ThreeVec::new(0.0,0.0,0.0),
        }
    }

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

    pub fn force(&mut self, part2: &Particle) {
        let g: f64 = 6.67408e-11;
        let rmrp = self.position - *part2.r();
        let r2 = rmrp*rmrp;
        self.t_force += (rmrp*(1.0/r2.sqrt()))*((-g*self.mass*part2.m())*(1.0/r2));
    }

    pub fn zero_force(&mut self) {
        self.t_force = ThreeVec::new(0.0,0.0,0.0);
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
    pub fn new(state: Vec<Particle>, dt: f64) -> Universe {
        let energy = 0.0;
        let previous_state = state.clone().to_vec();
        Universe {
            dt,
            energy,
            state,
            previous_state,
        }
    }

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

    pub fn run(&mut self, t: usize, p: usize) {
        let cur_state = Arc::new(Mutex::new(self.state.clone()));
        let pool = ThreadPool::new(p);
        let (tx,rx) = channel();
        for _ in 0..t {
            let part_len = self.previous_state.len();
            let prev_state = Arc::new(self.previous_state.clone());
            let mut n_jobs = 0;
            for i in 0..self.state.len() as usize{
                n_jobs+=1;
                let tx = tx.clone();
                let dt = self.dt;
                let cur_state = cur_state.clone();
                let prev_state = prev_state.clone();
                pool.execute(move|| {
                    let mut cur_state = cur_state.lock().unwrap();
                    cur_state[i].zero_force();
                    for j in 0..part_len{
                        if cur_state[i].pid() != prev_state[j].pid() {
                            cur_state[i].force(&prev_state[j]);
                        }
                    }
                    let diff = (*cur_state[i].v()*dt)+(*cur_state[i].f()*0.5*dt*dt);
                    cur_state[i].translate(&diff);
                    tx.send(1).expect("channel will be there waiting for the pool");
                })
            }
            assert_eq!(rx.iter().take(n_jobs).fold(0, |a, b| a + b),n_jobs);
            n_jobs = 0;
            for i in 0..self.state.len() as usize{
                n_jobs+=1;
                let tx = tx.clone();
                let dt = self.dt;
                let cur_state = cur_state.clone();
                pool.execute(move|| {
                    let mut cur_state = cur_state.lock().unwrap();
                    let pre_force = *cur_state[i].f();
                    cur_state[i].zero_force();
                    for j in 0..part_len{
                        if cur_state[i].pid() != cur_state[j].pid() {
                            let now_state = cur_state[j];
                            cur_state[i].force(&now_state);
                        }
                    }
                    let diff = (*cur_state[i].f() + pre_force)*0.5*dt;
                    cur_state[i].accelerate(&diff);
                    tx.send(1).expect("channel will be there waiting for the pool");
                })
            }
            assert_eq!(rx.iter().take(n_jobs).fold(0, |a, b| a + b),n_jobs);
            let temp_state = cur_state.clone();
            self.previous_state = temp_state.lock().unwrap().clone();
        }
        self.state = Arc::try_unwrap(cur_state).unwrap().into_inner().unwrap();
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
