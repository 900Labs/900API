# Roadmap

This roadmap describes product direction, not a delivery promise. New work must preserve local-first operation, low-bandwidth usability, and support for older hardware.

## Available in 0.2.1

- REST workbench with tabs, collections, nested folders, environments, history, request settings, scripts, response search, and saved examples
- JSON, raw text, text-only multipart fields, and URL-encoded request bodies
- GraphQL requests, variables, introspection, query assistance, and schema exploration
- WebSocket and SSE clients with visible errors and bounded event history
- Unary gRPC over h2c or TLS with raw protobuf bytes and scalar field assistance
- Local mock server with loopback defaults
- Postman, OpenAPI, Swagger, cURL, and portable 900API import paths where documented
- Portable collection export, Git Sync round trips, and OpenAPI export
- Headless CLI collection runs with bounded saved scripts and console, JSON, or JUnit reporting
- Local API documentation generation
- Plugin manifest metadata and local workspace planning records
- Canonical, version-checked release packages for macOS arm64 and Intel, Linux, and Windows before checksum publication

## Near-Term Work

- Signed and notarized release artifacts when project signing infrastructure is available
- Platform install and upgrade smoke tests in CI
- More frontend component and desktop IPC integration tests
- Better import diagnostics for partially compatible Postman and OpenAPI documents
- Data-driven CLI runs and request chaining
- Response size limits and streaming views for very large payloads
- Full gRPC descriptors, server reflection, and streaming methods
- Accessibility testing across keyboard, screen reader, contrast, and localization paths

## Later Exploration

- File change review and conflict assistance for Git Sync
- More import and export adapters through reviewed local plugins
- Additional translations maintained by native speakers
- Optional workspace bundles for sharing templates without accounts or live collaboration

## Current Boundaries

- No hosted account, cloud workspace, or live multi-user collaboration
- No executable third-party plugin hooks
- No binary request body or multipart file-part persistence
- No automatic OAuth login flow
- No gRPC streaming or server reflection
- No automatic application update service
