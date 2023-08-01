use output_coloring::*;

/// The templates for colored terminal output

// These templates using a certain background color
pub const INFO_TEMPLATE: ColoredTemplate = ColoredTemplate {
    ft_color: Color::White,
    bg_color: Some(Color::Green),
    style: Style::Bold,
};
pub const WARN_TEMPLATE: ColoredTemplate = ColoredTemplate {
    ft_color: Color::White,
    bg_color: Some(Color::Yellow),
    style: Style::Bold,
};
pub const ERROR_TEMPLATE: ColoredTemplate = ColoredTemplate {
    ft_color: Color::White,
    bg_color: Some(Color::Red),
    style: Style::Bold,
};

// These templates using no background color
pub const INFO_TEMPLATE_NO_BG_COLOR: ColoredTemplate = ColoredTemplate {
    ft_color: Color::Green,
    bg_color: None,
    style: Style::Bold,
};
pub const WARN_TEMPLATE_NO_BG_COLOR: ColoredTemplate = ColoredTemplate {
    ft_color: Color::Yellow,
    bg_color: None,
    style: Style::Bold,
};
pub const ERROR_TEMPLATE_NO_BG_COLOR: ColoredTemplate = ColoredTemplate {
    ft_color: Color::Red,
    bg_color: None,
    style: Style::Bold,
};
