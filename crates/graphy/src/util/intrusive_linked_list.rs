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
            debug_assert!(!node.is_null());

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
        let current = self.first;

        Drain {
            list: self,
            current,
        }
    }
}
impl<T> Drop for IntrusiveLinkedList<T> {
    fn drop(&mut self) {
        self.drain();
    }
}

pub struct Drain<'a, T> {
    list: &'a mut IntrusiveLinkedList<T>,
    current: *mut Node<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == std::ptr::null_mut() {
            None
        } else {
            unsafe {
                let data = Box::from_raw(self.current).data;
                let next = (*self.current).next;
                self.current = next;
                Some(data)
            }
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
        if self.current == std::ptr::null_mut() {
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
