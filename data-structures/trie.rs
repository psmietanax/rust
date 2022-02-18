use std::collections::HashMap;

#[derive(Debug)]
struct Trie {
    node: Box<TrieNode>
}

impl Trie {
    fn new() -> Self {
        Trie { node: Box::new(TrieNode::new('.')) }
    }
    fn insert(&mut self, str: String) {
        let mut node = self.node.as_mut();
        for c in str.chars() {
            node = node.insert(c);
        }
    }
    fn find(&self, str: String) -> bool {
        let mut node = self.node.as_ref();
        for c in str.chars() {
            match node.child_nodes.get(&c) {
                Some(child_node) => {
                    println!("{:?}", child_node);
                    node = child_node;
                },
                None => {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug)]
struct TrieNode {
    value: char,
    child_nodes: HashMap<char, Box<TrieNode>>
}

impl TrieNode {
    fn new(c: char) -> Self {
        TrieNode { value: c, child_nodes: HashMap::new() }
    }
    fn insert(&mut self, c: char) -> &mut Box<TrieNode> {
        if !self.child_nodes.contains_key(&c) {
            self.child_nodes.insert(c, Box::new(TrieNode::new(c)));
        }
        self.child_nodes.get_mut(&c).unwrap()
    }
    fn contains(&self, c: char) -> bool {
        self.child_nodes.contains_key(&c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie() {
        let mut trie = Trie::new();
        trie.insert("there".to_string());
        trie.insert("their".to_string());
        trie.insert("any".to_string());
        trie.insert("answer".to_string());

        assert_eq!(trie.find("t".to_string()), true);
        assert_eq!(trie.find("th".to_string()), true);
        assert_eq!(trie.find("there".to_string()), true);
        assert_eq!(trie.find("their".to_string()), true);
        assert_eq!(trie.find("a".to_string()), true);
        assert_eq!(trie.find("an".to_string()), true);
        assert_eq!(trie.find("any".to_string()), true);
        assert_eq!(trie.find("answer".to_string()), true);

        assert_eq!(trie.find("test".to_string()), false);
        assert_eq!(trie.find("answers".to_string()), false);
        assert_eq!(trie.find("theirs".to_string()), false);
    }

}
