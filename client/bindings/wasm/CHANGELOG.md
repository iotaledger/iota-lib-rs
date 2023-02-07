# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- ## Unreleased - YYYY-MM-DD

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security -->

## 1.0.0-alpha.2 - 202x-MM-DD

### Added

- `aliasIdToBech32()`;
- `nftIdToBech32()`;
- `computeAliasId()`;
- `computeNftId()`;
- `computeFoundryId()`;

### Changed

- Updated dependencies;
- Renamed `IInputSigningData::outputMetaData` to `IInputSigningData::outputMetadata`;
- Changed `ISegment::bs` from `Uint8Array` to `number[]` so that the serialization corresponds to what is expected;

### Fixed

- Returned JSON value for `IInputSigningData`;

## 1.0.0-alpha.1 - YYYY-MM-DD

Initial release of the wasm bindings.