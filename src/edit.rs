/// Vector Edit operation
#[derive(Debug, PartialEq, Eq)]
pub enum Edit {
    Delete {
        dest_index: usize,
    },
    Replace {
        dest_index: usize,
        source_index: usize,
    },
    Insert {
        dest_index: usize,
        source_index: usize,
    },
}

impl PartialOrd for Edit {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Ordering of [`Edit`] operations for in place mofication of a destination vector.
/// This ensures a Vec of [`Edit`] are applied in the order Replace, Insert, Delete
impl Ord for Edit {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (
                Edit::Delete { dest_index },
                Edit::Delete {
                    dest_index: other_index,
                },
            ) => other_index.cmp(dest_index),
            (
                Edit::Insert { dest_index, .. },
                Edit::Insert {
                    dest_index: other_index,
                    ..
                },
            ) => other_index.cmp(dest_index),
            (
                Edit::Replace { dest_index, .. },
                Edit::Replace {
                    dest_index: other_index,
                    ..
                },
            ) => dest_index.cmp(other_index),

            // Replace take precedent over all other edits
            (Edit::Replace { .. }, _) => std::cmp::Ordering::Less,
            (_, Edit::Replace { .. }) => std::cmp::Ordering::Greater,

            (Edit::Insert { .. }, _) => std::cmp::Ordering::Less,
            (_, Edit::Insert { .. }) => std::cmp::Ordering::Greater,
        }
    }
}

pub fn _vec_apply_edits<T: Copy>(dest: &mut Vec<T>, source: &Vec<T>, edits: Vec<Edit>) {
    for edit in edits {
        match edit {
            Edit::Insert {
                dest_index,
                source_index,
            } => {
                dest.insert(dest_index, source[source_index]);
            }
            Edit::Delete { dest_index } => {
                dest.remove(dest_index);
            }
            Edit::Replace {
                dest_index,
                source_index,
            } => dest[dest_index] = source[source_index],
        }
    }
}

/// Find minimum edits required to dest to make it equal to source
pub fn vec_edits<T>(dest: &Vec<T>, source: &Vec<T>) -> Vec<Edit>
where
    T: std::fmt::Debug + PartialEq,
{
    let dest_len = dest.len();
    let source_len = source.len();

    // Matrix of edit distances
    let mut dist = vec![vec![0u64; source_len + 1]; dest_len + 1];

    for i in 0..=dest_len {
        dist[i][0] = i as u64;
    }
    for j in 0..=source_len {
        dist[0][j] = j as u64;
    }

    // Fill the matrix
    for (i, dest_hash) in dest.iter().enumerate() {
        for (j, source_hash) in source.iter().enumerate() {
            if dest_hash == source_hash {
                // No edit required, as the hashes match
                dist[i + 1][j + 1] = dist[i][j];
            } else {
                // Find the minimum of replace, delete, insert
                dist[i + 1][j + 1] = 1 + dist[i][j].min(dist[i + 1][j]).min(dist[i][j + 1]);
            }
        }
    }

    // Initialize (i,j) to the last element in the matrix
    let (mut i, mut j) = (dest_len, source_len);

    let mut edits = Vec::new();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && dest[i - 1] == source[j - 1] {
            i -= 1;
            j -= 1;
        } else if i > 0 && (j == 0 || dist[i][j] == dist[i - 1][j] + 1) {
            edits.push(Edit::Delete { dest_index: i - 1 });
            i -= 1;
        } else if j > 0 && (i == 0 || dist[i][j] == dist[i][j - 1] + 1) {
            edits.push(Edit::Insert {
                dest_index: i,
                source_index: j - 1,
            });
            j -= 1;
        } else if i > 0 && j > 0 {
            edits.push(Edit::Replace {
                dest_index: i - 1,
                source_index: j - 1,
            });
            i -= 1;
            j -= 1;
        }
    }

    // Sort the edits for in place application in the dest vec
    edits.sort();

    edits
}

#[cfg(test)]
mod tests {
    use crate::edit::_vec_apply_edits;

    use super::{vec_edits, Edit};

