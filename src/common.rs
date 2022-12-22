use bitvector::BitVector;

pub type Point = (i32, i32);
pub type IndexMatch = (usize, usize);

pub trait Matchable {
    fn distance(&self, other: &Self) -> usize;
}

pub fn match_indices<T>(vec1: &Vec<T>, vec2: &Vec<T>) -> Vec<IndexMatch>
where
    T: Matchable,
{
    assert_eq!(vec1.len(), vec2.len());

    let mut index_vec = vec![];
    let len = vec1.len();
    let mut matched_indices = BitVector::new(len);

    for i in 0..len {
        let mut min_dist: usize = usize::MAX;
        let mut matched_index: usize = 0;
        for j in 0..len {
            if matched_indices.contains(j) {
                continue;
            }

            let dist = vec1[i].distance(&vec2[j]);
            if dist < min_dist {
                min_dist = dist;
                matched_index = j;
            }
        }

        index_vec.push((i as usize, matched_index as usize));
        matched_indices.insert(matched_index);
    }

    index_vec
}

pub fn adaptive_nonmax_suppression<T>(vec: &mut Vec<T>, n: usize) -> Vec<T>
where
    T: Matchable,
    T: Copy,
{
    assert!(n <= vec.len());

    let mut maximal_keypoints: Vec<T> = vec![];
    for i in 1..vec.len() - 1 {
        let d1 = &vec[i];
        let mut min_dist: usize = usize::MAX;
        let mut min_idx: usize = 0;

        for j in 0..i {
            let d0 = &vec[j];
            let dist = d0.distance(&d1);
            if dist < min_dist {
                min_dist = dist;
                min_idx = j;
            }
        }

        vec.swap(i, min_idx);
    }

    for k in 0..n {
        maximal_keypoints.push(vec[k]);
    }

    maximal_keypoints
}
