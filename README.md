# Crabviz

A static code analysis tool that generates interactive call graph.

## Introdution

Crabviz is based on the Langauge Server Protocol,
as long as you have the corresponding language server installed,
you can generate call graph with Crabviz for your project.

## Features

* Show types, methods, functions and interfaces, grouped by file
* Display function calling relationships and interface implementation relationships
* Specify folders and files for analysis

## Preview

![preview](https://user-images.githubusercontent.com/20551552/242812058-60584f59-a8f0-4a56-90eb-373c3f3b8cd5.gif)

## Editors

Because Crabviz utilizes the capabilities of LSP server, it is better suited as an IDE/editor extension instead of a standalone command line tool.

It is currently available in [VS Code](editors/code/), and PRs for other editors are welcome.

## TODO

* Collapse folder
* Beautify UI

## Credits

crabviz is inspired by [graphql-voyager](https://github.com/APIs-guru/graphql-voyager) and [
go-callvis](https://github.com/ofabry/go-callvis)
