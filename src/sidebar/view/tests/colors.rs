use super::*;

#[test]
fn the_title_and_status_use_the_configured_colors() {
    let theme = Theme {
        accent: Color::Magenta,
        done: Color::Indexed(82),
        ..Theme::default()
    };
    let model = Model {
        theme,
        ..model(Status::Done, "", false)
    };
    let lines = lines(&model, &EN, WIDTH, model.height);
    let colors = |i: usize| -> Vec<Option<Color>> {
        lines
            .get(i)
            .map(|l| l.spans.iter().map(|s| s.style.fg).collect())
            .unwrap_or_default()
    };
    assert!(colors(0).contains(&Some(theme.accent)));
    assert!(colors(3).contains(&Some(theme.done)));
}
