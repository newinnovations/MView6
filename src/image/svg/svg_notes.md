# SVG notes

## SVG text measuring

In the Rust `usvg` library, calculating the extents (bounding box) of a text segment is not straightforward because `usvg` primarily focuses on parsing and simplifying SVG files, converting text into paths for rendering. However, you can compute the extents of a text segment by leveraging the `usvg` and `usvg_text_layout` crates, particularly after converting text nodes into paths. Below, I outline the approach based on available information and the library's capabilities.

### Key Points from `usvg` Documentation

- **Text to Paths Conversion**: `usvg` converts `<text>` elements into paths (groups of paths or images) during preprocessing when `usvg::Tree::postprocess` is called with `usvg::PostProcessingSteps::convert_text_into_paths`. This is necessary because `usvg` does not directly provide text metrics before conversion.[](https://docs.rs/usvg-tree/latest/usvg_tree/struct.Text.html)
- **Bounding Box**: After text is converted to paths, the resulting `Group` or `Path` nodes have an associated `object_bounding_box` that represents the bounding box in SVG terms. This bounding box is not tight (i.e., it may not precisely fit the text content) but provides a starting point.[](https://docs.rs/usvg-tree/latest/usvg_tree/struct.Text.html)
- **Text Layout**: The `usvg_text_layout` crate, built on top of `usvg`, offers utilities for handling text layout, including the `convert_text` function, which transforms text nodes into renderable paths.[](https://docs.rs/usvg-text-layout)

### Approach to Calculate Text Segment Extents

To calculate the extents of a text segment in `usvg`, you need to:

1. **Parse the SVG and Convert Text to Paths**:
   - Load your SVG using `usvg::Tree::from_str` or similar methods.
   - Enable text-to-path conversion by calling `usvg::Tree::postprocess` with `usvg::PostProcessingSteps::convert_text_into_paths`.
   - This step transforms `<text>` elements into `Group` nodes containing `Path` elements.

2. **Access the Text Node**:
   - Traverse the `usvg::Tree` to locate the `Text` node corresponding to your text segment.
   - After conversion, the `Text` node will be replaced by a `Group` containing paths representing the glyphs.

3. **Compute the Bounding Box**:
   - For each `Path` in the `Group`, use the `bounding_box` or `object_bounding_box` field to get the geometric bounds.
   - Combine the bounding boxes of all paths in the group to compute the overall extents of the text segment. This involves taking the minimum and maximum x/y coordinates across all paths.

4. **Optional: Use `usvg_text_layout`**:
   - The `usvg_text_layout` crate provides a `convert_text` function that processes text nodes into paths. You can use this to handle text layout explicitly and access glyph positions or bounding boxes.
   - This crate integrates with `fontdb` for font resolution, which is necessary for accurate glyph metrics.[](https://docs.rs/usvg-text-layout)[](https://docs.rs/usvg/latest/usvg/)

### Example Code

Below is a conceptual example of how you might calculate the extents of a text segment using `usvg` and `usvg_text_layout`. Note that this is a simplified version and assumes you have an SVG with a `<text>` element:

```rust
use usvg::{Tree, PostProcessingSteps, Node, Rect};
use usvg_text_layout::{fontdb, convert_text};

fn calculate_text_extents(svg_data: &str) -> Option<Rect> {
    // Parse the SVG
    let opt = usvg::Options::default();
    let mut tree = Tree::from_str(svg_data, &opt).ok()?;

    // Convert text to paths
    tree.postprocess(PostProcessingSteps::convert_text_into_paths(), &opt);

    // Initialize font database (required for text layout)
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts(); // Load system fonts or specify custom fonts

    // Find the text node and convert it
    for node in tree.root.descendants() {
        if let Node::Text(text) = &node {
            // Convert text to paths using usvg_text_layout
            let group = convert_text(&text, &opt, &mut fontdb)?;

            // Calculate bounding box by iterating over paths in the group
            let mut combined_bbox: Option<Rect> = None;
            for child in group.children() {
                if let Node::Path(path) = child {
                    if let Some(bbox) = path.bounding_box {
                        combined_bbox = match combined_bbox {
                            None => Some(bbox),
                            Some(existing) => Some(Rect::new(
                                existing.left().min(bbox.left()),
                                existing.top().min(bbox.top()),
                                existing.right().max(bbox.right()),
                                existing.bottom().max(bbox.bottom()),
                            )?),
                        };
                    }
                }
            }
            return combined_bbox;
        }
    }
    None
}

fn main() {
    let svg = r#"<svg><text x="10" y="20" font-family="Arial" font-size="16">Hello</text></svg>"#;
    if let Some(bbox) = calculate_text_extents(svg) {
        println!("Text extents: {:?}", bbox);
    } else {
        println!("No text extents calculated");
    }
}
```

### Explanation

- **Parsing and Conversion**: The SVG is parsed into a `usvg::Tree`, and text is converted to paths using `postprocess`. This ensures the text is represented as renderable paths.
- **Font Database**: `fontdb` is used to resolve fonts, which is critical for accurate text layout. You may need to load specific fonts if the default system fonts are insufficient.
- **Bounding Box Calculation**: The code iterates over the `Group` node's children (which are `Path` nodes after conversion) and combines their bounding boxes to compute the overall extents.
- **Rect**: The `Rect` type from `usvg` represents the bounding box with `left`, `top`, `right`, and `bottom` coordinates.

### Limitations

- **Non-Tight Bounding Box**: As noted in the documentation, the `object_bounding_box` for text in `usvg` is not a tight fit around the text content. You may need to process individual glyph paths for precise extents.[](https://docs.rs/usvg-tree/latest/usvg_tree/struct.Text.html)
- **Font Dependency**: Accurate extents depend on the font used. Ensure the correct fonts are loaded into `fontdb`.
- **Complex Text**: For text with multiple spans, transformations, or text-on-path, you may need to handle additional complexity, such as iterating over `TextChunk` or `TextSpan` nodes.[](https://docs.rs/usvg/latest/usvg/)

### Alternative Approach with `rusttype` or `ttf_parser`

If `usvg` does not provide sufficient precision, you can use the `rusttype` or `ttf_parser` crates to calculate text extents directly, as shown in the Stack Overflow example for `rusttype`. This involves:

- Loading the font with `rusttype::Font` or `ttf_parser::Face`.
- Using `rusttype::Font::layout` or `ttf_parser::Face::glyph_hor_advance` to compute glyph widths and heights.
- This approach requires manually handling font metrics and does not integrate directly with `usvg`'s SVG parsing but can be more precise for raw text measurements.[](https://stackoverflow.com/questions/68151488/rusttype-get-text-width-for-font)

Example using `rusttype` (adapted from Stack Overflow):

```rust
use rusttype::{Font, Scale, point};

fn measure_text(font: &Font, text: &str, font_size: f32) -> (f32, f32) {
    let scale = Scale::uniform(font_size);
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font.layout(text, scale, point(0.0, 0.0)).collect();
    let width = glyphs
        .last()
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .unwrap_or(0.0);
    let height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
    (width, height)
}
```

### Recommendations

- **Use `usvg_text_layout`**: For most cases, rely on `usvg_text_layout::convert_text` and process the resulting `Group` to compute extents, as it integrates well with `usvg`.
- **Check Font Availability**: Ensure the fonts specified in the SVG are available in `fontdb` to avoid fallback issues.
- **Verify with Testing**: Test with different fonts, sizes, and text content to ensure the bounding box meets your needs, as SVG text rendering can vary.
- **Consider `rusttype` for Precision**: If you need pixel-precise measurements and are not constrained to `usvg`'s pipeline, use `rusttype` or `ttf_parser` for direct font metrics.
