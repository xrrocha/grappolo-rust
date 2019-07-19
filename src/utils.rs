//! This module provides miscellaneous utility functions.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::time::SystemTime;

pub fn string_vec(strs: Vec<&str>) -> Vec<String> {
    strs.iter().map(|s| String::from(*s)).collect::<Vec<String>>()
}


pub fn read_all_file_lines(filename: String) -> Vec<String> {
    read_file_lines(filename, usize::max_value())
}

pub fn read_file_lines(filename: String, up_to: usize) -> Vec<String> {
    let file = File::open(filename).expect("Can't open input file");

    BufReader::new(file)
        .lines()
        .take(up_to)
        .map(|line| line.expect("Error reading input file"))
        .collect::<Vec<String>>()
}

pub fn open_output_file(filename: String) -> File {
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(false)
        .open(filename)
        .expect("Error opening output file")
}

pub fn millis_since(start_time: SystemTime) -> u128 {
    SystemTime::now()
        .duration_since(start_time)
        .expect("Error in time!")
        .as_millis()
}
