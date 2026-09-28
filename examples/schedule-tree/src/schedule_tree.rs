/// Each branch chooses one child per tick, independent of subtree size.
pub struct ScheduleTree {
    root: Node,
}

enum Node {
    Leaf(&'static str),
    Branch { children: Vec<Node>, next: usize },
}

impl Node {
    fn tick(&mut self) -> &'static str {
        match self {
            Self::Leaf(name) => name,
            Self::Branch { children, next } => {
                let index = *next;
                *next = (index + 1) % children.len();
                children[index].tick()
            }
        }
    }
}

impl ScheduleTree {
    pub fn new(groups: Vec<Vec<&'static str>>) -> Self {
        assert!(!groups.is_empty() && groups.iter().all(|group| !group.is_empty()));
        let children = groups
            .into_iter()
            .map(|group| Node::Branch {
                children: group.into_iter().map(Node::Leaf).collect(),
                next: 0,
            })
            .collect();
        Self { root: Node::Branch { children, next: 0 } }
    }

    pub fn tick(&mut self) -> &'static str {
        self.root.tick()
    }
}
