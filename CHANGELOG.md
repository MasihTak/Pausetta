# Changelog

## [0.2.1](https://github.com/MasihTak/Pausetta/compare/v0.2.0...v0.2.1) (2026-10-07)


### Bug Fixes

* **autostart:** respect a Startup apps disable ([5867377](https://github.com/MasihTak/Pausetta/commit/5867377ff8f4ae7f7c069d9dff24838d350683fc))
* **autostart:** respect a Startup apps disable ([3c0105f](https://github.com/MasihTak/Pausetta/commit/3c0105f9493919f3d54df92daa7cadd423052904))
* **autostart:** rewrite a login item left pointing at an old path ([c8ca251](https://github.com/MasihTak/Pausetta/commit/c8ca251464d85bcfae924c75a63b2c865f039b2c))
* **autostart:** rewrite a login item left pointing at an old path ([79761bb](https://github.com/MasihTak/Pausetta/commit/79761bb0ff31be0abdb7f513adad0b0ae7092971))
* **reminder:** stack concurrent toasts instead of replacing the open one ([c6217ab](https://github.com/MasihTak/Pausetta/commit/c6217abdd38ea53beeb79d0609b5c1aa32b832a6))
* **reminder:** stack concurrent toasts instead of replacing the open one ([7ad228c](https://github.com/MasihTak/Pausetta/commit/7ad228cd9123f7c76921070f3aa6ce99cdd72587)), closes [#4](https://github.com/MasihTak/Pausetta/issues/4)
* **scheduler:** retry a reminder whose toast window failed to open ([864a730](https://github.com/MasihTak/Pausetta/commit/864a7300bf076aa1d767e408845daefba52491de))
* **scheduler:** retry a reminder whose toast window failed to open ([4ff3087](https://github.com/MasihTak/Pausetta/commit/4ff3087df47bd8575002faa489fb6a1e37009fa1)), closes [#10](https://github.com/MasihTak/Pausetta/issues/10)

## [0.2.0](https://github.com/MasihTak/Pausetta/compare/v0.1.0...v0.2.0) (2026-09-24)


### Features

* allow only one running instance ([9bddab4](https://github.com/MasihTak/Pausetta/commit/9bddab42be12b3fd5681b0cbd3dc983ff3db88d1))
* **settings:** show why a settings change was rejected ([14cdd6d](https://github.com/MasihTak/Pausetta/commit/14cdd6dbe87ac8f03764036c243b892cbae4ba73))
* **toast:** pause auto-dismiss on hover and keep eye reminders up for 22s ([a062296](https://github.com/MasihTak/Pausetta/commit/a062296ebbbb249186653324f5104df91c8bee7c))


### Bug Fixes

* **app:** keep loading settings when a window listener fails ([70b1ef3](https://github.com/MasihTak/Pausetta/commit/70b1ef3758d2f326796ff5c06747db8777fcbb6c))
* **autostart:** keep starting up when the login item sync fails ([b403989](https://github.com/MasihTak/Pausetta/commit/b40398904287b5fe24d7f19038a27eae4de4c866))
* **reminder:** let clicks through the toast's transparent margin ([15219ef](https://github.com/MasihTak/Pausetta/commit/15219ef7417c655f7cd3fc85ce6396a0967839d7))
* **scheduler:** keep reminders on time when the clock jumps back ([43faeec](https://github.com/MasihTak/Pausetta/commit/43faeec82c1893a7c4b047ab69c4e43c552c3891))
* **scheduler:** play the chime only after the toast opens ([6816b8d](https://github.com/MasihTak/Pausetta/commit/6816b8df68b7a08888a3b75946706852c61168cc))
* **scheduler:** skip only missed reminders instead of every tick after a gap ([16c8425](https://github.com/MasihTak/Pausetta/commit/16c8425a0b341b15ce0ab5a88f811d638e0a9f95))
* **settings-window:** fit the window height to the screen ([0638de2](https://github.com/MasihTak/Pausetta/commit/0638de28737af4a86a9f897e7e69ff1c359253a5))
* **settings:** apply the login item before saving settings ([89f5817](https://github.com/MasihTak/Pausetta/commit/89f5817a5654aef9254ceca97f7631bcd2d1d4df))
* **settings:** commit each migration with its version bump ([93ec6ea](https://github.com/MasihTak/Pausetta/commit/93ec6ea41e1923e69f40415ce2582e7f85133deb))
* **settings:** restore the mis-encoded en dash in the interval comment ([0d91645](https://github.com/MasihTak/Pausetta/commit/0d91645f2638dc9656e0b876c93b04c0a0b6d3f5))
* **settings:** serialize settings writes behind a lock ([1e39c93](https://github.com/MasihTak/Pausetta/commit/1e39c93a08affe9d2f96d9cbd23b87e3f161e252))
* **toast:** open each reminder in a uniquely labelled window ([f507f09](https://github.com/MasihTak/Pausetta/commit/f507f097f4dd90ed6e127b77eaba111d2a4ca63c))

## [0.1.0](https://github.com/MasihTak/Pausetta/compare/v0.1.0...v0.1.0) (2026-09-20)


### Features

* initial release ([2bf2f6a](https://github.com/MasihTak/Pausetta/commit/2bf2f6ac669169d1eb5042e583e7bbd23a5fd6d0))


### Miscellaneous Chores

* release 0.1.0 ([78aeefd](https://github.com/MasihTak/Pausetta/commit/78aeefd223036cffed1479e4888d3f6041c8a8b4))
