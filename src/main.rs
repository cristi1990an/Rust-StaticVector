mod static_containers {

    use std::{
        mem::MaybeUninit,
        ops::{Deref, DerefMut}
    };

    type StorageType<T, const N: usize> = [MaybeUninit<T>; N];
    pub struct StaticVector<T, const N: usize> {
        storage: StorageType<T, N>,
        len: usize,
    }

    impl<T, const N: usize> StaticVector<T, N> {
        #[inline]
        pub const fn new() -> Self {
            StaticVector {
                storage: [const { MaybeUninit::uninit() }; N],
                len: 0,
            }
        }

        #[inline]
        pub const fn capacity(&self) -> usize {
            N
        }

        #[inline]
        pub fn push(&mut self, value: T) {
            let len = self.len();
            match self.storage.get_mut(len) {
                Some(last_uninit) => {
                    last_uninit.write(value);
                    self.len += 1;
                }
                None => panic!("capacity (is {}) reached", self.len),
            }
        }

        #[inline]
        pub fn last(&self) -> Option<&T> {
            self.as_slice().last()
        }

        #[inline]
        pub fn last_mut(&mut self) -> Option<&mut T> {
            self.as_slice_mut().last_mut()
        }

        #[inline]
        pub fn as_slice(&self) -> &[T] {
            if self.is_empty() {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(self.storage[0].assume_init_ref(), self.len) }
            }
        }

        #[inline]
        pub fn as_slice_mut(&mut self) -> &mut [T] {
            if self.is_empty() {
                &mut []
            } else {
                unsafe { std::slice::from_raw_parts_mut(self.storage[0].as_mut_ptr(), self.len) }
            }
        }

        #[inline]
        fn unchecked_pop(&mut self) -> T {
            self.len -= 1;
            unsafe {
                let last = self.storage.as_ptr().add(self.len);
                last.read().assume_init()
            }
        }

        #[inline]
        pub fn pop(&mut self) -> Option<T> {
            if self.is_empty() {
                None
            } else {
                Some(self.unchecked_pop())
            }
        }

        #[inline]
        pub fn as_ptr(&self) -> *const T {
            self.as_slice().as_ptr()
        }

        #[inline]
        pub fn as_mut_ptr(&mut self) -> *mut T {
            self.as_slice_mut().as_mut_ptr()
        }

        #[inline]
        pub fn pop_if<F>(&mut self, f: F) -> Option<T>
        where
            F: FnOnce(&mut T) -> bool,
        {
            let last = self.last_mut()?;
            if f(last) {
                return self.pop();
            }
            None
        }

        #[inline]
        pub fn clear(&mut self) {
            while self.pop().is_some() {}
        }

        #[inline]
        fn unchecked_truncate(&mut self, new_len: usize) {
            while self.len() != new_len {
                self.pop();
            }
        }

        #[inline]
        pub fn truncate(&mut self, new_len: usize) {
            if new_len < self.len() {
                self.unchecked_truncate(new_len);
            }
        }

        #[inline]
        pub fn resize(&mut self, new_len: usize, value: T)
        where
            T: Clone,
        {
            let less_than_current = ..self.len();
            let more_than_current = self.len()..self.capacity() + 1;
            if less_than_current.contains(&new_len) {
                self.unchecked_truncate(new_len);
            } else if more_than_current.contains(&new_len) {
                for idx in self.len..new_len {
                    unsafe {
                        self.storage.get_unchecked_mut(idx).write(value.clone());
                    }
                }
                self.len = new_len;
            } else {
                panic!(
                    "resize call (is {}) bigger than capacity (is {})",
                    self.len,
                    self.capacity()
                );
            }
        }

        #[inline]
        pub fn remove(&mut self, index: usize) -> T {
            let len = self.len();
            if (..self.len).contains(&index) {
                unsafe { self.get_unchecked_mut(index..) }.rotate_left(1);
                self.unchecked_pop()
            } else {
                panic!("removal index (is {index}) should be < len (is {len})");
            }
        }

        #[inline]
        pub fn remove_swap(&mut self, index: usize) -> T {
            let len = self.len();
            if index < len {
                let range = unsafe { self.get_unchecked_mut(index..) };
                let first = range.as_mut_ptr();
                let last = unsafe { first.add(range.len() - 1) };
                unsafe { std::ptr::swap(first, last) };
                self.unchecked_pop()
            } else {
                panic!("removal index (is {index}) should be < len (is {len})");
            }
        }

        #[inline]
        pub fn insert(&mut self, index: usize, element: T) {
            let len = self.len();
            if index > len {
                panic!("insertion index (is {index}) should be <= len (is {len})");
            }
            if len == self.capacity() {
                panic!("capacity (is {len}) reached");
            }

            unsafe {
                let p = self.as_mut_ptr().add(index);
                if index < len {
                    std::ptr::copy(p, p.add(1), len - index);
                }
                std::ptr::write(p, element);
            }
            self.len += 1;
        }
    }

    impl<T, const N: usize> Default for StaticVector<T, N> {
        #[inline]
        fn default() -> StaticVector<T, N> {
            StaticVector::new()
        }
    }

    impl<T, const N: usize> AsRef<StaticVector<T, N>> for StaticVector<T, N> {
        #[inline]
        fn as_ref(&self) -> &StaticVector<T, N> {
            self
        }
    }

    impl<T, const N: usize> AsMut<StaticVector<T, N>> for StaticVector<T, N> {
        #[inline]
        fn as_mut(&mut self) -> &mut StaticVector<T, N> {
            self
        }
    }

    impl<T, const N: usize> Drop for StaticVector<T, N> {
        #[inline]
        fn drop(&mut self) {
            unsafe {
                while self.len != 0 {
                    self.len -= 1;
                    self.storage.get_unchecked_mut(self.len).assume_init_drop();
                }
            }
        }
    }

    impl<T, const N: usize> Clone for StaticVector<T, N>
    where
        T: Clone,
    {
        #[inline]
        fn clone(&self) -> Self {
            unsafe {
                let mut result = Self::new();
                for (dest, src) in std::iter::zip(&mut result.storage, &self.storage).take(self.len)
                {
                    dest.write(src.assume_init_ref().clone());
                    result.len += 1;
                }
                result
            }
        }
    }

    pub struct IntoIter<T, const N: usize> {
        storage: [MaybeUninit<T>; N],
        len: usize,
        index: usize,
    }

    impl<T, const N: usize> Drop for IntoIter<T, N> {
        #[inline]
        fn drop(&mut self) {
            unsafe {
                while self.index != self.len {
                    self.storage
                        .get_unchecked_mut(self.index)
                        .assume_init_drop();
                    self.index += 1;
                }
            }
        }
    }

    impl<T, const N: usize> Iterator for IntoIter<T, N> {
        type Item = T;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            let next_uninit = self.storage[..self.len].get(self.index)?;
            self.index += 1;
            Some(unsafe { next_uninit.assume_init_read() })
        }
    }

    impl<T, const N: usize> IntoIterator for StaticVector<T, N> {
        type Item = T;
        type IntoIter = IntoIter<T, N>;

        #[inline]
        fn into_iter(mut self) -> Self::IntoIter {
            let result = Self::IntoIter {
                storage: unsafe { std::mem::transmute_copy(&self.storage) },
                len: self.len,
                index: 0,
            };
            self.len = 0;
            result
        }
    }

    impl<T, const N: usize> Deref for StaticVector<T, N> {
        type Target = [T];

        #[inline]
        fn deref(&self) -> &Self::Target {
            match self.len == 0 {
                true => &[],
                false => unsafe {
                    let ptr = self.storage.get_unchecked(0);
                    std::slice::from_raw_parts(ptr.assume_init_ref(), self.len)
                },
            }
        }
    }

    impl<T, const N: usize> DerefMut for StaticVector<T, N> {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            match self.len == 0 {
                true => &mut [],
                false => unsafe {
                    let ptr = self.storage.get_unchecked_mut(0);
                    std::slice::from_raw_parts_mut(ptr.as_mut_ptr(), self.len)
                },
            }
        }
    }

    impl<'a, T, const N: usize> IntoIterator for &'a StaticVector<T, N> {
        type Item = &'a T;

        type IntoIter = std::slice::Iter<'a, T>;

        #[inline]
        fn into_iter(self) -> std::slice::Iter<'a, T> {
            self.iter()
        }
    }

    impl<'a, T, const N: usize> IntoIterator for &'a mut StaticVector<T, N> {
        type Item = &'a mut T;

        type IntoIter = std::slice::IterMut<'a, T>;

        #[inline]
        fn into_iter(self) -> std::slice::IterMut<'a, T> {
            self.iter_mut()
        }
    }

    impl<T: Clone, const N: usize> From<&[T]> for StaticVector<T, N> {
        #[inline]
        fn from(array: &[T]) -> Self {
            let mut result = Self::new();

            let mut new_len = 0;
            for (uninit, elem) in result.storage.iter_mut().zip(array) {
                uninit.write(elem.clone());
                new_len += 1;
            }
            result.len = new_len;
            result
        }
    }

    impl<T, const N: usize> From<[T; N]> for StaticVector<T, N> {
        #[inline]
        fn from(array: [T; N]) -> Self {
            let mut result = Self::new();

            for (uninit, elem) in result.storage.iter_mut().zip(array) {
                uninit.write(elem);
            }
            result.len = N;
            result
        }
    }

    impl<T: Clone, const N: usize> From<&[T; N]> for StaticVector<T, N> {
        #[inline]
        fn from(array: &[T; N]) -> Self {
            let mut result = Self::new();

            for (uninit, elem) in result.storage.iter_mut().zip(array) {
                uninit.write(elem.clone());
            }
            result.len = N;
            result
        }
    }

    impl<T: Clone, const N: usize> From<&mut [T; N]> for StaticVector<T, N> {
        #[inline]
        fn from(array: &mut [T; N]) -> Self {
            let mut result = Self::new();

            for (uninit, elem) in result.storage.iter_mut().zip(array) {
                uninit.write(elem.clone());
            }
            result.len = N;
            result
        }
    }

    impl<T: std::fmt::Debug, const N: usize> std::fmt::Debug for StaticVector<T, N> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&**self, f)
        }
    }

    #[macro_export]
    macro_rules! count_elements {
        // Base case: when no elements are left, the count is 0
        () => { 0 };
        ($last:expr) => {
            1
        };
        ($first:expr, $($rest:expr),*) => {
            1 + count_elements!($($rest),*)
        };
    }

    #[macro_export]
    macro_rules! static_vec {

        ($value:expr; $capacity:expr) => {

            {
            const CAPACITY: usize = $capacity;
            let array: [_; CAPACITY] = [($value); CAPACITY];
            $crate::static_containers::StaticVector::from(array)
        }};
        ($($elem:expr),* $(,)?) => {{
            use crate::count_elements;
            const CAPACITY: usize = count_elements!($($elem),*);
            let array: [_; CAPACITY] = [$($elem),*];
            $crate::static_containers::StaticVector::from(array)
        }};
        ($($elem:expr),* $(,)?; $capacity:expr) => {{
            const CAPACITY: usize = $capacity;
            use crate::count_elements;
            const COUNT: usize = count_elements!($($elem),*);
            assert!(CAPACITY >= COUNT,
                "capacity (is {}) must be bigger than elements count (is {})",
                CAPACITY, COUNT);
            let array = [$($elem),*].as_slice();
            || -> $crate::static_containers::StaticVector<_, CAPACITY>
            {
                $crate::static_containers::StaticVector::from(array)
            }()
        }};
    }
}

