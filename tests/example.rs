use templing::*;

#[test]
fn test_hello() {
    let name = "world";
    assert_eq!(templing!("Hello, {{name}}!").trim(), "Hello, world!");
}

#[test]
fn test_for() {
    let data = vec![1, 2, 3, 4, 5];
    assert_eq!(
        templing!("{{# for value in data { }}{{value}}{{# } }}").trim(),
        "12345",
    );
}

#[test]
fn test_for_included() {
    let data = vec![1, 2, 3, 4, 5];
    assert_eq!(
        include_templing!("tests/for.templing").trim(),
        "1\n2\n3\n4\n5",
    );
}

#[test]
fn test_included() {
    assert_eq!(
        include_templing!("tests/included.templing").trim(),
        "Included test"
    );
}
