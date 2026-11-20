//! `core/templates/sort_array.h`: the engine's introsort.
//!
//! Ported operation for operation: median-of-three pivot selection,
//! the Hoare-style `partitioner` (tracking the pivot slot across swaps), the
//! 16-element introsort threshold with the depth-limited heapsort fallback,
//! heap construction/sort for `partial_sort`, insertion sort (plain, linear and
//! unguarded) for the final pass, `introselect` for `nth_element`, and the
//! `sort`/`sort_range` entry points.
//!
//! The comparator is `Fn(&T, &T) -> bool` (a strict weak ordering, like the
//! engine's `Comparator::operator()`). The debug "bad comparison" checks are not
//! modelled.

pub const INTROSORT_THRESHOLD: i64 = 16;

fn compare<T, F: Fn(&T, &T) -> bool>(cmp: &F, a: &T, b: &T) -> bool {
    cmp(a, b)
}

fn median_of_3_index<T, F: Fn(&T, &T) -> bool>(
    array: &[T],
    a_index: i64,
    b_index: i64,
    c_index: i64,
    cmp: &F,
) -> i64 {
    let a = &array[a_index as usize];
    let b = &array[b_index as usize];
    let c = &array[c_index as usize];
    if compare(cmp, a, b) {
        if compare(cmp, b, c) {
            b_index
        } else if compare(cmp, a, c) {
            c_index
        } else {
            a_index
        }
    } else if compare(cmp, a, c) {
        a_index
    } else if compare(cmp, b, c) {
        c_index
    } else {
        b_index
    }
}

fn bitlog(mut n: i64) -> i64 {
    let mut k = 0;
    while n != 1 {
        n >>= 1;
        k += 1;
    }
    k
}

fn push_heap<T, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    first: i64,
    mut hole: i64,
    top: i64,
    value: T,
    cmp: &F,
) where
    T: Clone,
{
    let mut parent = (hole - 1) / 2;
    while hole > top && compare(cmp, &array[(first + parent) as usize], &value) {
        array[(first + hole) as usize] = array[(first + parent) as usize].clone();
        hole = parent;
        parent = (hole - 1) / 2;
    }
    array[(first + hole) as usize] = value;
}

fn adjust_heap<T, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    first: i64,
    mut hole: i64,
    len: i64,
    value: T,
    cmp: &F,
) where
    T: Clone,
{
    let top = hole;
    let mut second_child = 2 * hole + 2;
    while second_child < len {
        if compare(
            cmp,
            &array[(first + second_child) as usize],
            &array[(first + second_child - 1) as usize],
        ) {
            second_child -= 1;
        }
        array[(first + hole) as usize] = array[(first + second_child) as usize].clone();
        hole = second_child;
        second_child = 2 * (second_child + 1);
    }
    if second_child == len {
        array[(first + hole) as usize] = array[(first + second_child - 1) as usize].clone();
        hole = second_child - 1;
    }
    push_heap(array, first, hole, top, value, cmp);
}

fn pop_heap_at<T, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    first: i64,
    last: i64,
    result: i64,
    value: T,
    cmp: &F,
) where
    T: Clone,
{
    array[result as usize] = array[first as usize].clone();
    adjust_heap(array, first, 0, last - first, value, cmp);
}

fn pop_heap<T, F: Fn(&T, &T) -> bool>(array: &mut [T], first: i64, mut last: i64, cmp: &F)
where
    T: Clone,
{
    let value = array[(last - 1) as usize].clone();
    last -= 1;
    pop_heap_at(array, first, last, last, value, cmp);
}

fn sort_heap<T, F: Fn(&T, &T) -> bool>(array: &mut [T], first: i64, mut last: i64, cmp: &F)
where
    T: Clone,
{
    while last - first > 1 {
        pop_heap(array, first, last, cmp);
        last -= 1;
    }
}

fn make_heap<T, F: Fn(&T, &T) -> bool>(array: &mut [T], first: i64, last: i64, cmp: &F)
where
    T: Clone,
{
    if last - first < 2 {
        return;
    }
    let len = last - first;
    let mut parent = (len - 2) / 2;
    loop {
        let value = array[(first + parent) as usize].clone();
        adjust_heap(array, first, parent, len, value, cmp);
        if parent == 0 {
            return;
        }
        parent -= 1;
    }
}

/// `partial_sort`: the first `middle - first` elements end up sorted and are
/// the smallest ones.
pub fn partial_sort<T: Clone, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    first: i64,
    last: i64,
    middle: i64,
    cmp: &F,
) {
    make_heap(array, first, middle, cmp);
    for i in middle..last {
        if compare(cmp, &array[i as usize], &array[first as usize]) {
            let value = array[i as usize].clone();
            pop_heap_at(array, first, middle, i, value, cmp);
        }
    }
    sort_heap(array, first, middle, cmp);
}