    #[test]
    /// Test sorting a Vec of Edit
    fn edit_order() {
        // Create a test vector with out of order Edits
        let mut edits: Vec<Edit> = vec![
            Edit::Insert {
                dest_index: 1,
                source_index: 0,
            },
            Edit::Delete { dest_index: 0 },
            Edit::Replace {
                dest_index: 3,
                source_index: 0,
            },
            Edit::Replace {
                dest_index: 1,
                source_index: 0,
            },
            Edit::Delete { dest_index: 1 },
            Edit::Insert {
                dest_index: 0,
                source_index: 0,
            },
        ];

        // Sort the edits
        edits.sort();

        println!("Sorted edits {edits:#?}");

        // Check the expected order after sorting
        for (index, edit) in edits.iter().enumerate() {
            match index {
                // Replace should come first, with the lowest index
                0 => assert_eq!(
                    *edit,
                    Edit::Replace {
                        dest_index: 1,
                        source_index: 0,
                    }
                ),
                1 => assert_eq!(
                    *edit,
                    Edit::Replace {
                        dest_index: 3,
                        source_index: 0,
                    }
                ),

                // Insert should come next with the higest index
                2 => assert_eq!(
                    *edit,
                    Edit::Insert {
                        dest_index: 1,
                        source_index: 0,
                    }
                ),
                3 => assert_eq!(
                    *edit,
                    Edit::Insert {
                        dest_index: 0,
                        source_index: 0,
                    }
                ),

                // Deletes should come last starting with highest index
                4 => assert_eq!(*edit, Edit::Delete { dest_index: 1 }),
                5 => assert_eq!(*edit, Edit::Delete { dest_index: 0 }),
                _ => panic!("missing match case for index"),
            }
        }
    }

    #[test]
    fn replace_one() {
        let mut dest = vec![1u64, 2, 3, 4];
        let source = vec![1u64, 2, 3, 5];

        let edits = vec_edits(&dest, &source);

        println!("Edits: {edits:#?}");
        assert_eq!(edits.len(), 1);

        _vec_apply_edits(&mut dest, &source, edits);
        assert_eq!(dest, source);
    }

    #[test]
    fn insert_one() {
        let mut dest = vec![1u64, 2, 3, 4];
        let source = vec![1u64, 2, 3, 6, 4];

        let edits = vec_edits(&dest, &source);

        println!("Edits: {edits:#?}");
        assert_eq!(edits.len(), 1);

        _vec_apply_edits(&mut dest, &source, edits);
        assert_eq!(dest, source);
    }

    #[test]
    fn insert_two() {
        let mut dest = vec![1u64, 2, 3, 4];
        let source = vec![1u64, 2, 5, 3, 6, 4];

        let edits = vec_edits(&dest, &source);

        println!("Edits: {edits:#?}");

        assert_eq!(edits.len(), 2);

        _vec_apply_edits(&mut dest, &source, edits);
        assert_eq!(dest, source);
    }

    #[test]
    fn delete_one() {
        let mut dest = vec![1u64, 2, 3, 4];
        let source = vec![1u64, 3, 4];

        let edits = vec_edits(&dest, &source);

        println!("Edits: {edits:#?}");

        assert_eq!(edits.len(), 1);
        _vec_apply_edits(&mut dest, &source, edits);
        assert_eq!(dest, source);
    }

    #[test]
    fn delete_two() {
        let mut dest = vec![1u64, 2, 3, 4];
        let source = vec![1u64, 4];

        let edits = vec_edits(&dest, &source);

        println!("Edits: {edits:#?}");

        assert_eq!(edits.len(), 2);
        _vec_apply_edits(&mut dest, &source, edits);
        assert_eq!(dest, source);
    }

    #[test]
    fn delete_replace() {
        let mut dest = vec![1u64, 2, 3, 4];
        let source = vec![1u64, 3, 3];

        let edits = vec_edits(&dest, &source);

        println!("Edits: {edits:#?}");

        assert_eq!(edits.len(), 2);
        _vec_apply_edits(&mut dest, &source, edits);
        assert_eq!(dest, source);
    }

    #[test]
    fn delete_replace_two() {
        let mut dest = vec![1u64, 2, 3, 4];
        let source = vec![1u64, 5];

        println!("Dest: {dest:?}");
        println!("Source: {source:?}");

        let edits = vec_edits(&dest, &source);

        println!("Edits: {edits:#?}");

        assert_eq!(edits.len(), 3);
        _vec_apply_edits(&mut dest, &source, edits);
        assert_eq!(dest, source);
    }
}
