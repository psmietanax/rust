use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode {
  pub val: i32,
  pub left: Option<Rc<RefCell<TreeNode>>>,
  pub right: Option<Rc<RefCell<TreeNode>>>,
}

pub fn inorder(root: Option<Rc<RefCell<TreeNode>>>) -> Vec<i32> {
    fn _inorder(mut root: Option<Rc<RefCell<TreeNode>>>, vec: &mut Vec<i32>) {
        if let Some(node) = root {
            _inorder(node.borrow().left.as_ref().map(|x| Rc::clone(x)), vec);
            vec.push(node.borrow().val.clone());
            _inorder(node.borrow().right.as_ref().map(|x| Rc::clone(x)), vec);
        }
    }
    let mut result = Vec::new();
    _inorder(root, &mut result);
    result
}

pub fn preorder(root: Option<Rc<RefCell<TreeNode>>>) -> Vec<i32> {
    fn _preorder(mut root: Option<Rc<RefCell<TreeNode>>>, vec: &mut Vec<i32>) {
        if let Some(node) = root {
            vec.push(node.borrow().val.clone());
            _preorder(node.borrow().left.as_ref().map(|x| Rc::clone(x)), vec);
            _preorder(node.borrow().right.as_ref().map(|x| Rc::clone(x)), vec);
        }
    }
    let mut result = Vec::new();
    _preorder(root, &mut result);
    result
}

pub fn postorder(root: Option<Rc<RefCell<TreeNode>>>) -> Vec<i32> {
    fn _postorder(mut root: Option<Rc<RefCell<TreeNode>>>, vec: &mut Vec<i32>) {
        if let Some(node) = root {
            _postorder(node.borrow().left.as_ref().map(|x| Rc::clone(x)), vec);
            _postorder(node.borrow().right.as_ref().map(|x| Rc::clone(x)), vec);
            vec.push(node.borrow().val.clone());
        }
    }
    let mut result = Vec::new();
    _postorder(root, &mut result);
    result
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inorder() {
        assert_eq!(inorder(build_tree()), vec![5, 3, 4, 1, 2]);
    }

    #[test]
    fn test_preorder() {
        assert_eq!(preorder(build_tree()), vec![1, 3, 5, 4, 2]);
    }

    #[test]
    fn test_postorder() {
        assert_eq!(postorder(build_tree()), vec![5, 4, 3, 2, 1]);
    }

    fn build_tree() -> Option<Rc<RefCell<TreeNode>>> {
        let left2 = Some(Rc::new(RefCell::new(TreeNode { val: 5, left: None, right: None })));
        let right2 = Some(Rc::new(RefCell::new(TreeNode { val: 4, left: None, right: None })));
        let left = Some(Rc::new(RefCell::new(TreeNode { val: 3, left: left2, right: right2 })));
        let right = Some(Rc::new(RefCell::new(TreeNode { val: 2, left: None, right: None })));
        Some(Rc::new(RefCell::new(TreeNode { val: 1, left, right })))
    }

}
