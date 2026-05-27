use std::array;
use std::collections::{BTreeMap, VecDeque};

pub struct Case {
    pub name: &'static str,
    pub default_iterations: u64,
    pub run: fn(u64) -> u64,
}

pub const CASES: &[Case] = &[
    Case {
        name: "Richards",
        default_iterations: 2_400,
        run: richards,
    },
    Case {
        name: "DeltaBlue",
        default_iterations: 700,
        run: delta_blue,
    },
    Case {
        name: "Crypto",
        default_iterations: 160,
        run: crypto,
    },
    Case {
        name: "RayTrace",
        default_iterations: 26,
        run: ray_trace,
    },
    Case {
        name: "EarleyBoyer",
        default_iterations: 36,
        run: earley_boyer,
    },
    Case {
        name: "RegExp",
        default_iterations: 180,
        run: regexp,
    },
    Case {
        name: "Splay",
        default_iterations: 520,
        run: splay,
    },
    Case {
        name: "NavierStokes",
        default_iterations: 34,
        run: navier_stokes,
    },
];

pub(crate) fn mix(left: u64, right: u64) -> u64 {
    let mut x = left ^ right.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    x ^ (x >> 33)
}

#[derive(Clone, Copy)]
struct Packet {
    target: usize,
    value: u64,
}

pub(crate) fn richards(iterations: u64) -> u64 {
    const TASKS: usize = 6;
    let mut queues: [VecDeque<Packet>; TASKS] = array::from_fn(|task| {
        let mut queue = VecDeque::new();
        queue.push_back(Packet {
            target: (task + 1) % TASKS,
            value: mix(task as u64, 1),
        });
        queue
    });
    let mut state = [0_u64; TASKS];

    for tick in 0..iterations {
        for task in 0..TASKS {
            queues[task].push_back(Packet {
                target: ((task * 3) + tick as usize + 1) % TASKS,
                value: mix(tick, task as u64),
            });

            let budget = 3 + ((tick as usize + task) % 5);
            for _ in 0..budget {
                let Some(packet) = queues[task].pop_front() else {
                    break;
                };

                let value = mix(packet.value ^ state[task], tick + task as u64);
                state[task] = state[task].rotate_left(7) ^ value;

                let next = (packet.target + (value as usize & 3) + 1) % TASKS;
                if next != task {
                    queues[next].push_back(Packet {
                        target: task,
                        value,
                    });
                }
            }
        }
    }

    queues.iter().enumerate().fold(0_u64, |acc, (task, queue)| {
        let queue_sum = queue
            .iter()
            .take(16)
            .fold(queue.len() as u64, |sum, packet| sum ^ packet.value);
        mix(acc ^ state[task], queue_sum)
    })
}

#[derive(Clone, Copy)]
enum DbConstraint {
    Equality {
        source: usize,
        target: usize,
    },
    Scale {
        source: usize,
        target: usize,
        scale: f64,
        offset: f64,
    },
}

impl DbConstraint {
    #[inline]
    fn execute(self, values: &mut [f64]) {
        match self {
            Self::Equality { source, target } => {
                values[target] = values[source];
            }
            Self::Scale {
                source,
                target,
                scale,
                offset,
            } => {
                values[target] = values[source].mul_add(scale, offset);
            }
        }
    }
}

pub(crate) fn delta_blue(iterations: u64) -> u64 {
    const CHAIN: usize = 96;
    const PROJECTIONS: usize = 64;
    const VALUE_COUNT: usize = CHAIN + PROJECTIONS * 2;

    let mut values: [f64; VALUE_COUNT] = array::from_fn(|index| (index as f64 + 1.0) * 0.25);
    let chain: [DbConstraint; CHAIN - 1] = array::from_fn(|index| DbConstraint::Equality {
        source: index,
        target: index + 1,
    });
    let projections: [DbConstraint; PROJECTIONS] = array::from_fn(|index| {
        let source = CHAIN + index;
        DbConstraint::Scale {
            source,
            target: CHAIN + PROJECTIONS + index,
            scale: 0.875 + (index % 9) as f64 * 0.03125,
            offset: (index % 5) as f64 - 2.0,
        }
    });

    let mut checksum = 0_u64;

    for step in 0..iterations {
        values[0] = (step as f64 + 1.0) * 0.125;
        for &constraint in &chain {
            constraint.execute(&mut values);
        }

        let projection_seed = step as usize % PROJECTIONS;
        for index in 0..PROJECTIONS {
            let source = CHAIN + index;
            values[source] = values[source].mul_add(
                0.5,
                values[(projection_seed + index) % CHAIN] * 0.25 + step as f64 * 0.015625,
            );
        }
        for &constraint in &projections {
            constraint.execute(&mut values);
        }

        let changed = CHAIN + PROJECTIONS + projection_seed;
        values[CHAIN + projection_seed] =
            values[changed].mul_add(0.125, values[projection_seed] * 0.875);

        checksum = mix(
            checksum,
            values[CHAIN - 1].to_bits()
                ^ values[CHAIN + PROJECTIONS + ((step as usize * 13) % PROJECTIONS)].to_bits()
                ^ step,
        );
    }

    values
        .iter()
        .enumerate()
        .fold(checksum, |acc, (index, value)| {
            mix(acc, value.to_bits() ^ index as u64)
        })
}

