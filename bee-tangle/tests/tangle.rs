mod helpers;

use self::helpers::*;

#[test]
fn count_tips() {
    let (tangle, _, _) = create_test_tangle();

    assert_eq!(1, tangle.num_tips());
}
