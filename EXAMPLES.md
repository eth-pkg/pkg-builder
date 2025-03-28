# Examples

This document contains example usage for different package types. All examples assume you have already built and installed pkg-builder:

```bash
cargo build && cargo install --path .
```

## Virtual Package

```bash
pkg-builder env create examples/bookworm/virtual-package/pkg-builder.toml
pkg-builder package examples/bookworm/virtual-package/pkg-builder.toml
```

## Rust Package

```bash
pkg-builder env create examples/bookworm/rust/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/rust/hello-world/pkg-builder.toml
```

## TypeScript Package

```bash
pkg-builder env create examples/bookworm/typescript/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/typescript/hello-world/pkg-builder.toml
```

## JavaScript Package

```bash
pkg-builder env create examples/bookworm/javascript/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/javascript/hello-world/pkg-builder.toml
```

## Nim Package

```bash
pkg-builder env create examples/bookworm/nim/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/nim/hello-world/pkg-builder.toml
```

## .NET Package

```bash
pkg-builder env create examples/bookworm/dotnet/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/dotnet/hello-world/pkg-builder.toml
```

## Java Package

```bash
pkg-builder env create examples/bookworm/java/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/java/hello-world/pkg-builder.toml
```

## Testing Examples

After building a package, you can run specific tests:

### Piuparts Only

```bash
pkg-builder piuparts examples/bookworm/virtual-package/pkg-builder.toml
```

### Autopkgtest Only

```bash
pkg-builder autopkgtests examples/bookworm/virtual-package/pkg-builder.toml
```