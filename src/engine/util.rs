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

#[derive(Default)]
pub struct Rect<T> {
    pub left: T,
    pub top: T,
    pub width: T,
    pub height: T,
}

impl<
        T: std::ops::Add<Output = T>
            + std::cmp::PartialOrd<<T as std::ops::Add>::Output>
            + Clone
            + Copy,
    > Rect<T>
{
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

    #[test]
    fn test_overlap() {
        let r1 = Rect::<i32>::new(10, 10, 10, 10);
        let r2 = Rect::<i32>::new(21, 10, 10, 10);
        assert!(!r1.overlaps(&r2));
        assert!(!r2.overlaps(&r1));

        let r3 = Rect::<i32>::new(19, 10, 10, 10);
        assert!(r1.overlaps(&r3));
        assert!(r3.overlaps(&r1));

        let r4 = Rect::<i32>::new(10, 21, 10, 10);
        assert!(!r1.overlaps(&r4));
        assert!(!r4.overlaps(&r1));

        let r5 = Rect::<i32>::new(10, 19, 10, 10);
        assert!(r1.overlaps(&r5));
        assert!(r5.overlaps(&r1));
    }
}
