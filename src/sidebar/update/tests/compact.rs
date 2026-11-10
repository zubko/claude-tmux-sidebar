use super::*;
use crate::sidebar::compact::Compact;

/// With the cursor on heading `s`, the rows are `s`, Claude `a`, Claude `b`: no detail lines.
#[test]
fn clicks_in_a_compact_list_hit_the_row_drawn_there() {
    let model = loaded(
        Model {
            height: 20,
            compact: Compact::On,
            ..Model::default()
        },
        snapshot(false),
    );
    let model = update(model, Msg::Action(Action::First)).0;
    let (model, _) = update(
        model,
        Msg::Clicked {
            y: rows::FIRST_ROW + 2,
        },
    );
    assert_eq!(selected(&model), Some(&claude_key("b")));
}

/// With the cursor on Claude `b`, the rows are `s`, `a`, `b` and the detail line of `b`.
#[test]
fn the_selected_session_keeps_a_clickable_detail_line() {
    let model = loaded(
        Model {
            height: 20,
            compact: Compact::On,
            ..Model::default()
        },
        snapshot(false),
    );
    let model = update(model, Msg::Action(Action::Last)).0;
    let click = |line: u16| {
        let y = rows::FIRST_ROW + line;
        update(model.clone(), Msg::Clicked { y }).0
    };
    assert_eq!(selected(&click(1)), Some(&claude_key("a")));
    assert_eq!(selected(&click(2)), Some(&claude_key("b")));
    assert_eq!(selected(&click(3)), Some(&claude_key("b")));
}
