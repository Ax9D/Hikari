pub struct Node<T> {
    data: T,
    next: *mut Node<T>,
    prev: *mut Node<T>,
}
impl<T> Node<T> {
    pub fn new_boxed(data: T) -> Box<Self> {
        Box::new(Self {
            data,
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        })
    }
    pub fn data(&self) -> &T {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
    unsafe fn join(first: *mut Node<T>, second: *mut Node<T>) {
        (*first).next = second;
        (*second).prev = first;
    }
}
pub struct IntrusiveLinkedList<T> {
    first: *mut Node<T>,
    last: *mut Node<T>,
    len: usize,
}

impl<T> IntrusiveLinkedList<T> {
    pub fn new() -> Self {
        Self {
            first: std::ptr::null_mut(),
            last: std::ptr::null_mut(),
            len: 0,
        }
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn append_node(&mut self, data: Box<Node<T>>) -> *mut Node<T> {
        let data_ptr = Box::into_raw(data);

        if self.is_empty() {
            self.first = data_ptr;
            self.last = data_ptr;
        } else {
            unsafe {
                Node::join(self.last, data_ptr);
                self.last = data_ptr;
            }
        }

        self.len += 1;

        data_ptr
    }
    pub fn prepend_node(&mut self, data: Box<Node<T>>) -> *mut Node<T> {
        let data_ptr = Box::into_raw(data);

        if self.is_empty() {
            self.first = data_ptr;
            self.last = data_ptr;
        } else {
            unsafe {
                Node::join(data_ptr, self.first);
                self.first = data_ptr;
            }
        }

        self.len += 1;

        data_ptr
    }
    pub fn remove_node(&mut self, node: *mut Node<T>) -> Box<Node<T>> {
        unsafe {
            assert!(!node.is_null());

            if self.first == node {
                self.first = (*self.first).next;
            } else if self.last == node {
                self.last = (*self.last).prev;
            } else {
                let prev = (*node).prev;
                let next = (*node).next;

                Node::join(prev, next);
            }

            self.len -= 1;
            Box::from_raw(node)
        }
    }
    pub fn iter(&self) -> Iter<T> {
        let current = self.first;

        Iter {
            list: self,
            current,
        }
    }
    pub fn drain(&mut self) -> Drain<T> {
        Drain { list: self }
    }
}
impl<T> Drop for IntrusiveLinkedList<T> {
    fn drop(&mut self) {
        self.drain();
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for IntrusiveLinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

pub struct Drain<'a, T> {
    list: &'a mut IntrusiveLinkedList<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.list.len() > 0 {
            let removed = self.list.remove_node(self.list.first);
            Some(removed.data)
        } else {
            None
        }
    }
}

pub struct Iter<'a, T> {
    list: &'a IntrusiveLinkedList<T>,
    current: *mut Node<T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            unsafe {
                let data = &(*self.current).data;
                let next = (*self.current).next;
                self.current = next;
                Some(data)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IntrusiveLinkedList, Node};

    #[test]
    fn append() {
        let mut list = IntrusiveLinkedList::new();
        list.append_node(Node::new_boxed(0));
        assert!(list.len() == 1);
        list.append_node(Node::new_boxed(1));
        assert!(list.len() == 2);
    }
    #[test]
    fn prepend() {
        let mut list = IntrusiveLinkedList::new();
        list.prepend_node(Node::new_boxed(0));
        assert!(list.len() == 1);
        list.prepend_node(Node::new_boxed(1));
        assert!(list.len() == 2);
    }
    #[test]
    fn remove() {
        let mut list = IntrusiveLinkedList::new();
        let zero = list.prepend_node(Node::new_boxed(0));
        let one = list.prepend_node(Node::new_boxed(1));
        let two = list.prepend_node(Node::new_boxed(2));
        assert!(list.len() == 3);
        list.remove_node(zero);
        list.remove_node(one);
        assert!(list.len() == 1);
        list.remove_node(two);
        assert!(list.len() == 0);

        let zero = list.prepend_node(Node::new_boxed(0));
        let one = list.prepend_node(Node::new_boxed(1));
        let two = list.prepend_node(Node::new_boxed(2));

        assert!(list.len() == 3);
        list.remove_node(two);
        list.remove_node(zero);
        assert!(list.len() == 1);
        list.remove_node(one);
        assert!(list.len() == 0);

        let zero = list.prepend_node(Node::new_boxed(0));
        let one = list.prepend_node(Node::new_boxed(1));
        let two = list.prepend_node(Node::new_boxed(2));

        assert!(list.len() == 3);
        list.remove_node(one);
        list.remove_node(two);
        assert!(list.len() == 1);
        list.remove_node(zero);
        assert!(list.len() == 0);
    }
    #[test]
    fn drain() {
        let mut list = IntrusiveLinkedList::new();
        list.append_node(Node::new_boxed(0));
        list.append_node(Node::new_boxed(1));
        list.append_node(Node::new_boxed(2));
        list.append_node(Node::new_boxed(3));

        assert!(list.len() == 4);

        for _ in list.drain() {}
        assert!(list.len() == 0);
    }
}
