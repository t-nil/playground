#![feature(generators)]
#![feature(iter_from_generator)]
#![feature(iter_collect_into)]
#![feature(drain_filter)]

use std::{collections::HashMap, iter};

use itertools::Itertools;

pub mod cap03_scheduling;

fn main() {
    probeklausur();
    cap03_scheduling::test_round_robin();
}

fn probeklausur() {
    let mut questions: Vec<(
        &str,
        Box<dyn Fn() -> String>,
        //Box<dyn Fn(dyn Fn() -> String) -> ()>,
        String,
    )> = Vec::new();
    questions.push((
        "3.2",
        Box::new(|| {
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

            format!(
                "file size: {}KiB, actual space used: {}KiB, {} used blocks: {:?}",
                file_size as f64 / 1024.0,
                used_space as f64 / 1024.0,
                used_blocks.len(),
                used_blocks
            )
        }),
        "file size: 33KiB, actual space used: 36KiB, 9 used blocks: [302, 304, 306, 308, 310, 312, 314, 316, 318]".to_owned(),
    ));
    questions.push((
        "4",
        Box::new(|| {
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

            let mut cyls_tmp = cylinders.clone();
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
                if considered.len() == 0 {
                    direction = match direction {
                        Down => Up,
                        Up => Down,
                    };
                    considered = make_considered(direction);
                    if considered.len() == 0 {
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

            format!(
                "FCFS: {} {:?}, SSF: {} {:?}, Aufzug: {} {:?}",
                jumps_fcfs.iter().sum::<usize>(),
                jumps_fcfs,
                jumps_ssf.iter().sum::<usize>(),
                jumps_ssf,
                jumps_aufzug.iter().sum::<usize>(),
                jumps_aufzug
            )
        }),
        "FCFS: 145 [9, 36, 19, 15, 25, 3, 28, 10], SSF: 59 [1, 3, 7, 17, 15, 4, 2, 10], Aufzug: 87 [1, 7, 15, 4, 2, 10, 41, 7]".to_owned(),
    ));
    questions.iter().for_each(|(number, fun, verify)| {
        println!("### Question number {}", number);
        let result = fun();
        println!("{}", result);
        assert_eq!(result, *verify);
    });
}
