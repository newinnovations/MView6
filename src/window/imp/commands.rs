use crate::window::imp::MViewWindowImp;

#[derive(Clone)]
pub struct Command {
    pub name: &'static str,
    pub shortcut: Option<&'static str>,
    pub action: fn(&MViewWindowImp),
}

pub const COMMANDS: &[Command] = &[
    Command {
        name: "Open file",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Zoom: No scaling",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Zoom: Fit window",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Zoom: Fill window",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Zoom: Maximum zoom",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Transparency background: Checkerboard",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Transparency background: White",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Transparency background: Black",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Rotate 90° Clockwise",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Rotate 90° Counterclockwise",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Rotate 180°",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Page mode: Single",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Page mode: Dual (1, 2-3, 4-5, ...)",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Page mode: Dual (1-2, 3-4, 5-6, ...)",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "PDF backend: MuPDF",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "PDF backend: PDFium",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Toggle Files pane",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Toggle Information pane",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Thumbnail size: Extra small (80 px)",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Thumbnail size: Small (100 px)",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Thumbnail size: Medium (140 px)",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Thumbnail size: Large (175 px)",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Thumbnail size: Extra large (250 px)",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Show thumbnails",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Slideshow interval: 1 second",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Slideshow interval: 3 seconds",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Slideshow interval: 5 seconds",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Slideshow interval: 10 seconds",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Slideshow interval: 30 seconds",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Slideshow interval: 1 minute",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Start slideshow",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Stop slideshow",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Toggle full screen",
        shortcut: Some("F"),
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "About MView6",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Help screen 1",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Help screen 2",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Quit MView6",
        shortcut: None,
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Edit navigation filter",
        shortcut: Some("Shift+F"),
        action: |w| w.adjust_filter(),
    },
];
