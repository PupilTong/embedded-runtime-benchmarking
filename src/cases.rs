use std::collections::{BTreeMap, BTreeSet, VecDeque};

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
    let mut queues: Vec<VecDeque<Packet>> = (0..TASKS)
        .map(|task| {
            let mut queue = VecDeque::new();
            queue.push_back(Packet {
                target: (task + 1) % TASKS,
                value: mix(task as u64, 1),
            });
            queue
        })
        .collect();
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
struct Constraint {
    left: usize,
    right: usize,
    output: usize,
    scale: f64,
    bias: f64,
}

pub(crate) fn delta_blue(iterations: u64) -> u64 {
    let mut values: Vec<f64> = (0..96).map(|index| (index as f64 + 1.0) * 0.25).collect();
    let constraints: Vec<Constraint> = (0..192)
        .map(|index| Constraint {
            left: index % values.len(),
            right: (index * 7 + 13) % values.len(),
            output: (index * 11 + 17) % values.len(),
            scale: 0.875 + (index % 9) as f64 * 0.03125,
            bias: (index % 5) as f64 - 2.0,
        })
        .collect();

    for step in 0..iterations {
        let anchor = step as usize % values.len();
        values[0] = (step as f64 + 1.0) * 0.125;
        values[anchor] = values[anchor].mul_add(0.5, step as f64 * 0.015625);

        for _ in 0..3 {
            for constraint in &constraints {
                let propagated = values[constraint.left].mul_add(
                    constraint.scale,
                    values[constraint.right] * 0.125 + constraint.bias,
                );
                values[constraint.output] = propagated.mul_add(0.999, constraint.bias * 0.001);
            }
        }
    }

    values
        .iter()
        .enumerate()
        .fold(0_u64, |acc, (index, value)| {
            mix(acc, value.to_bits() ^ index as u64)
        })
}

pub(crate) fn crypto(iterations: u64) -> u64 {
    let mut data: Vec<u8> = (0..4096)
        .map(|index| mix(index, 0x51ed_270b).to_le_bytes()[0])
        .collect();
    let mut state = 0x243f_6a88_85a3_08d3_u64;

    for round in 0..iterations.saturating_mul(12) {
        for chunk in data.chunks_exact_mut(8) {
            let mut bytes = [0_u8; 8];
            bytes.copy_from_slice(chunk);
            let word = u64::from_le_bytes(bytes);
            state = mix(state.rotate_left(9), word ^ round);
            chunk.copy_from_slice(&state.to_le_bytes());
        }

        let rotation = (round as usize % (data.len() - 1)) + 1;
        data.rotate_left(rotation);
    }

    data.chunks_exact(8)
        .enumerate()
        .fold(state, |acc, (index, chunk)| {
            let mut bytes = [0_u8; 8];
            bytes.copy_from_slice(chunk);
            mix(acc ^ index as u64, u64::from_le_bytes(bytes))
        })
}

#[derive(Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }

    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }

    fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
    }

    fn dot(self, rhs: Self) -> f64 {
        self.x.mul_add(rhs.x, self.y.mul_add(rhs.y, self.z * rhs.z))
    }

    fn normalize(self) -> Self {
        let length = self.dot(self).sqrt();
        if length == 0.0 {
            self
        } else {
            self.scale(1.0 / length)
        }
    }
}

struct Sphere {
    center: Vec3,
    radius: f64,
    albedo: f64,
}

pub(crate) fn ray_trace(iterations: u64) -> u64 {
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

    let mut checksum = 0_u64;
    for frame in 0..iterations {
        let camera = Vec3::new((frame as f64 * 0.011).sin() * 0.25, 0.15, -2.75);
        for y in 0..28 {
            for x in 0..36 {
                let mut origin = camera;
                let mut direction =
                    Vec3::new((x as f64 - 18.0) / 22.0, (14.0 - y as f64) / 22.0, 1.0).normalize();
                let mut light = 0.0;
                let mut throughput = 1.0;

                for bounce in 0..2 {
                    let mut closest: Option<(f64, &Sphere)> = None;
                    for sphere in &spheres {
                        if let Some(distance) = intersect_sphere(origin, direction, sphere) {
                            if closest.map_or(true, |(best, _)| distance < best) {
                                closest = Some((distance, sphere));
                            }
                        }
                    }

                    let Some((distance, sphere)) = closest else {
                        light += throughput * (0.2 + 0.03 * y as f64);
                        break;
                    };

                    let hit = origin.add(direction.scale(distance));
                    let normal = hit.sub(sphere.center).normalize();
                    let lambert = normal.dot(Vec3::new(-0.4, 0.9, -0.3).normalize()).max(0.0);
                    light += throughput * sphere.albedo * lambert;
                    throughput *= 0.42 + bounce as f64 * 0.08;
                    origin = hit.add(normal.scale(0.001));
                    direction = direction
                        .sub(normal.scale(2.0 * direction.dot(normal)))
                        .normalize();
                }

                checksum = mix(
                    checksum,
                    light.to_bits() ^ ((frame << 16) + (y * 36 + x) as u64),
                );
            }
        }
    }

    checksum
}

