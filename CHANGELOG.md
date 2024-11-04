# Changelog

## [0.1.1](https://github.com/fortress-build/whirlwind/compare/v0.1.0...v0.1.1) (2024-11-04)


### Miscellaneous Chores

* update minor version ([f36a463](https://github.com/fortress-build/whirlwind/commit/f36a463c0b6b7886282fbe68d45e76edace1beae))

## [0.1.0](https://github.com/fortress-build/whirlwind/compare/v0.1.0-rc1...v0.1.0) (2024-11-03)


### Features

* allow custom hasher implementations ([0dbc985](https://github.com/fortress-build/whirlwind/commit/0dbc9857825786e7bf2dfc8b4364cddd331b5bc4))


### Bug Fixes

* add `crossbeam_utils` dependency ([f9ad383](https://github.com/fortress-build/whirlwind/commit/f9ad38370be37543dd3af81bc217658094a93b7a))
* versioning for release-please ([1d51202](https://github.com/fortress-build/whirlwind/commit/1d51202687c710c8fc441bb3cb9d201530956250))


### Performance Improvements

* use `hashbrown::HashTable` for shards ([f0d45f1](https://github.com/fortress-build/whirlwind/commit/f0d45f1d4b110b6b8c36439e375b4f7c685e4a8e))
* use `hashbrown::RawTable` + `dashmap`'s shard selection strategy ([c47f3ef](https://github.com/fortress-build/whirlwind/commit/c47f3efb0715ace74e75ecdeee840298d4da34db))
