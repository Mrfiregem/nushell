use std::sync::LazyLock;

use nu_test_support::prelude::*;

#[derive(Clone, IntoValue)]
struct StockItem {
    id: &'static str,
    rating: f64,
    out_of_stock: bool,
}

#[rustfmt::skip]
static STOCK_LIST: LazyLock<[StockItem; 4]> = LazyLock::new(|| [
    StockItem { id: "a1", rating: 8.3, out_of_stock: false },
    StockItem { id: "a2", rating: 6.0, out_of_stock: true  },
    StockItem { id: "b1", rating: 3.7, out_of_stock: false },
    StockItem { id: "c1", rating: 5.1, out_of_stock: false }
]);

#[test]
fn update_all_table_cells() -> Result {
    let code =
        "update cells { into string | describe } | values | flatten | all { $in == 'string' }";
    test()
        .run_with_data(code, STOCK_LIST.clone())
        .expect_value_eq(true)
}

#[test]
fn update_all_record_cells() -> Result {
    let code = "first | update cells { into string | describe } | values | all { $in == 'string' }";
    test()
        .run_with_data(code, STOCK_LIST.clone())
        .expect_value_eq(true)
}

#[test]
fn specify_table_columns() -> Result {
    let code = "
        reject rating
        | update cells -c [id] {|s| $s ++ '0' }
        | $in == [[id out_of_stock]; [a10 false] [a20 true] [b10 false] [c10 false]]
    ";
    test()
        .run_with_data(code, STOCK_LIST.clone())
        .expect_value_eq(true)
}

#[test]
fn specify_record_columns() -> Result {
    let code = "
        {a: 'not a bool', b: false, c: true}
        | update cells -c [b c] { not $in }
        | $in == {a: 'not a bool', b: true, c: false}
    ";
    test().run(code).expect_value_eq(true)
}

#[test]
fn table_error_displayed() -> Result {
    let err = test()
        .run_with_data("update cells {|x| $x + 100 }", STOCK_LIST.clone())
        .expect_shell_error()?;
    assert!(matches!(
        err,
        ShellError::OperatorIncompatibleTypes {
            lhs: nu_protocol::Type::String,
            rhs: nu_protocol::Type::Int,
            ..
        }
    ));
    Ok(())
}

#[test]
fn record_error_displayed() -> Result {
    let code = "version | update cells {|x| $x + 100 }";
    let err = test().run(code).expect_shell_error()?;
    assert!(matches!(
        err,
        ShellError::OperatorIncompatibleTypes {
            lhs: nu_protocol::Type::String,
            rhs: nu_protocol::Type::Int,
            ..
        }
    ));
    Ok(())
}
