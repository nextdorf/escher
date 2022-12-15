
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum HierarchyError {
  PathNotFound,
  HierarchyNotFound,
  ToplevelNotFound,
}

