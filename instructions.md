Build a cross-platform image compressor app called "Kompressor" for Linux, macOS and Windows using Rust.

# Application window

The UI is built using [GPUI](https://gpui.rs).
Default to a 450x225 window size on start.
Support light and dark colour scheme and adapt automatically to the system appearance.
Use the default system font (e.g. Segoe UI on Windows and system-ui on macOS).
The main app window is a drop zone for dragging images into.
It shows a list of images in the current session as a scrollable table.
Each item in the list shows the file name, the original file size and the output file size.
The output file size should show as 'processing' while it is being processed.
The bottom of the window is like a status bar which is not part of the scrollable area.
This status bar shows a set of radio boxes on the left to target the output file type: JPG, PNG and WEBP.
Default to the first item in the set.
Changing the output file type only affects new images, never the ones already in the file list.
On the right side of the status bar is a slider to select the output quality, 0-100%.
Same as the output file type, changing this value only affects new images.

# Behaviour

Validate the input file type first.
Supported input file types are: JPG, JPEG and PNG (no WEBP as input).
Processing starts automatically when an image is dragged into the app.
The output overwrites the original input file if the file type is unchanged.
Any processing error stops the affected item and a popup is displayed to show the error message.

# Coding style

The code should be readable and maintainable. Keep it simple.
Since an image compression library may support a small number of file types,
a wrapper interface should be used so that we can plug in multiple libraries.
The app selects the compression library based on the output–library map.
For now all supported output types will use [libcaesium](https://github.com/Lymphatus/libcaesium).
Commit to git regularly at important checkpoints using 'conventional commit' messages.

# Out of scope

- CI/CD pipeline

