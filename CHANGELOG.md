# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Action to open video in the browser.

### Fixed

- Outdated window title
- Wrong offset of the reload-spinner in relation to the reload-button.

## [1.14.3] - 2024-01-11

### Added

- "Livi" video player in the list of predefined video players.
- Option to hide all short videos.

## [1.14.2] - 2023-12-08

### Added

- Video title as tooltip of the video.
- Video title in the video information screen.
- Error handling when failing to add a subscription.
- Error handling when failing to play or download videos.

## [1.14.1] - 2023-11-11

### Added

- Keyboard shortcuts for actions.

### Fixed

- Video duration placed out-of-bounds if thumbnail does not load.

### Chores

- Updated dependencies.

## [1.14.0] - 2023-10-14

### Added

- A dropdown for some popular video players.
- A dropdown for some popular Piped APIs.

### Changed

- Removed buttons in the list in favor of a menu shown on right-click or long-press (touch-screen only). 
- Updated to GTK 4.12 and Libadwaita 1.4.
- Use GridView instead of ListView for all pages.
- Improvements regarding UI in the add-subscription dialog.
- The button on the empty feed page will now lead to the subscription page instead of directly adding a subscription.

### Fixed

- Inconsistent size of video thumbnails.

## [1.13.1] - 2023-08-24

### Fixed

- Copy video URL not working anymore.
- Missing accesibility labels.

## [1.13.0] - 2023-08-20

### Added

- Dialog showing video information including likes, dislikes (not for YouTube), views and video description.
- Show video duration on video thumbnails.

### Removed

- Removed Lbry support as it will have to shut down soon.

### Fixed

- File chooser dialog for importing videos not working.

[Unreleased]: https://gitlab.com/schmiddi-on-mobile/pipeline/-/compare/v1.14.3...master
[1.14.3]: https://gitlab.com/schmiddi-on-mobile/pipeline/-/compare/v1.14.2...v1.14.3
[1.14.2]: https://gitlab.com/schmiddi-on-mobile/pipeline/-/compare/v1.14.1...v1.14.2
[1.14.1]: https://gitlab.com/schmiddi-on-mobile/pipeline/-/compare/v1.14.0...v1.14.1
[1.14.0]: https://gitlab.com/schmiddi-on-mobile/pipeline/-/compare/v1.13.1...v1.14.0
[1.13.1]: https://gitlab.com/schmiddi-on-mobile/pipeline/-/compare/v1.13.0...v1.13.1
[1.13.0]: https://gitlab.com/schmiddi-on-mobile/pipeline/-/compare/v1.12.0...v1.13.0