fn partial_select<T: Clone, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    first: i64,
    last: i64,
    middle: i64,
    cmp: &F,
) {
    make_heap(array, first, middle, cmp);
    for i in middle..last {
        if compare(cmp, &array[i as usize], &array[first as usize]) {
            let value = array[i as usize].clone();
            pop_heap_at(array, first, middle, i, value, cmp);
        }
    }
}

fn partitioner<T, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    mut first: i64,
    mut last: i64,
    mut pivot: i64,
    cmp: &F,
) -> i64 {
    loop {
        while first != pivot && compare(cmp, &array[first as usize], &array[pivot as usize]) {
            first += 1;
        }
        last -= 1;
        while last != pivot && compare(cmp, &array[pivot as usize], &array[last as usize]) {
            last -= 1;
        }
        if first >= last {
            return first;
        }
        if pivot == first {
            pivot = last;
        } else if pivot == last {
            pivot = first;
        }
        array.swap(first as usize, last as usize);
        first += 1;
    }
}

fn introsort<T: Clone, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    mut first: i64,
    mut last: i64,
    mut max_depth: i64,
    cmp: &F,
) {
    while last - first > INTROSORT_THRESHOLD {
        if max_depth == 0 {
            partial_sort(array, first, last, last, cmp);
            return;
        }
        max_depth -= 1;
        let pivot = median_of_3_index(array, first, first + (last - first) / 2, last - 1, cmp);
        let cut = partitioner(array, first, last, pivot, cmp);
        introsort(array, cut, last, max_depth, cmp);
        last = cut;
    }
}

fn introselect<T: Clone, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    mut first: i64,
    nth: i64,
    mut last: i64,
    mut max_depth: i64,
    cmp: &F,
) {
    while last - first > 3 {
        if max_depth == 0 {
            partial_select(array, first, nth + 1, last, cmp);
            array.swap(first as usize, nth as usize);
            return;
        }
        max_depth -= 1;
        let pivot = median_of_3_index(array, first, first + (last - first) / 2, last - 1, cmp);
        let cut = partitioner(array, first, last, pivot, cmp);
        if cut <= nth {
            first = cut;
        } else {
            last = cut;
        }
    }
    insertion_sort(array, first, last, cmp);
}

fn unguarded_linear_insert<T, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    mut last: i64,
    value: T,
    cmp: &F,
) where
    T: Clone,
{
    let mut next = last - 1;
    while compare(cmp, &value, &array[next as usize]) {
        array[last as usize] = array[next as usize].clone();
        last = next;
        next -= 1;
    }
    array[last as usize] = value;
}

fn linear_insert<T, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    first: i64,
    last: i64,
    cmp: &F,
) where
    T: Clone,
{
    let value = array[last as usize].clone();
    if compare(cmp, &value, &array[first as usize]) {
        let mut i = last;
        while i > first {
            array[i as usize] = array[(i - 1) as usize].clone();
            i -= 1;
        }
        array[first as usize] = value;
    } else {
        unguarded_linear_insert(array, last, value, cmp);
    }
}

/// `insertion_sort` over `[first, last)`.
pub fn insertion_sort<T: Clone, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    first: i64,
    last: i64,
    cmp: &F,
) {
    if first == last {
        return;
    }
    for i in first + 1..last {
        linear_insert(array, first, i, cmp);
    }
}

fn unguarded_insertion_sort<T: Clone, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    first: i64,
    last: i64,
    cmp: &F,
) {
    for i in first..last {
        let value = array[i as usize].clone();
        unguarded_linear_insert(array, i, value, cmp);
    }
}

fn final_insertion_sort<T: Clone, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    first: i64,
    last: i64,
    cmp: &F,
) {
    if last - first > INTROSORT_THRESHOLD {
        insertion_sort(array, first, first + INTROSORT_THRESHOLD, cmp);
        unguarded_insertion_sort(array, first + INTROSORT_THRESHOLD, last, cmp);
    } else {
        insertion_sort(array, first, last, cmp);
    }
}

/// `sort_range`.
pub fn sort_range<T: Clone, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    first: i64,
    last: i64,
    cmp: &F,
) {
    if first == last {
        return;
    }
    introsort(array, first, last, bitlog(last - first) * 2, cmp);
    final_insertion_sort(array, first, last, cmp);
}

/// `sort`: the whole slice.
pub fn sort<T: Clone, F: Fn(&T, &T) -> bool>(array: &mut [T], cmp: &F) {
    sort_range(array, 0, array.len() as i64, cmp);
}

