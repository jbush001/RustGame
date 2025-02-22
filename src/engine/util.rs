//
// Copyright 2025 Jeff Bush
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

#[derive(Clone, Copy, Default, Debug)]
pub struct Rect<T> {
    pub left: T,
    pub top: T,
    pub width: T,
    pub height: T,
}

impl<T: std::ops::Add<Output = T> + std::cmp::PartialOrd<T> + Clone + Copy> Rect<T> {
    pub fn new(left: T, top: T, width: T, height: T) -> Rect<T> {
        Rect {
            left,
            top,
            width,
            height,
        }
    }

    pub fn overlaps(&self, rect: &Rect<T>) -> bool {
        self.left < rect.right()
            && rect.left < self.right()
            && self.top < rect.bottom()
            && rect.top < self.bottom()
    }

    pub fn right(&self) -> T {
        self.left + self.width
    }

    pub fn bottom(&self) -> T {
        self.top + self.height
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // +--------+--------+--------+
    // |   r1   |   r2   |   r3   |
    // +--------+--------+--------+
    // |   r4   |        |   r6   |
    // +--------+--------+--------+
    // |   r7   |   r8   |   r9   |
    // +--------+--------+--------+
    #[test]
    fn test_overlap() {
        let r1 = Rect::<i32>::new(0, 0, 10, 10);
        let r2 = Rect::<i32>::new(10, 0, 10, 10);
        let r3 = Rect::<i32>::new(20, 0, 10, 10);
        let r4 = Rect::<i32>::new(0, 10, 10, 10);
        let r6 = Rect::<i32>::new(20, 10, 10, 10);
        let r7 = Rect::<i32>::new(0, 20, 10, 10);
        let r8 = Rect::<i32>::new(10, 20, 10, 10);
        let r9 = Rect::<i32>::new(20, 20, 10, 10);

        let middle_large = Rect::<i32>::new(9, 9, 12, 12);
        assert!(r1.overlaps(&middle_large));
        assert!(r2.overlaps(&middle_large));
        assert!(r3.overlaps(&middle_large));
        assert!(r4.overlaps(&middle_large));
        assert!(r6.overlaps(&middle_large));
        assert!(r7.overlaps(&middle_large));
        assert!(r8.overlaps(&middle_large));
        assert!(r9.overlaps(&middle_large));

        let middle_small = Rect::<i32>::new(11, 11, 8, 8);
        assert!(!r1.overlaps(&middle_small));
        assert!(!r2.overlaps(&middle_small));
        assert!(!r3.overlaps(&middle_small));
        assert!(!r4.overlaps(&middle_small));
        assert!(!r6.overlaps(&middle_small));
        assert!(!r7.overlaps(&middle_small));
        assert!(!r8.overlaps(&middle_small));
        assert!(!r9.overlaps(&middle_small));
    }

    #[test]
    fn test_bounds() {
        let r1 = Rect::<i32>::new(2, 3, 4, 5);
        assert_eq!(r1.right(), 6);
        assert_eq!(r1.bottom(), 8);
    }

    #[test]
    fn test_copy_clone() {
        let r1 = Rect::<i32>::new(1, 2, 3, 4);
        let r2 = r1;
        let r3 = r1.clone();
        assert_eq!(r2.left, 1);
        assert_eq!(r3.left, 1);
        assert_eq!(r2.top, 2);
        assert_eq!(r3.top, 2);
        assert_eq!(r2.width, 3);
        assert_eq!(r3.width, 3);
        assert_eq!(r2.height, 4);
        assert_eq!(r3.height, 4);
    }

    #[test]
    fn test_debug() {
        let r1 = Rect::<i32>::new(1, 2, 3, 4);
        assert_eq!(
            "Rect { left: 1, top: 2, width: 3, height: 4 }",
            format!("{:?}", r1)
        );
    }
}
