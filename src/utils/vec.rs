/// Extension methods for [`Vec`]
pub trait VecExt<T> {
    fn r#as(self) -> Vec<T>;
}

impl VecExt<i8> for Vec<u8> {
    fn r#as(self) -> Vec<i8> {
        let (ptr, length, capacity) = self.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr as *mut i8, length, capacity) }
    }
}

impl VecExt<u8> for Vec<i8> {
    fn r#as(self) -> Vec<u8> {
        let (ptr, length, capacity) = self.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr as *mut u8, length, capacity) }
    }
}
