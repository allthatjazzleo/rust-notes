use prettytable::{color, format::Alignment, Attr, Cell, Row, Table};
use rand::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

type RealNode = Rc<RefCell<Node>>;
type Link = Option<Rc<RefCell<Node>>>;

#[derive(Debug, Clone)]
pub struct Node {
    data: String,
    next: Vec<Link>,
    offset: u64,
    pos: u64,
}

impl Node {
    fn new(next: Vec<Link>, offset: u64, data: String, pos: u64) -> RealNode {
        Rc::new(RefCell::new(Node {
            next,
            offset,
            data,
            pos,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct SkipList {
    head: Link,
    tails: Vec<Link>,
    max_level_idx: usize,
    length: u64,
}

impl SkipList {
    pub fn new(level: usize) -> Self {
        SkipList {
            head: None,
            tails: vec![None; level],
            max_level_idx: level - 1,
            length: 0,
        }
    }

    fn get_level(&self) -> usize {
        let mut n = 0;
        let mut rng = rand::thread_rng();
        while rng.gen::<bool>() && n < self.max_level_idx {
            n += 1;
        }
        n
    }

    pub fn append(&mut self, offset: u64, data: String) {
        let level = 1 + if self.head.is_none() {
            self.max_level_idx
        } else {
            self.get_level()
        };
        let node = Node::new(vec![None; level], offset, data, self.length);
        for i in 0..level {
            if let Some(old) = self.tails[i].take() {
                let next = &mut old.borrow_mut().next;
                next[i] = Some(node.clone());
            }
            self.tails[i] = Some(node.clone());
        }

        if self.head.is_none() {
            self.head = Some(node);
        }
        self.length += 1;
    }

    pub fn level_path(&self, offset: u64, found_level: usize) {
        // Create the table
        let mut table = Table::new();
        if let Some(ref head) = self.head {
            let node = head.clone();

            for level in (0..=self.max_level_idx).rev() {
                let mut cells = vec![];
                let mut n = node.clone();
                cells.push(Cell::new(format!("level={level:?}").as_str()));
                let mut pos: u64 = 0;
                loop {
                    let next = n.clone();

                    let mut color = if level == found_level && next.borrow().offset <= offset {
                        color::RED
                    } else {
                        color::WHITE
                    };
                    while next.borrow().pos > pos {
                        cells.push(
                            Cell::new_align("->", Alignment::CENTER)
                                .with_style(Attr::ForegroundColor(color)),
                        );
                        pos += 1;
                    }
                    color = if next.borrow().offset == offset {
                        color::GREEN
                    } else {
                        color
                    };
                    cells.push(
                        Cell::new(
                            format!(
                                "offset={:?}, data={:?}",
                                next.borrow().offset,
                                next.borrow().data
                            )
                            .as_str(),
                        )
                        .with_style(Attr::ForegroundColor(color)),
                    );
                    pos += 1;
                    match next.borrow().next[level] {
                        Some(ref next) => {
                            n = next.clone();
                        }
                        _ => break,
                    };
                }
                while self.length > pos {
                    cells.push(Cell::new_align("->", Alignment::CENTER));
                    pos += 1;
                }
                table.add_row(Row::new(cells));
            }
            table.printstd();
        }
    }

    pub fn find(&self, offset: u64) -> Option<(String, usize)> {
        match self.head {
            Some(ref head) => {
                let mut start_level = self.max_level_idx;
                let node = head.clone();
                let mut result = None;

                loop {
                    if node.borrow().next[start_level].is_some() {
                        break;
                    }
                    start_level -= 1;
                }
                let mut n = node;
                for level in (0..=start_level).rev() {
                    loop {
                        let next = n.clone();
                        match next.borrow().next[level] {
                            Some(ref tmp) => {
                                if tmp.borrow().offset <= offset {
                                    n = tmp.clone();
                                } else {
                                    break;
                                }
                            }
                            _ => break,
                        };
                    }
                    if n.borrow().offset == offset {
                        let tmp = n.borrow();
                        result = Some((tmp.data.clone(), level));
                        break;
                    }
                }
                result
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter;

    fn generate(len: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        let one_char = || CHARSET[rng.gen_range(0..CHARSET.len())] as char;
        iter::repeat_with(one_char).take(len).collect()
    }
    #[test]
    fn run_test() {
        let mut skip_list = SkipList::new(6);
        let mut generate_values = vec![];
        for i in 0..10 {
            generate_values.push(generate(6));
            skip_list.append(i, generate_values[i as usize].clone());
        }
        let offset = rand::thread_rng().gen_range(0..10);
        if let Some((data, level)) = skip_list.find(offset) {
            println!("offset={}, data={}, level_found={}", offset, data, level);
            skip_list.level_path(offset, level);
            assert_eq!(data, generate_values[offset as usize])
        }
    }
}
