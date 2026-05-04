/// ## [Second](https://rust-unofficial.github.io/too-many-lists/second.html)
///
/// A much better **stack** compared to the one in [`crate::first`].
///
/// Now uses Rust primitives like [`Option`] and [`Option::map`] instead of reinventing the wheel, and also supports peeking and iterating over the list.
pub struct List<T> {
    head: Link<T>,
}

/// We reinvented the "Option" wheel in the previous level (see below),
/// no need to do that!
///
/// ```rs
/// enum Link {
///    Empty,
///    More(Box<Node>),
/// }
/// ```
///
/// Here, `Empty` is just `None`, and `More(Box<Node>)` is just `Some(Box<Node>)`.
type Link<T> = Option<Box<Node<T>>>;

/// A typical linked list node:
/// - it holds an element of type T
/// - it holds a pointer to the next node in the list (or None if it's the end of the list)
struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    /// We create the list with a `None` head, which signifies an empty list.
    pub fn new() -> Self {
        List { head: None }
    }

    /// To add an element, we create a new node with the element and the current head as its next node,
    /// and then we update the head to point to the new node.
    ///
    /// ```sh
    /// # before
    /// (top) node -> node -> None (bottom)
    ///      (head)
    ///
    /// # after
    /// (top) node* -> node -> node -> None (bottom)
    ///      (head) (prev-head)
    /// ```
    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem,
            // `mem::replace(&mut option, None)` is such an incredibly common
            // idiom that Option actually just went ahead and made it a method: take.
            //
            // this will move `self.head` out of `self` and send it to the `next` field of the new node, and replace `self.head` with `None`
            next: self.head.take(),
        });

        // at this point head is `None`, so we can just put the new node in there
        self.head = Some(new_node);
    }

    /// To pop an element, we take the head, and if it's not `None`,
    /// we replace the head with the next node, and return the element of the old head.
    pub fn pop(&mut self) -> Option<T> {
        // `match option { None => None, Some(x) => Some(y) }` is such an incredibly common idiom that it was called map
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    //// peek ////

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        // instead of reference with `as_ref`, return a mutable reference with `as_mut`
        self.head.as_mut().map(|node| &mut node.elem)
    }

    //// iterators ////

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    // the '_ here is to notify the reader that this
    // is an "explicitly elided lifetime"
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}

// Into-Iterator implementations
// will take own the values as it iterates
pub struct IntoIter<T>(List<T>);
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // access fields of a tuple struct numerically
        self.0.pop()
    }
}

// Iterator implementations
// will return a pointer to the underlying value as it iterates
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

// Mutable Iterator implementations
// will return mutable references to each item
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}
impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        // `Copy`. when we did `self.next.map` it was fine because the `Option` was just copied.
        //
        // now we can't do that, because &mut isn't `Copy` (if you copied an &mut, you'd have
        // two &mut's to the same location in memory, which is forbidden).
        // Instead, we should properly take the Option to get it.
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));

        list.peek_mut().map(|value| *value = 42);

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
    }
}