pub(crate) fn crypto(iterations: u64) -> u64 {
    const WORDS: usize = 512;
    let mut data: [u64; WORDS] = array::from_fn(|index| mix(index as u64, 0x51ed_270b));
    let mut state = 0x243f_6a88_85a3_08d3_u64;

    for round in 0..iterations.saturating_mul(12) {
        for word in &mut data {
            state = mix(state.rotate_left(9), *word ^ round);
            *word = state;
        }

        let rotation = (round as usize % (WORDS - 1)) + 1;
        data.rotate_left(rotation);
    }

    data.iter()
        .enumerate()
        .fold(state, |acc, (index, word)| mix(acc ^ index as u64, *word))
}

#[derive(Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    #[inline]
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }

    #[inline]
    fn scale(self, factor: f32) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
    }

    #[inline]
    fn dot(self, rhs: Self) -> f32 {
        self.x.mul_add(rhs.x, self.y.mul_add(rhs.y, self.z * rhs.z))
    }

    #[inline]
    fn normalize(self) -> Self {
        let length = self.dot(self).sqrt();
        if length == 0.0 {
            self
        } else {
            self.scale(1.0 / length)
        }
    }
}

#[derive(Clone, Copy)]
struct Sphere {
    center: Vec3,
    radius: f32,
    albedo: f32,
}

pub(crate) fn ray_trace(iterations: u64) -> u64 {
    const WIDTH: usize = 36;
    const HEIGHT: usize = 28;

    let spheres = [
        Sphere {
            center: Vec3::new(-1.25, -0.2, 3.6),
            radius: 0.7,
            albedo: 0.85,
        },
        Sphere {
            center: Vec3::new(0.85, 0.0, 3.0),
            radius: 0.55,
            albedo: 0.75,
        },
        Sphere {
            center: Vec3::new(0.15, -0.8, 4.25),
            radius: 0.9,
            albedo: 0.65,
        },
        Sphere {
            center: Vec3::new(1.6, 0.35, 4.6),
            radius: 0.5,
            albedo: 0.9,
        },
    ];
    let rays: [Vec3; WIDTH * HEIGHT] = array::from_fn(|pixel| {
        let x = pixel % WIDTH;
        let y = pixel / WIDTH;
        Vec3::new((x as f32 - 18.0) / 22.0, (14.0 - y as f32) / 22.0, 1.0).normalize()
    });
    let light_direction = Vec3::new(-0.4, 0.9, -0.3).normalize();

    let mut checksum = 0_u64;
    for frame in 0..iterations {
        let camera = Vec3::new((frame as f32 * 0.011).sin() * 0.25, 0.15, -2.75);
        for y in 0..HEIGHT {
            let sky_light = 0.2 + 0.03 * y as f32;
            for x in 0..WIDTH {
                let pixel = y * WIDTH + x;
                let mut origin = camera;
                let mut direction = rays[pixel];
                let mut light = 0.0_f32;
                let mut throughput = 1.0_f32;

                for bounce in 0..2 {
                    let mut closest_distance = f32::INFINITY;
                    let mut closest_sphere = None;
                    for (sphere_index, sphere) in spheres.iter().enumerate() {
                        if let Some(distance) = intersect_sphere(origin, direction, sphere) {
                            if distance < closest_distance {
                                closest_distance = distance;
                                closest_sphere = Some(sphere_index);
                            }
                        }
                    }

                    let Some(sphere_index) = closest_sphere else {
                        light += throughput * sky_light;
                        break;
                    };
                    let sphere = spheres[sphere_index];

                    let hit = origin.add(direction.scale(closest_distance));
                    let normal = hit.sub(sphere.center).normalize();
                    let lambert = normal.dot(light_direction).max(0.0);
                    light += throughput * sphere.albedo * lambert;
                    throughput *= 0.42 + bounce as f32 * 0.08;
                    origin = hit.add(normal.scale(0.001));
                    direction = direction
                        .sub(normal.scale(2.0 * direction.dot(normal)))
                        .normalize();
                }

                checksum = mix(
                    checksum,
                    ((light * 1_000_003.0) as u64)
                        ^ (frame * (WIDTH * HEIGHT) as u64 + pixel as u64),
                );
            }
        }
    }

    checksum
}

