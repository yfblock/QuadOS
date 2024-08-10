use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use polyhal::percpu::def_percpu;

use super::task::Task;

pub struct Scheduler {
    tasks: BTreeMap<usize, Task>,
}
