// TODO: remove the clone // expect: comment/untracked-todo
pub fn total(values: &[u32]) -> u32 {
    // FIXME: overflow // expect: comment/untracked-todo
    values.iter().sum()
}
