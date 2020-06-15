// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

#[cfg(test)]
mod tests {

    use bee_common::WaitPriorityQueue;

    use std::cmp::Ordering;

    use async_std::task::block_on;

    #[derive(Eq, PartialEq, Debug)]
    pub(crate) struct TestMinHeapEntry(u64, char);

    impl PartialOrd for TestMinHeapEntry {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            other.0.partial_cmp(&self.0)
        }
    }

    impl Ord for TestMinHeapEntry {
        fn cmp(&self, other: &Self) -> Ordering {
            other.0.cmp(&self.0)
        }
    }

    #[test]
    fn min_heap() {
        let queue = WaitPriorityQueue::default();

        queue.insert(TestMinHeapEntry(5, 'F'));
        queue.insert(TestMinHeapEntry(1, 'B'));
        queue.insert(TestMinHeapEntry(9, 'J'));
        queue.insert(TestMinHeapEntry(0, 'A'));
        queue.insert(TestMinHeapEntry(7, 'H'));
        queue.insert(TestMinHeapEntry(6, 'G'));
        queue.insert(TestMinHeapEntry(2, 'C'));
        queue.insert(TestMinHeapEntry(3, 'D'));
        queue.insert(TestMinHeapEntry(8, 'I'));
        queue.insert(TestMinHeapEntry(4, 'E'));

        assert_eq!(block_on(queue.pop()), TestMinHeapEntry(0, 'A'));
        assert_eq!(block_on(queue.pop()), TestMinHeapEntry(1, 'B'));
        assert_eq!(block_on(queue.pop()), TestMinHeapEntry(2, 'C'));
        assert_eq!(block_on(queue.pop()), TestMinHeapEntry(3, 'D'));
        assert_eq!(block_on(queue.pop()), TestMinHeapEntry(4, 'E'));
        assert_eq!(block_on(queue.pop()), TestMinHeapEntry(5, 'F'));
        assert_eq!(block_on(queue.pop()), TestMinHeapEntry(6, 'G'));
        assert_eq!(block_on(queue.pop()), TestMinHeapEntry(7, 'H'));
        assert_eq!(block_on(queue.pop()), TestMinHeapEntry(8, 'I'));
        assert_eq!(block_on(queue.pop()), TestMinHeapEntry(9, 'J'));
    }
}
