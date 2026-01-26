#![allow(warnings)]

mod init;
mod test;

use test::__string;
use test::__group_parentheses;

use std::fs;

fn main() {
    __string();
    __group_parentheses();
}
