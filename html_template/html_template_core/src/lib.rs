/// A node represents a segment of the template that cannot be reduced further
pub enum Node<'a> {
    List(Vec<Node<'a>>),
    Str(&'a str),
    Fn(Box<dyn Fn() -> String>)
}

impl<'a> std::fmt::Debug for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            Self::Str(arg0) => f.debug_tuple("Str").field(arg0).finish(),
            Self::Fn(_) => f.debug_tuple("Fn").finish(),
        }
    }
}

impl<'a> std::fmt::Display for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Str(s) => f.write_str(s)?,
            Node::List(l) => for n in l {
                n.fmt(f)?;
            },
            Node::Fn(func) => func().fmt(f)?,
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Root<'a> {
    pub root: Node<'a>
}

impl<'a> std::fmt::Display for Root<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.fmt(f)
    }
}

impl<'a> From<Node<'a>> for Root<'a> {
    fn from(value: Node<'a>) -> Self {
        Root {
            root: value
        }
    }
}

impl<'a, I: Iterator<Item=Node<'a>>> From<I> for Node<'a> {
    fn from(value: I) -> Self {
        Node::List(value.collect())
    }
}

impl<'a> FromIterator<Node<'a>> for Node<'a> {
    fn from_iter<T: IntoIterator<Item = Node<'a>>>(iter: T) -> Self {
        Node::List(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<Node<'a>> for String {
    fn from_iter<T: IntoIterator<Item = Node<'a>>>(iter: T) -> Self {
        let mut string = String::new();
        for node in iter {
            string.push_str(&node.to_string())
        }
        string
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn sanity() {
        let root = Root {
            root: Node::List(vec![
                      Node::Str("Hello "),
                      Node::Fn(Box::new(|| "world".into())),
                      Node::List(vec![
                          Node::Str("!"),
                      ]),
            ])
        };
        let expected = "Hello world!";

        let out = root.to_string();
        assert_eq!(expected, out);
    }
}
