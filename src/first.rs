use std::mem;

/// ## [First](https://rust-unofficial.github.io/too-many-lists/first.html)
///
/// **A primitive stack**. We push some numbers on it, and pop them off.
/// The list is implemented using a single linked list, which means that each node points to the next node in the list, and the last node points to None.
pub struct List {
    head: Link,
}

// hint: this is like an `Option` enum
enum Link {
    Empty,
    More(Box<Node>),
}

// not a generic type, just a list of i32s for now
struct Node {
    elem: i32,
    next: Link,
}

impl List {
    /// Initialize our list with an empty head.
    pub fn new() -> Self {
        List { head: Link::Empty }
    }

    /// To add an element, we create a new node with the element and the current head as its next node,
    /// and then we update the head to point to the new node.
    pub fn push(&mut self, elem: i32) {
        let new_node = Box::new(Node {
            elem,
            // mem::replace lets us steal a value out of a borrow by replacing it with another value
            // here, we replace head with an empty node (just for a moment)
            //
            // replace returns the destination right before we override it, so this "next"
            // will point to the head
            next: mem::replace(&mut self.head, Link::Empty),
        });

        // then, we replace the actual head (which is temporarily empty at this point)
        // with the new node we have just created
        self.head = Link::More(new_node);
    }

    /// To remove an element, we replace the head with an empty node, and handle the head case.
    pub fn pop(&mut self) -> Option<i32> {
        // we replace the head with an empty node, and handle the head case
        match mem::replace(&mut self.head, Link::Empty) {
            // head was empty, return
            Link::Empty => None,
            // head was not empty, transfer its next node
            // to the current head, and return the element within
            Link::More(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }
}

// like C++ destructor, will run when our guy is out of scope
impl Drop for List {
    fn drop(&mut self) {
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);
        // `while let` == "do this thing until this pattern doesn't match"
        while let Link::More(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
            // boxed_node goes out of scope and gets dropped here;
            // but its Node's `next` field has been set to Link::Empty
            // so no unbounded recursion occurs.
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // check empty list behaves right
        assert_eq!(list.pop(), None);

        // populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);

        // going beyond exhaustion does not crash anything
        assert_eq!(list.pop(), None);
    }
}
