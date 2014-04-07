use core::*;
use core::str::*;
use core::option::{Some, Option, None}; 
use core::iter::Iterator;
use kernel::*;
use kernel::memory::Allocator;

struct List {
    head : Option<~FileNode>,
    length : uint
}

impl List {
    unsafe fn new() -> List {
        List {
            head : None,
            length : 0
        } 
    }

    unsafe fn add(node: &List, file : ~FileNode) {
        match node.head {
            None    => { self.head = Some(file); }
            Some(x)      => { 
                        let mut prev = self.head;
                        loop { 
                            match temp {
                                None => { break; }
                                Some(y)   => {
                                    prev = temp;
                                    temp = y.next;}
                            }
                        }
                        match prev {
                            None => { ""; }
                            Some(y) => {y.next = Some(file);}
                        }
                        self.length += 1;}
        }
    }

    unsafe fn remove(index : uint) {

    }
}

pub struct FileNode {
    filename : strings::cstr,
    data : strings::cstr,
    children : Option<~List>,
    next : Option<~FileNode>
}

impl FileNode {
    pub unsafe fn new(name : strings::cstr, contents : strings::cstr) -> FileNode{
        FileNode {
            filename : name,
            data : contents, 
            children : None, 
            next : None
        }
    }

    pub unsafe fn add(&mut self, name : strings::cstr, contents : strings::cstr) {
        let temp = ~FileNode::new(name, contents);
        match self.children {
            None    => { let t = ~List::new();
                        t.add(temp);
                        self.children = Some(t);}
            Some(y) =>  { y.add(temp); }
        }
    }

    pub unsafe fn add2(&mut self, file : ~FileNode) {
        match self.next {
            None    => { self.next = Some(file) }
            Some(x) => { x.add2(file); }
        }
        }
    }
}