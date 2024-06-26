use marshal_update::forest::{Forest, ForestRoot};
use marshal_update::tester::Tester;

#[test]
fn test_forest() -> anyhow::Result<()> {
    let mut forest = Forest::new();
    let tree = forest.add(4u8);
    let mut root = ForestRoot::new(forest, tree);
    Tester::new(root, r#"{

}"#);
    Ok(())
}
