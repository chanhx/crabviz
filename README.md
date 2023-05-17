# Crabviz

a static code analysis tool that generates interactive call graph

## Introdution

Crabviz is based on the Langauge Server Protocol,
as long as you have the corresponding language server installed,
you can generate call graph with Crabviz for your project.

## Features

* show types, methods and functions, and group them by file
* display function calling relationships and interface implementation relationships

## Preview

![preview](https://user-images.githubusercontent.com/20551552/238906454-8bc073c1-b593-4a99-84f5-5bfdd9525d7c.gif)

## Editors

Because Crabviz utilizes the capabilities of LSP server, it is more suitable as an editor extension than a command line tool.

It is currently available in [VS Code](editors/code/), and PRs for other editors are welcome.

## TODO

* Specify files to scan and ignore
* Collapse folder
* Beautify UI

## Credits

crabviz is inspired by [graphql-voyager](https://github.com/APIs-guru/graphql-voyager) and [
go-callvis](https://github.com/ofabry/go-callvis)
