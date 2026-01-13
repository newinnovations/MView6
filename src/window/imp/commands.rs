use crate::window::imp::MViewWindowImp;

#[derive(Clone)]
pub struct Command {
    pub name: &'static str,
    pub shortcut: Option<&'static str>,
    pub action: fn(&MViewWindowImp),
}

pub const COMMANDS: &[Command] = &[
    Command {
        name: "About MView6",
        shortcut: None,
        action: |w| w.show_about_dialog(),
    },
    Command {
        name: "Edit navigation filter",
        shortcut: Some("Shift+F"),
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Help screen 1",
        shortcut: None,
        action: |w| w.show_help_page(1),
    },
    Command {
        name: "Help screen 2",
        shortcut: None,
        action: |w| w.show_help_page(2),
    },
    Command {
        name: "Open file",
        shortcut: None,
        action: |w| w.open_file(),
    },
    Command {
        name: "PDF backend: MuPDF",
        shortcut: None,
        action: |w| w.change_pdf_provider("mupdf"),
    },
    Command {
        name: "PDF backend: PDFium",
        shortcut: None,
        action: |w| w.change_pdf_provider("pdfium"),
    },
    Command {
        name: "Page mode: Single",
        shortcut: None,
        action: |w| w.change_page_mode("single"),
    },
    Command {
        name: "Page mode: Dual (1, 2-3, 4-5, ...)",
        shortcut: None,
        action: |w| w.change_page_mode("deo"),
    },
    Command {
        name: "Page mode: Dual (1-2, 3-4, 5-6, ...)",
        shortcut: None,
        action: |w| w.change_page_mode("doe"),
    },
    Command {
        name: "Quit MView6",
        shortcut: Some("q"),
        action: |w| w.quit(),
    },
    Command {
        name: "Rotate 90° Clockwise",
        shortcut: None,
        action: |w| w.rotate_image(270),
    },
    Command {
        name: "Rotate 90° Counterclockwise",
        shortcut: None,
        action: |w| w.rotate_image(90),
    },
    Command {
        name: "Rotate 180°",
        shortcut: None,
        action: |w| w.rotate_image(180),
    },
    Command {
        name: "Slideshow interval: 1 second",
        shortcut: None,
        action: |w| w.set_slideshow_interval(1),
    },
    Command {
        name: "Slideshow interval: 3 seconds",
        shortcut: None,
        action: |w| w.set_slideshow_interval(3),
    },
    Command {
        name: "Slideshow interval: 5 seconds",
        shortcut: None,
        action: |w| w.set_slideshow_interval(5),
    },
    Command {
        name: "Slideshow interval: 10 seconds",
        shortcut: None,
        action: |w| w.set_slideshow_interval(10),
    },
    Command {
        name: "Slideshow interval: 30 seconds",
        shortcut: None,
        action: |w| w.set_slideshow_interval(30),
    },
    Command {
        name: "Slideshow interval: 60 seconds",
        shortcut: None,
        action: |w| w.set_slideshow_interval(60),
    },
    Command {
        name: "Start slideshow",
        shortcut: None,
        action: |w| w.set_slideshow_active(true),
    },
    Command {
        name: "Stop slideshow",
        shortcut: None,
        action: |w| w.set_slideshow_active(false),
    },
    Command {
        name: "Thumbnail size: Extra small (80 px)",
        shortcut: None,
        action: |w| w.set_thumbnail_size(80),
    },
    Command {
        name: "Thumbnail size: Small (100 px)",
        shortcut: None,
        action: |w| w.set_thumbnail_size(100),
    },
    Command {
        name: "Thumbnail size: Medium (140 px)",
        shortcut: None,
        action: |w| w.set_thumbnail_size(140),
    },
    Command {
        name: "Thumbnail size: Large (175 px)",
        shortcut: None,
        action: |w| w.set_thumbnail_size(175),
    },
    Command {
        name: "Thumbnail size: Extra large (250 px)",
        shortcut: None,
        action: |w| w.set_thumbnail_size(250),
    },
    Command {
        name: "Toggle Files pane",
        shortcut: Some("space"),
        action: |w| w.toggle_pane_files(),
    },
    Command {
        name: "Toggle Information pane",
        shortcut: Some("i"),
        action: |w| w.toggle_pane_info(),
    },
    Command {
        name: "Toggle full screen",
        shortcut: Some("F"),
        action: |w| w.toggle_fullscreen(),
    },
    Command {
        name: "Toggle thumbnail view",
        shortcut: Some("t"),
        action: |w| w.toggle_thumbnail_view(),
    },
    Command {
        name: "Transparency background: Black",
        shortcut: None,
        action: |w| w.change_transparency("black"),
    },
    Command {
        name: "Transparency background: Checkerboard",
        shortcut: None,
        action: |w| w.change_transparency("checkerboard"),
    },
    Command {
        name: "Transparency background: White",
        shortcut: None,
        action: |w| w.change_transparency("white"),
    },
    Command {
        name: "Zoom: Fill window",
        shortcut: None,
        action: |w| w.change_zoom("fill"),
    },
    Command {
        name: "Zoom: Fit window",
        shortcut: None,
        action: |w| w.change_zoom("fit"),
    },
    Command {
        name: "Zoom: Maximum zoom",
        shortcut: None,
        action: |w| w.change_zoom("max"),
    },
    Command {
        name: "Zoom: No scaling",
        shortcut: None,
        action: |w| w.change_zoom("nozoom"),
    },
];
