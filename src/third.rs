use std::rc::Rc;

/// ## [Third](https://rust-unofficial.github.io/too-many-lists/third.html)
///
/// Now, unlike the [`crate::second`] this one supports sharing nodes between multiple lists,
/// which means that we can have multiple lists that share the same tail.
/// This is achieved by using reference counting with `Rc`.
///
/// However, this implementation is still immutable, which means:
/// - we cannot modify the list after it's created
/// - we cannot have mutable references to the nodes, no `IterMut`!
///
/// We simply want to be able to do stuff like:
///
/// ```sh
/// list1 = A -> B -> C -> D
/// list2 = tail(list1) = B -> C -> D
/// list3 = push(list2, X) = X -> B -> C -> D
/// ```
///
/// Which in effect should result in the following memory layout:
///
/// ```sh
/// list1 -> A ---+
///               |
///               v
/// list2 ------> B -> C -> D
///               ^
///               |
/// list3 -> X ---+
///
pub struct List<T> {
    head: Link<T>,
}

/// Now we have a reference-counted pointer to a node, which means that we can have multiple lists that share the same tail.
type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    /// Initialize our list with an empty head.
    pub fn new() -> Self {
        List { head: None }
    }

    /// Prepend a new element to the list, and return the new list.
    ///
    /// It creates a new node and wraps it in an `Rc`, and then it creates a new list with the new node as the head, and the old head as the next node.
    pub fn prepend(&self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                elem: elem,
                // cant .take() here because the underlying data is shared (via `Rc`)
                next: self.head.clone(),
            })),
        }
    }

    /// Return the tail of the list, which is a new list with the head of the current list as its next node.
    ///
    /// It's just a "pointer" to the next node, so it doesn't actually copy the data, and it can be shared between multiple lists.
    pub fn tail(&self) -> List<T> {
        List {
            head: self.head.as_ref().and_then(|node| node.next.clone()),
        }
    }

    /// Reference to the `head` value.
    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }

    /// Return an iterator over the list.
    ///
    /// [`Iter`] is defined below.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

/// An iterator implementor for our [`List`].
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);
    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}