#[cfg(test)]
mod static_vec_tests {
    use crate::{static_containers::*, static_vec};

    #[test]
    fn test_default_init() {
        let vec = StaticVector::<i32, 32>::default();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn test_new() {
        let vec = StaticVector::<i32, 32>::new();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn test_macro_list_init() {
        let vec = static_vec![1, 2, 3, 4];
        assert_eq!(vec.len(), 4);
        assert_eq!(vec.capacity(), 4);
        assert!(vec.iter().eq([1, 2, 3, 4].iter()));
    }

    #[test]
    fn test_macro_init() {
        let vec = static_vec![42; 4];
        assert_eq!(vec.len(), 4);
        assert_eq!(vec.capacity(), 4);
        assert!(vec.iter().eq(vec![42; 4].iter()));
    }

    #[test]
    #[should_panic]
    fn test_macro_list_init_with_not_enough_capacity() {
        let vec = static_vec![1, 2, 3, 4; 3];
        assert_eq!(vec.len(), 4);
        assert_eq!(vec.capacity(), 10);
        assert!(vec.iter().eq([1, 2, 3, 4].iter()));
    }

    #[test]
    fn test_from_array() {
        let vec: StaticVector<i32, 4> = [1, 2, 3, 4].into();
        assert_eq!(vec.len(), 4);
        assert!(vec.iter().eq([1, 2, 3, 4].iter()));
    }

    #[test]
    fn test_push() {
        let mut vec = StaticVector::<String, 4>::new();

        vec.push("1".to_string());

        assert_eq!(vec.len(), 1);
        assert!(vec.iter().eq(["1"].iter()));

        vec.push("2".to_string());

        assert_eq!(vec.len(), 2);
        assert!(vec.iter().eq(["1", "2"].iter()));

        vec.push("3".to_string());

        assert_eq!(vec.len(), 3);
        assert!(vec.iter().eq(["1", "2", "3"].iter()));

        vec.push("4".to_string());

        assert_eq!(vec.len(), 4);
        assert!(vec.iter().eq(["1", "2", "3", "4"].iter()));
    }

    #[test]
    #[should_panic]
    fn test_push_panic() {
        let mut vec = static_vec![1, 2, 3, 4];
        vec.push(5);
    }

    #[test]
    fn test_pop() {
        let mut vec = static_vec!["1".to_string(), "2".to_string(), "3".to_string()];

        assert_eq!(vec.pop(), Some("3".to_string()));
        assert_eq!(vec.len(), 2);

        assert_eq!(vec.pop(), Some("2".to_string()));
        assert_eq!(vec.len(), 1);

        assert_eq!(vec.pop(), Some("1".to_string()));
        assert_eq!(vec.len(), 0);

        assert_eq!(vec.pop(), None);
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn test_remove_front() {
        let mut vec = static_vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string()
        ];

        let removed = vec.remove(0);
        assert_eq!(removed, "1");
        assert_eq!(vec.len(), 3);
        assert!(vec.iter().eq(["2", "3", "4"]));
    }

    #[test]
    fn test_remove_mid() {
        let mut vec = static_vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string()
        ];

        let removed = vec.remove(2);
        assert_eq!(removed, "3");
        assert_eq!(vec.len(), 3);
        assert!(vec.iter().eq(["1", "2", "4"]));
    }

    #[test]
    fn test_remove_end() {
        let mut vec = static_vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string()
        ];

        let removed = vec.remove(3);
        assert_eq!(removed, "4");
        assert_eq!(vec.len(), 3);
        assert!(vec.iter().eq(["1", "2", "3"]));
    }