fn intersect_sphere(origin: Vec3, direction: Vec3, sphere: &Sphere) -> Option<f64> {
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

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct EarleyState {
    rule: usize,
    dot: usize,
    origin: usize,
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
    let mut chart: Vec<BTreeSet<EarleyState>> =
        (0..=tokens.len()).map(|_| BTreeSet::new()).collect();
    chart[0].insert(EarleyState {
        rule: 0,
        dot: 0,
        origin: 0,
    });

    for index in 0..=tokens.len() {
        loop {
            let before = chart[index].len();
            let states: Vec<EarleyState> = chart[index].iter().copied().collect();

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
                        let origin_states: Vec<EarleyState> =
                            chart[state.origin].iter().copied().collect();
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
    let total_states = chart.iter().map(BTreeSet::len).sum();
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
    const PATTERNS: &[&str] = &[
        "agggtaaa", "tttaccct", "cgggtaaa", "gggtaaat", "taaaacc", "gtaac",
    ];
    let alphabet = [b'a', b'c', b'g', b't'];
    let mut dna = String::with_capacity(8192);
    for index in 0..8192 {
        dna.push(alphabet[(mix(index, 17) & 3) as usize] as char);
    }

    let mut checksum = 0_u64;
    for iteration in 0..iterations {
        if iteration % 8 == 0 {
            dna = dna.chars().rev().collect();
        }

        for pattern in PATTERNS {
            checksum = mix(checksum, count_overlapping(&dna, pattern) as u64);
        }

        let gc = dna
            .bytes()
            .filter(|byte| matches!(byte, b'g' | b'c'))
            .count();
        let runs = dna.split('a').filter(|segment| !segment.is_empty()).count();
        checksum = mix(checksum ^ gc as u64, runs as u64 ^ iteration);
    }

    checksum
}

fn count_overlapping(haystack: &str, needle: &str) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(position) = haystack[start..].find(needle) {
        count += 1;
        start += position + 1;
    }
    count
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
    let mut velocity_x = vec![0.0_f64; size];
    let mut velocity_y = vec![0.0_f64; size];

    let idx = |x: usize, y: usize| y * (N + 2) + x;
    for step in 0..iterations {
        let center_x = 8 + (step as usize % 17);
        let center_y = 8 + ((step as usize * 5) % 17);
        let center = idx(center_x, center_y);
        density[center] += 24.0 + (step % 7) as f64;
        velocity_x[center] += (step as f64 * 0.13).sin() * 0.7;
        velocity_y[center] += (step as f64 * 0.17).cos() * 0.7;

        for _ in 0..4 {
            for y in 1..=N {
                for x in 1..=N {
                    let current = idx(x, y);
                    density[current] = (density[current]
                        + density[idx(x - 1, y)]
                        + density[idx(x + 1, y)]
                        + density[idx(x, y - 1)]
                        + density[idx(x, y + 1)])
                        * 0.2;
                    velocity_x[current] = (velocity_x[current]
                        + velocity_x[idx(x - 1, y)]
                        + velocity_x[idx(x + 1, y)])
                        / 3.0;
                    velocity_y[current] = (velocity_y[current]
                        + velocity_y[idx(x, y - 1)]
                        + velocity_y[idx(x, y + 1)])
                        / 3.0;
                }
            }
        }

        let previous = density.clone();
        for y in 1..=N {
            for x in 1..=N {
                let current = idx(x, y);
                let source_x = (x as f64 - velocity_x[current]).clamp(1.0, N as f64);
                let source_y = (y as f64 - velocity_y[current]).clamp(1.0, N as f64);
                density[current] = bilinear_sample(&previous, N, source_x, source_y);
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

fn bilinear_sample(field: &[f64], n: usize, x: f64, y: f64) -> f64 {
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(n);
    let y1 = (y0 + 1).min(n);
    let sx = x - x0 as f64;
    let sy = y - y0 as f64;
    let idx = |px: usize, py: usize| py * (n + 2) + px;

    let top = field[idx(x0, y0)].mul_add(1.0 - sx, field[idx(x1, y0)] * sx);
    let bottom = field[idx(x0, y1)].mul_add(1.0 - sx, field[idx(x1, y1)] * sx);
    top.mul_add(1.0 - sy, bottom * sy)
}
