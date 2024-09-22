# wordpress-singlepage

Generate a giant page with all Wordpress posts with a given tag.

## Usage
- First, [export your WordPress site](https://wordpress.com/support/export/).
- Then unzip the output and move the .xml file into this directory and name it "wordpress.xml".
  - **NOTE** - if you have any password-protected posts, those posts and passwords will be in this .xml
    file, so don't share it!
- (optional) Modify `style.css` to change the CSS that will be generated.
- `cargo run --release -- -t <tag name>` will generate `output.html`.
  - Run `cargo run --release -- --help` to see all options.
- Enjoy!