    #[test]
    fn test_remove_last() {
        let mut vec = static_vec!["1".to_string()];

        vec.remove(0);

        assert!(vec.is_empty());
    }

    #[test]
    #[should_panic]
    fn test_remove_panic() {
        let mut vec = static_vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string()
        ];

        vec.remove(4);
    }

    
    #[test]
    fn test_remove_swap_front() {
        let mut vec = static_vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string()
        ];

        let removed = vec.remove_swap(0);
        assert_eq!(removed, "1");
        assert_eq!(vec.len(), 3);
        assert!(vec.iter().eq(["4", "2", "3"]));
    }

    #[test]
    fn test_remove_swap_mid() {
        let mut vec = static_vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string()
        ];

        let removed = vec.remove_swap(2);
        assert_eq!(removed, "3");
        assert_eq!(vec.len(), 3);
        assert!(vec.iter().eq(["1", "2", "4"]));
    }

    #[test]
    fn test_remove_swap_end() {
        let mut vec = static_vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string()
        ];

        let removed = vec.remove_swap(3);
        assert_eq!(removed, "4");
        assert_eq!(vec.len(), 3);
        assert!(vec.iter().eq(["1", "2", "3"]));
    }

    #[test]
    fn test_remove_swap_last() {
        let mut vec = static_vec!["1".to_string()];

        vec.remove_swap(0);

        assert!(vec.is_empty());
    }

    #[test]
    #[should_panic]
    fn test_remove_swap_panic() {
        let mut vec = static_vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string()
        ];

        vec.remove_swap(4);
    }

    #[test]
    fn test_resize_less() {
        let mut vec = static_vec![1, 2, 3, 4, 5];
        vec.resize(3, 0);

        assert!(vec.iter().eq(&[1, 2, 3]));
        assert_eq!(vec.len(), 3);
    }

    #[test]
    fn test_resize_equal() {
        let mut vec = static_vec![1, 2, 3, 4, 5];

        vec.resize(5, 0);

        assert!(vec.iter().eq(&[1, 2, 3, 4, 5]));
        assert_eq!(vec.len(), 5);
    }

    #[test]
    fn test_resize_higher() {
        let mut vec = static_vec![1, 2, 3, 4, 5; 10];
        vec.resize(7, 42);

        assert!(vec.iter().eq(&[1, 2, 3, 4, 5, 42, 42]));
        assert_eq!(vec.len(), 7);
    }

    #[test]
    #[should_panic]
    fn test_resize_over_capacity() {
        let mut vec = static_vec![1, 2, 3, 4, 5; 10];
        vec.resize(12, 42);
    }

    #[test]
    fn test_into_iter() {
        let vec = static_vec![1, 2, 3, 4; 10];
        let mut as_iter = vec.into_iter();
        assert_eq!(as_iter.next(), Some(1));
        assert_eq!(as_iter.next(), Some(2));
        assert_eq!(as_iter.next(), Some(3));
        assert_eq!(as_iter.next(), Some(4));
        assert_eq!(as_iter.next(), None);
    }
}

fn main() {}
