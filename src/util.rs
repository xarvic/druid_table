
pub fn set_len<T, F: Fn() -> T>(list: &mut Vec<T>, new_len: usize, f: F) {
    if list.len() > new_len {
        list.truncate(new_len);
    } else {
        while list.len() < new_len {
            list.push(f());
        }
    }
}