/// `nth_element`: after the call `array[nth]` is the element that would be there
/// in sorted order, with smaller elements before it.
pub fn nth_element<T: Clone, F: Fn(&T, &T) -> bool>(
    array: &mut [T],
    first: i64,
    last: i64,
    nth: i64,
    cmp: &F,
) {
    if first == last || nth == last {
        return;
    }
    introselect(array, first, nth, last, bitlog(last - first) * 2, cmp);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    fn ascending(a: &i32, b: &i32) -> bool {
        a < b
    }

    fn descending(a: &i32, b: &i32) -> bool {
        a > b
    }

    #[test]
    fn sorts_ascending_and_descending() {
        let mut values: Vec<i32> = (0..200).rev().collect();
        sort(&mut values, &ascending);
        assert_eq!(values, (0..200).collect::<Vec<_>>());

        let mut values: Vec<i32> = (0..200).collect();
        sort(&mut values, &descending);
        assert_eq!(values, (0..200).rev().collect::<Vec<_>>());

        // Already sorted, duplicates and small ranges.
        let mut values = vec![1, 1, 1, 2, 2, 0];
        sort(&mut values, &ascending);
        assert_eq!(values, vec![0, 1, 1, 1, 2, 2]);

        let mut values = vec![]
            ;
        sort(&mut values, &ascending);
        let mut values = vec![42];
        sort(&mut values, &ascending);
        assert_eq!(values, vec![42]);
        let mut values = vec![3, 1, 2];
        sort(&mut values, &ascending);
        assert_eq!(values, vec![1, 2, 3]);

        // sort_range only touches the requested window.
        let mut values = vec![9, 5, 1, 4, 3];
        sort_range(&mut values, 1, 4, &ascending);
        assert_eq!(values, vec![9, 1, 4, 5, 3]);

        // Sorting structs by key.
        let mut pairs = vec![(3, "c"), (1, "a"), (2, "b")];
        sort(&mut pairs, &|a: &(i32, &str), b: &(i32, &str)| a.0 < b.0);
        assert_eq!(pairs, vec![(1, "a"), (2, "b"), (3, "c")]);
    }

    #[test]
    fn heap_and_partial_sort() {
        let mut values = vec![5, 1, 4, 2, 3, 9, 0];
        let len = values.len() as i64;
        partial_sort(&mut values, 0, len, 3, &ascending);
        assert_eq!(&values[..3], &[0, 1, 2]);
        // The rest is not necessarily sorted but contains the remaining values.
        let mut rest = values[3..].to_vec();
        rest.sort();
        assert_eq!(rest, vec![3, 4, 5, 9]);

        let mut values = vec![1, 2, 3];
        make_heap(&mut values, 0, 3, &ascending);
        // The heap's front is the maximum for this comparator.
        assert_eq!(values[0], 3);
        sort_heap(&mut values, 0, 3, &ascending);
        assert_eq!(values, vec![1, 2, 3]);

        let mut values = vec![4, 2, 7];
        insertion_sort(&mut values, 0, 3, &ascending);
        assert_eq!(values, vec![2, 4, 7]);
    }

    #[test]
    fn nth_element_places_the_pivot() {
        for nth in 0..10 {
            let mut values = vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
            let len = values.len() as i64;
            nth_element(&mut values, 0, len, nth, &ascending);
            assert_eq!(values[nth as usize], nth as i32);
            assert!(values[..nth as usize].iter().all(|value| *value < nth as i32));
            assert!(values[nth as usize + 1..]
                .iter()
                .all(|value| *value >= nth as i32));
        }

        // A large input exercises the introselect partition/fallback paths.
        let mut values: Vec<i32> = (0..1000).map(|i| ((i * 37) % 1000) as i32).collect();
        let len = values.len() as i64;
        nth_element(&mut values, 0, len, 500, &ascending);
        assert_eq!(values[500], 500);
        let mut before = values[..500].to_vec();
        before.sort();
        assert_eq!(before, (0..500).collect::<Vec<_>>());
    }

    #[test]
    fn bitlog_and_median() {
        assert_eq!(bitlog(1), 0);
        assert_eq!(bitlog(2), 1);
        assert_eq!(bitlog(1024), 10);
        assert_eq!(bitlog(1000), 9);

        let values = vec![5, 1, 9];
        // The median of {5, 1, 9} is 5 at index 0.
        assert_eq!(median_of_3_index(&values, 0, 1, 2, &ascending), 0);
        // Index order does not change which value is the median.
        assert_eq!(median_of_3_index(&values, 2, 1, 0, &ascending), 0);
        // Descending: the median of {9, 5, 1} is 5 at index 1.
        let values = vec![9, 5, 1];
        assert_eq!(median_of_3_index(&values, 0, 1, 2, &descending), 1);
    }
}
