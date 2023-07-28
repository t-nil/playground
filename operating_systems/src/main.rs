#![feature(generators)]
#![feature(iter_from_generator)]
#![feature(iter_collect_into)]
#![feature(drain_filter)]
#![feature(trait_alias)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(associated_type_bounds)]

use std::fmt::Debug;
use std::{iter, vec};

use dyn_clone::DynClone;
use itertools::Itertools;

use crate::cap03_scheduling::*;

pub mod cap03_scheduling;

macro_rules! funs_to_tuple {
    ($($f:ident),+) => {
        {vec![
            $((stringify!($f), $f()),)*
        ]}
    };
}
fn main() {
    let all = funs_to_tuple![klausur_stappert_ss23_probeklausur, klausur_stappert_ss15];
    for (name, problems) in all {
        println!("\n# {}\n\n", name);
        problems.iter().for_each(Problem::eval);
    }
    cap03_scheduling::test_round_robin();
    cap03_scheduling::test_rate_monotonic();
}

// https://github.com/dtolnay/dyn-clone
// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
trait ProblemData: Debug + DynClone {}
impl<T: Debug + Clone> ProblemData for T {}

trait SolutionGenerator: DynClone + (Fn() -> Box<dyn ProblemData>) {}
impl<T: (Fn() -> Box<dyn ProblemData>) + Clone> SolutionGenerator for T {}

dyn_clone::clone_trait_object!(ProblemData);
dyn_clone::clone_trait_object!(SolutionGenerator);

#[derive(Clone)]
struct Problem {
    which: String,
    algo: Box<dyn SolutionGenerator>,
    solution: Box<dyn ProblemData>,
}

// trait Aufgabe_t {
//     fn get_which(&self) -> &str;
//     fn get_algo(&self) -> &Box<dyn Fn() -> Box<dyn AufgabeData>>;
//     fn get_solution(&self) -> &Box<dyn AufgabeData>;
// }

// impl Aufgabe_t for Aufgabe {
//     fn get_which(&self) -> &str {
//         &self.which
//     }

//     fn get_algo(&self) -> &Box<dyn Fn() -> Box<dyn AufgabeData>> {
//         &self.algo
//     }

//     fn get_solution(&self) -> &Box<dyn AufgabeData> {
//         &self.solution
//     }
// }

fn klausur_stappert_ss15() -> Vec<Problem> {
    vec![{
        let input = vec![
            Process::new(1, 6),
            Process::new(4, 2),
            Process::new(2, 4),
            Process::new(9, 3),
            Process::new(8, 4),
        ];
        Problem {
            which: "4.1".to_owned(),
            algo: Box::new(|| {
                Box::new((
                    (&input).clone(),
                    round_robin((&input).clone().into_iter(), 3),
                )) as Box<dyn ProblemData>
            }),
            solution: Box::new((
                vec![
                    Process {
                        arrival: 1,
                        computation_time: 6,
                    },
                    Process {
                        arrival: 4,
                        computation_time: 2,
                    },
                    Process {
                        arrival: 2,
                        computation_time: 4,
                    },
                    Process {
                        arrival: 9,
                        computation_time: 3,
                    },
                    Process {
                        arrival: 8,
                        computation_time: 4,
                    },
                ],
                Schedule(vec![
                    None,
                    Some(0),
                    Some(0),
                    Some(0),
                    Some(2),
                    Some(2),
                    Some(2),
                    Some(1),
                    Some(1),
                    Some(0),
                    Some(0),
                    Some(0),
                    Some(2),
                    Some(4),
                    Some(4),
                    Some(4),
                    Some(3),
                    Some(3),
                    Some(3),
                    Some(4),
                ]),
            )),
        }
    }]
}

