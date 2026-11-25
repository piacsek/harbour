use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Member {
    Alone,
    First,
    Later,
}

pub fn member<T: PartialEq>(keys: &[T], at: usize) -> Member {
    match run(keys, at) {
        Some(group) if group.len() < 2 => Member::Alone,
        Some(group) if group.start == at => Member::First,
        Some(_) => Member::Later,
        None => Member::Alone,
    }
}

pub fn order<T: PartialEq>(keys: &[T]) -> Vec<usize> {
    let mut order = Vec::with_capacity(keys.len());
    for (first, key) in keys.iter().enumerate() {
        if keys[..first].contains(key) {
            continue;
        }
        order.extend((first..keys.len()).filter(|index| keys[*index] == *key));
    }
    order
}

pub fn step<T: PartialEq>(keys: &[T], from: usize, down: bool) -> Option<Vec<usize>> {
    let group = run(keys, from)?;
    let (moved, past) = if from != group.start {
        let to = if down { from + 1 } else { from - 1 };
        group.contains(&to).then_some(())?;
        (from..from + 1, to..to + 1)
    } else if down {
        (group.clone(), run(keys, group.end)?)
    } else {
        (group.clone(), run(keys, group.start.checked_sub(1)?)?)
    };
    let (first, second) = if down { (past, moved) } else { (moved, past) };
    let start = first.start.min(second.start);
    let end = first.end.max(second.end);
    Some(
        (0..start)
            .chain(first)
            .chain(second)
            .chain(end..keys.len())
            .collect(),
    )
}

fn run<T: PartialEq>(keys: &[T], at: usize) -> Option<Range<usize>> {
    let key = keys.get(at)?;
    let start = (0..at)
        .rev()
        .take_while(|index| keys[*index] == *key)
        .last()
        .unwrap_or(at);
    let end = (at..keys.len())
        .take_while(|index| keys[*index] == *key)
        .last()?
        + 1;
    Some(start..end)
}
