#![allow(dead_code)]

use alloc::collections::btree_map::BTreeMap;

use super::task::Task;

pub struct Scheduler {
    tasks: BTreeMap<usize, Task>,
}