fn klausur_stappert_ss23_probeklausur() -> Vec<Problem> {
    let mut questions: Vec<Problem> = Vec::new();
    questions.push(Problem {
        which: "3.2".to_owned(),
        algo: Box::new(|| {
            // gegeben
            let block_size = 2_u64.pow(12);
            let total_used_blocks = iter::from_generator(|| {
                for i in 0..u64::MAX {
                    if i < 301 || i % 2 == 1 {
                        continue;
                    }
                    yield i;
                }
            });
            let file_size: u64 = 33 * 1024;

            // gesucht
            let mut used_space: u64 = 0;
            let mut used_blocks: Vec<u64> = Vec::new();
            for block in total_used_blocks {
                used_blocks.push(block);
                used_space += block_size;
                if used_space >= file_size {
                    break;
                }
            }

            Box::new(format!(
                "file size: {}KiB, actual space used: {}KiB, {} used blocks: {:?}",
                file_size as f64 / 1024.0,
                used_space as f64 / 1024.0,
                used_blocks.len(),
                used_blocks
            )) as Box<dyn ProblemData>
        }),
        solution: Box::new("file size: 33KiB, actual space used: 36KiB, 9 used blocks: [302, 304, 306, 308, 310, 312, 314, 316, 318]".to_owned()),
    });
    questions.push(Problem {
        which: "4".to_owned(),
        algo: Box::new(|| {
            // gegeben
            let cylinders: Vec<usize> = vec![11, 2, 38, 19, 34, 9, 12, 40, 50];

            // FCFS
            let jumps_fcfs = cylinders
                .iter()
                .skip(1)
                .zip(cylinders.iter())
                .map(|(c1, c2)| usize::abs_diff(*c1, *c2))
                .collect_vec();

            // SSF
            let mut cyls_tmp = cylinders.clone();
            let mut last_diff = (0_usize, 0);
            let jumps_ssf = iter::from_generator(|| loop {
                let last_cyl = cyls_tmp.remove(last_diff.0);
                if cyls_tmp.is_empty() {
                    return;
                }
                let new_diff = cyls_tmp
                    .iter()
                    .map(|c| usize::abs_diff(*c, last_cyl))
                    .enumerate()
                    .reduce(|(i, diff), (i_cur, diff_cur)| {
                        if diff_cur < diff {
                            (i_cur, diff_cur)
                        } else {
                            (i, diff)
                        }
                    })
                    .expect("cylinders were empty maybe?");
                last_diff = new_diff;

                yield last_diff.1;
            })
            .collect_vec();

            // Aufzug

            let mut cyls_tmp = cylinders;
            cyls_tmp.sort();
            let mut last_diff = (2_usize, 0);

            #[derive(Clone, Copy, Debug)]
            enum Direction {
                Up,
                Down,
            }
            use Direction::*;
            let mut direction = Direction::Up;
            let jumps_aufzug = iter::from_generator(|| loop {
                /*println!(
                    "DEBUG: processing cyl ({}, {}, diff {})",
                    last_diff.0, cyls_tmp[last_diff.0], last_diff.1
                );*/
                let last_cyl = cyls_tmp.remove(last_diff.0);
                if cyls_tmp.is_empty() {
                    return;
                }
                // sort because we go in one direction only
                let new_diff = cyls_tmp.iter();
                // Aufzug part
                let n = last_diff.0;
                let make_considered = |direction| match direction {
                    // kill previous (n == last cylinder idx). Works even after deletion. E.g. if the first was removed, we want to skip 0 etc.
                    Up => new_diff.clone().skip(n).collect_vec(),
                    // else stop after n
                    Down => new_diff.clone().take(n).collect_vec(),
                };
                let mut considered = make_considered(direction);
                if considered.is_empty() {
                    direction = match direction {
                        Down => Up,
                        Up => Down,
                    };
                    considered = make_considered(direction);
                    if considered.is_empty() {
                        // both are empty -> we have no more elems and are finished
                        // probably superfluous but an extra check nonetheless
                        return;
                    }
                }
                let new_diff = considered
                    .iter()
                    .map(|c| usize::abs_diff(**c, last_cyl))
                    .enumerate()
                    .reduce(|(i, diff), (i_cur, diff_cur)| {
                        if diff_cur < diff {
                            (i_cur, diff_cur)
                        } else {
                            (i, diff)
                        }
                    })
                    .expect("cylinders were empty maybe?");
                // we add the index back, because we were dealing with offsets relative to the previous element
                let new_diff = match direction {
                    Up => (new_diff.0 + n, new_diff.1),
                    Down => new_diff,
                };
                last_diff = new_diff;

                yield last_diff.1;
            })
            .collect_vec();

            Box::new(format!(
                "FCFS: {} {:?}, SSF: {} {:?}, Aufzug: {} {:?}",
                jumps_fcfs.iter().sum::<usize>(),
                jumps_fcfs,
                jumps_ssf.iter().sum::<usize>(),
                jumps_ssf,
                jumps_aufzug.iter().sum::<usize>(),
                jumps_aufzug
            )) as Box<dyn ProblemData>
        }),
        solution: Box::new("FCFS: 145 [9, 36, 19, 15, 25, 3, 28, 10], SSF: 59 [1, 3, 7, 17, 15, 4, 2, 10], Aufzug: 87 [1, 7, 15, 4, 2, 10, 41, 7]".to_owned()),
    });
    questions
}

impl Problem {
    fn eval(&self) -> () {
        let Problem {
            which,
            algo,
            solution,
        } = self;
        println!("### Question number {}", which);
        let result = algo();
        println!("{:?}", result);
        assert_eq!(format!("{:?}", result), format!("{:?}", solution));
    }
}
