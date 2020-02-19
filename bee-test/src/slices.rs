pub fn slice_eq<'a, T: PartialEq>(a: &'a [T], b: &'a [T]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }

    true
}
