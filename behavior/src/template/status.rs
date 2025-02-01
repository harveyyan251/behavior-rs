// Inner must be unsigned integer type
type Inner = u32;

#[derive(Copy, Clone, PartialEq)]
pub struct BranchData {
    inner: Inner,
}

impl BranchData {
    pub const MAX_BRANCH: usize = size_of::<Inner>() * 8 - 2;

    // Branch must in [0, MAX_BRANCH]
    pub fn single_branch(branch: usize) -> Self {
        assert!(
            branch <= Self::MAX_BRANCH,
            "Branch can't be greater than {}, branch={}",
            Self::MAX_BRANCH,
            branch
        );
        Self {
            inner: (branch as Inner) << 1,
        }
    }

    // Branch must in [0, MAX_BRANCH]
    pub fn multiple_branch(branchs: (impl IntoIterator<Item = usize> + std::fmt::Debug)) -> Self {
        let mut inner: Inner = 0;
        for branch in branchs.into_iter() {
            assert!(
                branch <= Self::MAX_BRANCH,
                "Branch can't be greater than {}, branch={}",
                Self::MAX_BRANCH,
                branch
            );
            inner |= 1 << branch;
        }
        inner = (inner << 1) | 1;
        Self { inner }
    }

    #[inline]
    pub fn is_single_branch(&self) -> bool {
        self.inner & 1 == 0
    }

    #[inline]
    pub fn is_multiple_branch(&self) -> bool {
        !self.is_single_branch()
    }

    #[inline]
    pub fn get_single_branch(&self) -> usize {
        (self.inner >> 1) as usize
    }

    pub fn get_multiple_branch(&self, max_hint: Option<usize>) -> (Vec<usize>, bool) {
        let value = self.inner >> 1;
        let max_branch =
            max_hint.map_or(Self::MAX_BRANCH, |hint_max| hint_max.min(Self::MAX_BRANCH));
        (
            (0..=max_branch)
                .filter(|index| value & (1 << index) != 0)
                .collect(),
            value >> (max_branch + 1) == 0,
        )
    }

    fn fmt_branch(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_single_branch() {
            write!(f, "{}", self.get_single_branch())
        } else {
            write!(f, "{:?}", self.get_multiple_branch(None).0)
        }
    }
}

impl std::fmt::Debug for BranchData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_branch(f)
    }
}

// IMPORTANT: Custom nodes should NEVER return Idle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Status {
    Idle,
    Success,
    Failure,
    Running,
    Branch(BranchData),
}

impl Default for Status {
    fn default() -> Self {
        Status::Idle
    }
}

impl From<bool> for Status {
    fn from(value: bool) -> Self {
        if value {
            Status::Success
        } else {
            Status::Failure
        }
    }
}

impl Status {
    pub fn replace(&mut self, status: Status) -> Status {
        let result = *self;
        *self = status;
        result
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Idle => "Idle",
            Status::Success => "Success",
            Status::Failure => "Failure",
            Status::Running => "Running",
            Status::Branch(_) => "Branch",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashSet, VecDeque};

    #[test]
    fn debug_print() {
        let stauts = Status::Idle;
        println!("{:?}", stauts);
        let stauts = Status::Success;
        println!("{:?}", stauts);
        let stauts = Status::Failure;
        println!("{:?}", stauts);
        let stauts = Status::Running;
        println!("{:?}", stauts);
        let status = Status::Branch(BranchData::single_branch(3));
        println!("{:?}", status);
        let status = Status::Branch(BranchData::multiple_branch(vec![
            0,
            1,
            3,
            BranchData::MAX_BRANCH / 2,
            BranchData::MAX_BRANCH,
        ]));
        println!("{:?}", status);
    }

    #[test]
    fn test_branch() {
        // Single branch
        (0..=BranchData::MAX_BRANCH).for_each(|num| {
            let single_branch = BranchData::single_branch(num);
            println!(
                "is_single_branch={}, single_branch={}",
                single_branch.is_single_branch(),
                single_branch.get_single_branch()
            );
        });

        // Multiple branch
        let branchs: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        println!("branch={:?}", BranchData::multiple_branch(branchs.clone()));
        let branchs: VecDeque<usize> = branchs.into();
        println!("branch={:?}", BranchData::multiple_branch(branchs.clone()));
        let branchs: HashSet<usize> = branchs.into_iter().collect();
        println!("branch={:?}", BranchData::multiple_branch(branchs));
        (0..=BranchData::MAX_BRANCH).for_each(|num| {
            let multiple_branch = BranchData::multiple_branch(0..=num);
            let (branchs, is_valid) = multiple_branch.get_multiple_branch(Some(25));
            println!(
                "num={}, is_multiple_branch={}, multiple_branchs={:?}, is_valid={}",
                num,
                multiple_branch.is_multiple_branch(),
                branchs,
                is_valid,
            );
        });
    }
}
