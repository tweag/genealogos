# Introduction

Genealogos is a project to empower software user, packages, developer in helping to understand the dependency chain of software they use.

It's a Bill Of Material (BOM) generator, pulling information from [nixpkgs](https://github.com/NixOS/nixpkgs), and flakes using nixpkgs. It generates CycloneDX BOM files, and static HTML reports.

# Design

Genealogos is designed as a traditional UNIX program, using stdin and stdout to get data from / output data. This allows greater composability and reuse.

There are at least three components to achieve the complete task.

- Data extraction from nixpkgs / a flake
- CycloneDX BOM generation from the previous step
- static HTML report from the previous step

## Data extraction

The user gives a package as a flake URL (like `nixpkgs#sacc`) and should obtain a JSON file with the derivation attributes of the package derivations and all its dependencies. This mean, the program should recursively get all transient dependencies of the given package.

The JSON file must keep the dependency information as it's required in ther CycloneDX format.

## CycloneDX BOM file

The program receives a JSON on its standard input or a filename as a parameter and outputs the CycloneDX file.

We may want the program to support multiple CycloneDX formats as it's evolving very often, so it may make sense to have files containing a mapping of input into the output.

## Static HTML report generator

The program receives a CycloneDX file on input or a filename as a parameter, and a directory path as a parameter for the output path.

It must creates a subdirectory named after the package name, and will contain an html file.

The output directory must contains an index.html with a link to each subdirectories. This index file must be created everytime an entry is added/updated.

It may be needed to support packages with multiple versions, or multiple BOMs versions in each subdirectory.