#[inline]
fn intersect_sphere(origin: Vec3, direction: Vec3, sphere: &Sphere) -> Option<f32> {
    let oc = origin.sub(sphere.center);
    let b = oc.dot(direction);
    let c = oc.dot(oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return None;
    }

    let distance = -b - discriminant.sqrt();
    (distance > 0.001).then_some(distance)
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct EarleyState {
    rule: usize,
    dot: usize,
    origin: usize,
}

#[derive(Clone)]
struct StateSet {
    states: Vec<EarleyState>,
}

impl StateSet {
    fn new() -> Self {
        Self { states: Vec::new() }
    }

    fn insert(&mut self, state: EarleyState) -> bool {
        if self.states.contains(&state) {
            false
        } else {
            self.states.push(state);
            true
        }
    }

    fn contains(&self, state: &EarleyState) -> bool {
        self.states.contains(state)
    }

    fn len(&self) -> usize {
        self.states.len()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Symbol {
    NonTerm(u8),
    Term(u8),
}

struct Rule {
    lhs: u8,
    rhs: &'static [Symbol],
}

const NT_START: u8 = 0;
const NT_EXPR: u8 = 1;
const NT_TERM: u8 = 2;
const NT_FACTOR: u8 = 3;
const TOK_NUM: u8 = 1;
const TOK_PLUS: u8 = 2;
const TOK_STAR: u8 = 3;
const TOK_LPAREN: u8 = 4;
const TOK_RPAREN: u8 = 5;

const RHS_START: &[Symbol] = &[Symbol::NonTerm(NT_EXPR)];
const RHS_EXPR_PLUS: &[Symbol] = &[
    Symbol::NonTerm(NT_EXPR),
    Symbol::Term(TOK_PLUS),
    Symbol::NonTerm(NT_TERM),
];
const RHS_EXPR_TERM: &[Symbol] = &[Symbol::NonTerm(NT_TERM)];
const RHS_TERM_STAR: &[Symbol] = &[
    Symbol::NonTerm(NT_TERM),
    Symbol::Term(TOK_STAR),
    Symbol::NonTerm(NT_FACTOR),
];
const RHS_TERM_FACTOR: &[Symbol] = &[Symbol::NonTerm(NT_FACTOR)];
const RHS_FACTOR_GROUP: &[Symbol] = &[
    Symbol::Term(TOK_LPAREN),
    Symbol::NonTerm(NT_EXPR),
    Symbol::Term(TOK_RPAREN),
];
const RHS_FACTOR_NUM: &[Symbol] = &[Symbol::Term(TOK_NUM)];

const GRAMMAR: &[Rule] = &[
    Rule {
        lhs: NT_START,
        rhs: RHS_START,
    },
    Rule {
        lhs: NT_EXPR,
        rhs: RHS_EXPR_PLUS,
    },
    Rule {
        lhs: NT_EXPR,
        rhs: RHS_EXPR_TERM,
    },
    Rule {
        lhs: NT_TERM,
        rhs: RHS_TERM_STAR,
    },
    Rule {
        lhs: NT_TERM,
        rhs: RHS_TERM_FACTOR,
    },
    Rule {
        lhs: NT_FACTOR,
        rhs: RHS_FACTOR_GROUP,
    },
    Rule {
        lhs: NT_FACTOR,
        rhs: RHS_FACTOR_NUM,
    },
];

pub(crate) fn earley_boyer(iterations: u64) -> u64 {
    let mut checksum = 0_u64;
    for seed in 0..iterations {
        let terms = 8 + (seed as usize % 5);
        let tokens = expression_tokens(seed, terms);
        let (accepted, states) = earley_parse(&tokens);
        let rewrite_sum = boyer_rewrite(seed);
        checksum = mix(
            checksum ^ rewrite_sum,
            states as u64 ^ u64::from(accepted) ^ tokens.len() as u64,
        );
    }
    checksum
}

fn expression_tokens(seed: u64, terms: usize) -> Vec<u8> {
    let mut tokens = Vec::with_capacity(terms * 2 - 1);
    for term in 0..terms {
        if term % 4 == 0 && term + 1 < terms {
            tokens.extend([TOK_LPAREN, TOK_NUM, TOK_PLUS, TOK_NUM, TOK_RPAREN]);
        } else {
            tokens.push(TOK_NUM);
        }

        if term + 1 < terms {
            tokens.push(if (term as u64 + seed) % 3 == 0 {
                TOK_STAR
            } else {
                TOK_PLUS
            });
        }
    }
    tokens
}

fn earley_parse(tokens: &[u8]) -> (bool, usize) {
    let mut chart: Vec<StateSet> = (0..=tokens.len()).map(|_| StateSet::new()).collect();
    chart[0].insert(EarleyState {
        rule: 0,
        dot: 0,
        origin: 0,
    });

    for index in 0..=tokens.len() {
        loop {
            let before = chart[index].len();
            let states = chart[index].states.clone();

            for state in states {
                match GRAMMAR[state.rule].rhs.get(state.dot) {
                    Some(Symbol::NonTerm(non_terminal)) => {
                        for (rule_index, rule) in GRAMMAR.iter().enumerate() {
                            if rule.lhs == *non_terminal {
                                chart[index].insert(EarleyState {
                                    rule: rule_index,
                                    dot: 0,
                                    origin: index,
                                });
                            }
                        }
                    }
                    Some(Symbol::Term(token)) => {
                        if tokens.get(index) == Some(token) {
                            chart[index + 1].insert(EarleyState {
                                rule: state.rule,
                                dot: state.dot + 1,
                                origin: state.origin,
                            });
                        }
                    }
                    None => {
                        let completed_lhs = GRAMMAR[state.rule].lhs;
                        let origin_states = chart[state.origin].states.clone();
                        for previous in origin_states {
                            if GRAMMAR[previous.rule].rhs.get(previous.dot)
                                == Some(&Symbol::NonTerm(completed_lhs))
                            {
                                chart[index].insert(EarleyState {
                                    rule: previous.rule,
                                    dot: previous.dot + 1,
                                    origin: previous.origin,
                                });
                            }
                        }
                    }
                }
            }

            if chart[index].len() == before {
                break;
            }
        }
    }

    let accepted = chart[tokens.len()].contains(&EarleyState {
        rule: 0,
        dot: 1,
        origin: 0,
    });
    let total_states = chart.iter().map(StateSet::len).sum();
    (accepted, total_states)
}

fn boyer_rewrite(seed: u64) -> u64 {
    let mut terms: Vec<(u64, u64, u64)> = (0..48)
        .map(|index| {
            let lhs = mix(seed, index);
            let rhs = mix(index, seed ^ 0x0bad_f00d);
            (lhs & 31, rhs & 31, mix(lhs, rhs))
        })
        .collect();

    for round in 0..16 {
        terms.sort_unstable_by_key(|term| (term.0, term.1, term.2));
        terms.dedup_by_key(|term| (term.0, term.1));
        for term in &mut terms {
            if term.0 == term.1 {
                term.2 = mix(term.2, round);
            } else {
                term.0 = (term.0 + term.2 + round) & 31;
                term.1 = (term.1 ^ (term.2 >> 3)) & 31;
                term.2 = mix(term.2, term.0 ^ term.1 ^ round);
            }
        }
        while terms.len() < 48 {
            let next = mix(seed ^ round, terms.len() as u64);
            terms.push((next & 31, (next >> 7) & 31, next));
        }
    }

    terms.iter().fold(seed, |acc, (left, right, value)| {
        mix(acc ^ left, right ^ value)
    })
}

pub(crate) fn regexp(iterations: u64) -> u64 {
    const PATTERNS: &[&[u8]] = &[
        b"agggtaaa",
        b"tttaccct",
        b"cgggtaaa",
        b"gggtaaat",
        b"taaaacc",
        b"gtaac",
    ];
    let alphabet = [b'a', b'c', b'g', b't'];
    let mut dna: Vec<u8> = (0..8192)
        .map(|index| alphabet[(mix(index, 17) & 3) as usize])
        .collect();

    let mut checksum = 0_u64;
    for iteration in 0..iterations {
        if iteration % 8 == 0 {
            dna.reverse();
        }

        for pattern in PATTERNS {
            checksum = mix(checksum, count_overlapping(&dna, pattern) as u64);
        }

        let mut gc = 0_usize;
        let mut runs = 0_usize;
        let mut in_run = false;
        for &byte in &dna {
            gc += usize::from(matches!(byte, b'g' | b'c'));
            if byte == b'a' {
                in_run = false;
            } else if !in_run {
                runs += 1;
                in_run = true;
            }
        }
        checksum = mix(checksum ^ gc as u64, runs as u64 ^ iteration);
    }

    checksum
}

fn count_overlapping(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|window| *window == needle)
        .count()
}

pub(crate) fn splay(iterations: u64) -> u64 {
    let mut map = BTreeMap::new();
    for key in 0..1024_u64 {
        map.insert(mix(key, 0xfeed_face) & 4095, mix(key, 0x1234_5678));
    }

    let mut checksum = 0_u64;
    for step in 0..iterations.saturating_mul(24) {
        let key = mix(step, checksum) & 4095;
        map.insert(key, mix(key, step));

        if step % 3 == 0 {
            map.remove(&(mix(step, 7) & 4095));
        }

        if step % 8 == 0 {
            let lower = key.saturating_sub(24);
            let upper = key.saturating_add(24);
            for (range_key, value) in map.range(lower..=upper).take(12) {
                checksum = mix(checksum ^ range_key, *value);
            }
        }
    }

    map.iter()
        .take(256)
        .fold(mix(checksum, map.len() as u64), |acc, (key, value)| {
            mix(acc ^ key, *value)
        })
}

pub(crate) fn navier_stokes(iterations: u64) -> u64 {
    const N: usize = 32;
    let size = (N + 2) * (N + 2);
    let mut density = vec![0.0_f64; size];
    let mut previous_density = vec![0.0_f64; size];
    let mut velocity_x = vec![0.0_f64; size];
    let mut velocity_y = vec![0.0_f64; size];

    let stride = N + 2;
    for step in 0..iterations {
        let center_x = 8 + (step as usize % 17);
        let center_y = 8 + ((step as usize * 5) % 17);
        let center = grid_index(stride, center_x, center_y);
        density[center] += 24.0 + (step % 7) as f64;
        velocity_x[center] += (step as f64 * 0.13).sin() * 0.7;
        velocity_y[center] += (step as f64 * 0.17).cos() * 0.7;

        for _ in 0..4 {
            for y in 1..=N {
                let row = y * stride;
                for x in 1..=N {
                    let current = row + x;
                    density[current] = (density[current]
                        + density[current - 1]
                        + density[current + 1]
                        + density[current - stride]
                        + density[current + stride])
                        * 0.2;
                    velocity_x[current] =
                        (velocity_x[current] + velocity_x[current - 1] + velocity_x[current + 1])
                            / 3.0;
                    velocity_y[current] = (velocity_y[current]
                        + velocity_y[current - stride]
                        + velocity_y[current + stride])
                        / 3.0;
                }
            }
        }

        previous_density.copy_from_slice(&density);
        for y in 1..=N {
            let row = y * stride;
            for x in 1..=N {
                let current = row + x;
                let source_x = (x as f64 - velocity_x[current]).clamp(1.0, N as f64);
                let source_y = (y as f64 - velocity_y[current]).clamp(1.0, N as f64);
                density[current] = bilinear_sample(&previous_density, N, source_x, source_y);
            }
        }
    }

    density
        .iter()
        .enumerate()
        .fold(0_u64, |acc, (index, value)| {
            mix(acc ^ index as u64, value.to_bits())
        })
}

#[inline]
fn grid_index(stride: usize, x: usize, y: usize) -> usize {
    y * stride + x
}

fn bilinear_sample(field: &[f64], n: usize, x: f64, y: f64) -> f64 {
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(n);
    let y1 = (y0 + 1).min(n);
    let sx = x - x0 as f64;
    let sy = y - y0 as f64;
    let stride = n + 2;

    let top =
        field[grid_index(stride, x0, y0)].mul_add(1.0 - sx, field[grid_index(stride, x1, y0)] * sx);
    let bottom =
        field[grid_index(stride, x0, y1)].mul_add(1.0 - sx, field[grid_index(stride, x1, y1)] * sx);
    top.mul_add(1.0 - sy, bottom * sy)
}
